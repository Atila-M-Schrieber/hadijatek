use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use eyre::eyre;
use eyre::Result;
use petgraph::csr::Csr;
use petgraph::visit::{IntoNeighbors, IntoNodeReferences};
use petgraph::Undirected;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Namespace;
use surrealdb::Response;
use surrealdb::Surreal;
use tokio::runtime::Runtime;

use crate::draw::{Color, Shape};
use crate::game::region::{Base, Border, Region, RegionType};
use crate::game::team::Team;
use crate::game::State;

use super::{Database, Prelude};

pub struct Surrealdb<'a> {
    name: String,
    water_stroke: Color,
    land_stroke: Color,
    address: String, // IP adress / webside
    credentials: Namespace<'a>,
}

impl Surrealdb<'_> {
    pub fn new<'a>(
        name: &'a str,
        ip_address: &'a str,
        username: &'a str,
        password: &'a str,
        water_stroke: Color,
        land_stroke: Color,
    ) -> Surrealdb<'a> {
        Surrealdb {
            name: name.to_owned(),
            water_stroke,
            land_stroke,
            credentials: Namespace {
                namespace: "hadijatek",
                username,
                password,
            },
            address: ip_address.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SerializedBase {
    owner_name: Option<String>,
}

fn serialize_base(base: &Base) -> SerializedBase {
    SerializedBase {
        owner_name: base.owner().map(|team| team.name().to_owned()),
    }
}

fn deserialize_base(sbase: SerializedBase, teams: &[Rc<Team>]) -> Base {
    let mut base = Base::new();
    if let Some(team_name) = sbase.owner_name {
        // Should always work
        if let Some(team) = teams.iter().find(|t| t.name() == &team_name) {
            base.set(Rc::clone(team));
        }
    }
    base
}

#[derive(Debug, Serialize, Deserialize)]
struct SerializedRegion {
    name: String,
    region_type: RegionType,
    base: Option<SerializedBase>,
    shape: Shape,
    color: Color,
}

fn serialize_region(region: &Rc<Region>) -> SerializedRegion {
    SerializedRegion {
        name: region.name().to_string(),
        region_type: region.region_type(),
        base: region.base().as_ref().map(|base| serialize_base(base)),
        shape: region.shape().clone(),
        color: region.color(),
    }
}

fn deserialize_region(sregion: SerializedRegion, teams: &[Rc<Team>]) -> eyre::Result<Region> {
    let region = Region::new(
        sregion.name,
        sregion.region_type,
        sregion
            .base
            .map(|b| RefCell::new(deserialize_base(b, teams))),
        sregion.shape,
        sregion.color,
    )?;
    Ok(region)
}

#[derive(Debug, Serialize, Deserialize)]
struct SerializedBorder {
    border_type: String,
    strait_region: Option<String>,
}

fn serialize_border(border: &Border) -> SerializedBorder {
    match border {
        Border::Strait(region) => SerializedBorder {
            border_type: "Strait".into(),
            strait_region: Some(region.name().to_string()),
        },
        _ => SerializedBorder {
            border_type: format!("{:?}", border),
            strait_region: None,
        },
    }
}

fn deserialize_border(
    sborder: SerializedBorder,
    regions: &Csr<Rc<Region>, Border, Undirected>,
) -> eyre::Result<Border> {
    let get_strait = |strait_string: Option<String>| -> eyre::Result<Rc<Region>> {
        let strait_string = strait_string.ok_or(eyre!("Strait without region"))?;
        let region = regions
            .node_references()
            .find(|(_, region)| region.name() == strait_string);
        let region = region.ok_or(eyre!("Region {} not found", strait_string))?.1;
        Ok(Rc::clone(region))
    };

    use Border::*;
    let border = match sborder.border_type.as_str() {
        "Land" => Land,
        "Shore" => Shore,
        "Sea" => Sea,
        "Strait" => Strait(get_strait(sborder.strait_region)?),
        s => return Err(eyre!("Non-existent border-type: {}", s)),
    };
    Ok(border)
}

impl Database for Surrealdb<'_> {
    fn load(&self) -> eyre::Result<()> {
        todo!()
    }

    fn write(&self) -> eyre::Result<()> {
        Ok(())
    }

    fn to_state(&self) -> eyre::Result<crate::game::State> {
        let rt = Runtime::new()?;
        let future = async {
            let db = Surreal::new::<Ws>(self.address.as_str()).await?;
            // Namespace apparently implements clone, but .clone() is not found...
            db.signin(Namespace {
                namespace: self.credentials.namespace.clone(),
                username: self.credentials.username.clone(),
                password: self.credentials.password.clone(),
            })
            .await?;

            db.use_ns("hadijatek").use_db(&self.name).await?;

            let prelude: Prelude = db.select(("prelude", "prelude")).await?;

            let teams: Vec<Rc<Team>> = db.select("team").await?.into_iter().map(Rc::new).collect();

            let region_: Vec<Region> = db
                .select("region")
                .await?
                .into_iter()
                .filter_map(|r| deserialize_region(r, &teams).ok())
                .collect();
            let mut regions: Csr<Rc<Region>, Border, Undirected> = Csr::new();

            let mut region_ids: HashMap<String, u32> = HashMap::new();

            for region in region_.into_iter() {
                let mut db_id = db
                    .query("SELECT VALUE id FROM region WHERE name = $name;")
                    .bind(("name", region.name().to_owned()))
                    .await?;
                let db_id: Option<String> = db_id.take(0)?;
                let db_id: String = db_id.expect("I know it exists");
                let region = Rc::new(region);
                let id = regions.add_node(region);
                region_ids.insert(db_id, id);
            }

            let mut borders = db.query("SELECT * FROM border;").await?;
            #[derive(Deserialize)]
            struct DirectedBorder {
                r#in: String,
                out: String,
                border_type: String,
                strait_region: Option<String>,
            }
            let borders: Vec<DirectedBorder> = borders.take(0)?;
            let borders: Result<Vec<(String, String, Border)>> = borders
                .into_iter()
                .map(
                    |DirectedBorder {
                         r#in,
                         out,
                         border_type,
                         strait_region,
                     }|
                     -> Result<(String, String, Border)> {
                        Ok((
                            r#in,
                            out,
                            deserialize_border(
                                SerializedBorder {
                                    border_type,
                                    strait_region,
                                },
                                &regions,
                            )?,
                        ))
                    },
                )
                .collect();
            let borders = borders?;

            for (from, to, border) in borders {
                let i = region_ids.get(&from).ok_or(eyre!("Invalid region ID"))?;
                let j = region_ids.get(&to).ok_or(eyre!("Invalid region ID"))?;
                regions.add_edge(*i, *j, border);
            }

            // TODO: units

            let mut state = State::new(teams, regions, self.water_stroke, self.land_stroke);

            state.turn = prelude.turn;

            Ok(state)
        };

        rt.block_on(future)
    }

    fn read_from_state(&mut self, state: crate::game::State) -> Result<()> {
        let rt = Runtime::new()?;
        let future = async {
            let db = Surreal::new::<Ws>(self.address.as_str()).await?;
            // Namespace apparently implements clone, but .clone() is not found...
            db.signin(Namespace {
                namespace: self.credentials.namespace.clone(),
                username: self.credentials.username.clone(),
                password: self.credentials.password.clone(),
            })
            .await?;
            // let db = Surreal::new::<Ws>("127.0.0.0:8000").await?;
            //db.signin(Root {
            //    username: "root",
            //    password: "root",
            //})
            //.await?;

            db.use_ns("hadijatek").use_db(&self.name).await?;

            // Clear database
            let _deletion: Response = db
                .query(
                    r#"DELETE prelude RETURN NONE;
                DELETE team RETURN NONE;
                DELETE border RETURN NONE;
                DELETE region RETURN NONE;
                DELETE unit RETURN NONE;"#,
                )
                .await?;

            // Write basic info (prelude)
            let _prelude: Prelude = db
                .create(("prelude", "prelude"))
                .content(Prelude {
                    turn: state.turn,
                    water_stroke: state.water_stroke,
                    land_stroke: state.land_stroke,
                })
                .await?;

            // Create team records
            for team in state.teams() {
                let _team_creation: Team = db.create("team").content(team).await?;
            }

            // Create region records
            let map = state.regions();

            let mut region_ids = HashMap::new();
            for (i, region) in map.node_references() {
                let data: Value = db
                    .create("region")
                    .content(serialize_region(region))
                    // .return("id") // something like this would make my life much easier
                    .await?;
                let id: String = data["id"]["tb"].as_str().expect("No tb field").to_owned()
                    + ":"
                    + data["id"]["id"]["String"]
                        .as_str()
                        .expect("No idString field");
                region_ids.insert(i, id);
            }

            // Create border records
            for (i, _region) in map.node_references() {
                let i_id = region_ids.get(&i).expect("It was just created");

                let neighbor_data = map.neighbors(i).map(|j| {
                    (
                        map.edge_weight(i, j),
                        region_ids.get(&j).expect("It was just created"),
                    )
                });

                for (border, j_id) in neighbor_data {
                    let query = format!("RELATE {}->border->{} CONTENT $border", i_id, j_id);
                    db.query(query)
                        .bind(("border", serialize_border(border)))
                        .await?;
                    // This should work but doesn't
                    /*                     let thing = db
                    .query("RELATE $i_id->border->$j_id CONTENT $border")
                    .bind(("i_id", i_id))
                    .bind(("j_id", j_id))
                    .bind(("border", serialize_border(border)))
                    .await?; */
                }
            }

            // Create unit records
            for _unit in state.units().iter() {
                todo!();
            }

            Ok::<(), eyre::Report>(())
        };

        rt.block_on(future)?;
        Ok(())
    }
}
