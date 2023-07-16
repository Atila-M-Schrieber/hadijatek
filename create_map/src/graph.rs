use std::{cell::RefCell, rc::Rc};

use eyre::Result;
use petgraph::{csr::Csr, Undirected};
use prelude::{
    draw::{Color, Shape},
    region::{Base, Border, Region},
};

use crate::read::PreRegion;

pub fn graphify(
    pre_regions: Vec<PreRegion>,
    water_color: Color,
) -> Result<Csr<Rc<Region>, Border, Undirected>> {
    todo!()
}
