use crate::State;

pub mod read;
pub mod write;

pub trait Database {
    fn read_state(&self) -> State;
    fn write_state(&mut self, state: State);
}
