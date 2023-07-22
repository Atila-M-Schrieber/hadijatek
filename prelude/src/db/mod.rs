use eyre::Result;

use crate::State;

// pub mod read;
pub mod legacy;

pub trait Database {
    fn load(&self) -> Result<()>;
    fn write(&self) -> Result<()>;
    fn to_state(&self) -> Result<State>;
    fn read_from_state(&mut self, state: State) -> Result<()>;
}
