use crate::State;

mod read;
mod write;

pub trait Database {
    fn read_state(&self) -> State;
    fn write_state(&mut self, state: State);
}
