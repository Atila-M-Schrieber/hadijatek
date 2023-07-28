//! Legacy .hmap files

use std::fs;
use std::rc::Rc;

use crate::game::region::{Region, RegionType};
use crate::game::unit::Unit;
use crate::lang;
use crate::{draw::Color, game::team::Team, game::State};

use super::Database;

use eyre::Result;
use itertools::Itertools;
use petgraph::visit::IntoNodeReferences;

pub struct Legacy {
    name: String,
    turn: usize,
    water_stroke: Color,
    land_stroke: Color,
    content: String,
}

impl Legacy {
    pub fn new(name: String, water_stroke: Color, land_stroke: Color) -> Legacy {
        Legacy {
            name,
            turn: 0,
            water_stroke,
            land_stroke,
            content: String::new(),
        }
    }
}

impl Database for Legacy {
    fn read_from_state(&mut self, state: State) -> Result<()> {
        let prelude = format!(
            "{}: {}\n{}: {}\n{}: {}\n{}: {}",
            lang!["Név", "Name"],
            self.name,
            lang!["Lépések", "Steps"],
            self.turn,
            lang!["Tengeri mezők körvonalának Színe", "Water Stroke Color"],
            self.water_stroke,
            lang!["Szárazföld körvonal Szín", "Land Stroke Color: "],
            self.land_stroke,
        );

        let team_to_entry = |(i, team): (usize, &Rc<Team>)| {
            let bases = state
                .regions()
                .node_references()
                .filter(|(_, region)| (region.owner() == Some(Rc::clone(team))))
                .map(|(i, region)| {
                    let tf = if region.color() == team.color() {
                        "True"
                    } else {
                        "False"
                    };
                    format!("({},{})", i, /* region.name(), */ tf)
                })
                .join(",");
            format!("{i},\"{}\",\"{}\",[{}]", team.name(), team.color(), bases)
        };

        let unit_to_entry = |(_i, _unit): (usize, &Unit)| -> String { todo!() };

        let region_to_entry = |(i, region): (u32, &Rc<Region>)| {
            use RegionType::*;
            let tp = match region.region_type() {
                Sea => 0,
                _ => 1 + region.has_base() as u32,
            };
            format!(
                "{i},{},\"{}\",\"{}\",{},{}",
                tp,
                region.name(),
                region.color(),
                region.shape().points().first().unwrap(),
                region.shape()
            )
        };

        // TODO add units

        self.content = prelude
            + "\n---\n"
            + &state
                .teams()
                .iter()
                .sorted_by_key(|t| t.name())
                .enumerate()
                .map(team_to_entry)
                .join("\n")
            + "\n---\n"
            + &state
                .units()
                .borrow()
                .iter()
                .enumerate()
                .map(unit_to_entry)
                .join("\n")
            + "---\n"
            + &state
                .regions()
                .node_references()
                .map(region_to_entry)
                .join("\n");

        Ok(())
    }

    fn to_state(&self) -> Result<State> {
        todo!()
    }

    fn write(&self) -> Result<()> {
        let filename = format!("{}_{}.hmap", self.name, self.turn);

        fs::write(filename, &self.content)?;
        Ok(())
    }

    fn load(&self) -> Result<()> {
        todo!()
    }
}
