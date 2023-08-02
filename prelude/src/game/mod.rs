//! Structure of the game.
//!
//! Contains all types needed for the implementation of game logic.

use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

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

/// The State struct defines the current state of the game. Most importantly, it contains the
/// (sparse) graph of the regions on the map.
pub struct State {
    teams: Vec<Rc<Team>>,
    regions: Csr<Rc<Region>, Border, Undirected>,
    units: RefCell<Vec<Unit>>,
    // orders?
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

    pub fn units(&self) -> Ref<Vec<Unit>> {
        self.units.borrow()
    }

    pub fn regions(&self) -> &Csr<Rc<Region>, Border, Undirected> {
        &self.regions
    }
}
