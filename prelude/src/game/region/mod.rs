use std::{cell::RefCell, rc::Rc};

use super::team::Team;
use crate::draw::{Color, Shape};
use errors::RegionCreationError;
use serde::{Deserialize, Serialize};

mod errors;

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

impl Base {
    pub fn new() -> Self {
        Base { owner: None }
    }
    pub fn set(&mut self, team: Rc<Team>) {
        self.owner = Some(Rc::clone(&team))
    }
}

#[derive(Debug)]
pub struct Region {
    name: String,
    region_type: RegionType,
    base: Option<RefCell<Base>>,
    shape: Shape,
    color: Color,
}

impl Region {
    pub fn new(
        name: String,
        region_type: RegionType,
        base: Option<RefCell<Base>>,
        shape: Shape,
        color: Color,
    ) -> Result<Self, RegionCreationError> {
        use RegionCreationError::*;
        use RegionType::*;
        if base.is_some() && region_type == Sea {
            return Err(BaseOnSea);
        }
        Ok(Region {
            name,
            region_type,
            base,
            shape,
            color,
        })
    }
}
