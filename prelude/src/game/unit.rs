use super::region::Region;
use super::team::Team;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitType {
    Tank,
    Ship,
    Plane,
    Supertank,
    Submarine,
    Artillery,
}

#[derive(Debug, Clone)]
pub struct Unit {
    unit_type: UnitType,
    region: Rc<Region>,
    owner: Rc<Team>,
}
