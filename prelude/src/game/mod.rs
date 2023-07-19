use std::{cell::RefCell, rc::Rc};

use self::{
    region::{Border, Region},
    team::Team,
    unit::Unit,
};
use petgraph::{csr::Csr, Undirected};

// pub mod order;
pub mod region;
pub mod team;
pub mod unit;

pub struct State {
    teams: Vec<Rc<Team>>,
    regions: Csr<Rc<Region>, Border, Undirected>,
    units: RefCell<Vec<Unit>>,
}

impl State {
    /// All game states at creation will have no units
    pub fn new(teams: Vec<Rc<Team>>, regions: Csr<Rc<Region>, Border, Undirected>) -> Self {
        State {
            teams,
            regions,
            units: RefCell::new(Vec::new()),
        }
    }

    pub fn teams(&self) -> &[Rc<Team>] {
        &self.teams
    }

    pub fn units(&self) -> &RefCell<Vec<Unit>> {
        &self.units
    }

    pub fn regions(&self) -> &Csr<Rc<Region>, Border, Undirected> {
        &self.regions
    }
}
