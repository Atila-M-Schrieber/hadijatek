use std::collections::HashMap;
use std::rc::Rc;

use eyre::Result;
use petgraph::visit::{IntoNeighbors, IntoNodeReferences};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::{Namespace, Root};
use surrealdb::Response;
use surrealdb::Surreal;
use tokio::runtime::Runtime;

use crate::draw::{Color, Shape};
use crate::game::region::{Base, Border, Region, RegionType};
use crate::game::team::Team;

use super::{Database, Prelude};

pub struct Surrealdb<'a> {
    name: String,
    turn: usize,
    water_stroke: Color,
    land_stroke: Color,
    credentials: Namespace<'a>,
    db: Surreal<Client>,
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
        let rt = Runtime::new().expect("Runtime creation failed");
        let db = rt.block_on(async {
            Surreal::new::<Ws>(ip_address)
                .await
                .expect("Could not connect to database")
        });
        Surrealdb {
            name: name.to_owned(),
            turn: 0,
            water_stroke,
            land_stroke,
            credentials: Namespace {
                namespace: name,
                username,
                password,
            },
            db,
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

#[derive(Debug, Serialize, Deserialize)]
struct SerializedRegion<'a> {
    name: &'a str,
    region_type: RegionType,
    base: Option<SerializedBase>,
    shape: Shape,
    color: Color,
}

fn serialize_region(region: &Rc<Region>) -> SerializedRegion {
    SerializedRegion {
        name: region.name(),
        region_type: region.region_type(),
        base: region.base().as_ref().map(|base| serialize_base(base)),
        shape: region.shape().clone(),
        color: region.color(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum SerializedBorder<'a> {
    Land,
    Shore,
    Strait(&'a str),
    Sea,
}

fn serialize_border(border: &Border) -> SerializedBorder {
    use Border as B;
    use SerializedBorder as SB;
    match border {
        B::Sea => SB::Sea,
        B::Land => SB::Land,
        B::Shore => SB::Shore,
        B::Strait(region) => SB::Strait(region.name()),
    }
}

impl Database for Surrealdb<'_> {
    fn load(&self) -> eyre::Result<()> {
        todo!()
    }

    fn write(&self) -> eyre::Result<()> {
        Ok(())
    }

    fn to_state(&self) -> eyre::Result<(crate::game::State, super::Prelude)> {
        todo!()
    }

    fn read_from_state(&mut self, state: crate::game::State) -> Result<()> {
        let db = &self.db;
        let rt = Runtime::new()?;
        let future = async {
            /*             db.signin(Namespace {
                namespace: self.credentials.namespace.clone(),
                username: self.credentials.username.clone(),
                password: self.credentials.password.clone(),
            })
            .await?; */
            let db = Surreal::new::<Ws>("127.0.0.0:8000").await?;
            db.signin(Root {
                username: "root",
                password: "root",
            })
            .await?;

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
                    turn: self.turn,
                    water_stroke: self.water_stroke,
                    land_stroke: self.land_stroke,
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
