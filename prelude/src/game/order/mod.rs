use super::{team::Team, unit::Unit, State};
use std::{fmt::Debug, rc::Rc};

mod attack;
mod bombard;
mod defend;
mod kill;
mod stay;
mod summon;
mod support;
mod transform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderStatus {
    Failed,
    Unresolved,
    Succeeded,
}

#[derive(Debug)]
struct Order {
    status: OrderStatus,
    team: Team,
    unit: Rc<Unit>,
    order: Box<dyn Orderable>,
}

trait Orderable: Debug {
    fn legal(&self, state: State) -> bool;
}
