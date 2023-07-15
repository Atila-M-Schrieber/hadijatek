use std::{cell::RefCell, rc::Rc};

use eyre::Result;
use petgraph::{csr::Csr, Undirected};
use prelude::{
    draw::{Color, Shape},
    region::{Base, Border, Region},
};

pub fn graphify(
    pre_regions: Vec<(String, Option<RefCell<Base>>, Shape, Color)>,
    water_color: Color,
) -> Result<Csr<Rc<Region>, Border, Undirected>> {
    todo!()
}
