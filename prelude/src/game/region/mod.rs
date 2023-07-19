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

#[derive(Debug, Clone)]
pub enum Border {
    Land,
    Shore,
    Strait(Rc<Region>), // will simplyfy orders involving straits
    Sea,
}

#[derive(Debug, Clone)]
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
    pub fn owner(&self) -> Option<Rc<Team>> {
        self.owner.as_ref().map(Rc::clone)
    }
}

impl Default for Base {
    fn default() -> Self {
        Self::new()
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn region_type(&self) -> RegionType {
        self.region_type
    }

    pub fn has_base(&self) -> bool {
        self.base.is_some()
    }

    pub fn owner(&self) -> Option<Rc<Team>> {
        self.base
            .as_ref()
            .and_then(|r_base| r_base.borrow().owner())
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn color(&self) -> Color {
        self.color
    }
}
