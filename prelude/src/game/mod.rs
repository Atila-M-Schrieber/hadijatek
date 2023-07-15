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
