use std::fs;
use std::rc::Rc;

use crate::lang;
use crate::{draw::Color, lang::Language, lang::LANGUAGE, team::Team, State};

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
                .fold("".to_owned(), |p_str, (i, region)| {
                    let tf = if region.color() == team.color() {
                        "True"
                    } else {
                        "False"
                    };
                    p_str + format!(",({i},{})", tf).as_str()
                });
            format!("{i},\"{}\",\"{}\",[{}]", team.name(), team.color(), bases)
        };

        // TODO add units & regions

        self.content = prelude
            + "\n---\n"
            + &state
                .teams()
                .iter()
                .sorted_by_key(|t| t.name())
                .enumerate()
                .map(team_to_entry)
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
