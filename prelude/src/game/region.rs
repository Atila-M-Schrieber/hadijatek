use std::{cell::RefCell, rc::Rc};

use super::team::Team;
use crate::draw::{Color, Shape};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegionType {
    Land,
    Shore,
    Strait,
    Sea,
}

pub type Border = RegionType;

#[derive(Debug)]
pub struct Base {
    owner: Option<Rc<Team>>,
}

#[derive(Debug)]
pub struct Region {
    name: String,
    region_type: RegionType,
    base: Option<RefCell<Base>>,
    shape: Shape,
    color: Color,
}
