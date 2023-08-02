//! Database trait for providing a "frontend" for working with databases.

use eyre::Result;
use serde::{Deserialize, Serialize};

use crate::{draw::Color, game::State};

// pub mod read;
pub mod legacy;
pub mod surrealdb;

#[derive(Debug, Serialize, Deserialize)]
pub struct Prelude {
    pub turn: usize,
    pub water_stroke: Color,
    pub land_stroke: Color,
}

pub trait Database {
    fn load(&self) -> Result<()>;
    fn write(&self) -> Result<()>;
    fn to_state(&self) -> Result<(State, Prelude)>;
    fn read_from_state(&mut self, state: State) -> Result<()>;
}
