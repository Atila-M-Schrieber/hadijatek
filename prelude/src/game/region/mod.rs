//! Regions, their types, and the borders between them
//!
//! # Region types:
//! * Land - occupiable only by land units - can't border sea regions
//! * Sea - occupiable only by sea units - can't border land regions, can't have bases
//! * Shore - occupiable by land and sea units - can border land and sea regions
//!     * Strait - as above, but any (sea) units moving through the strait act as if they were
//!     traversing through the strait as well (units defending on the stait may block passage)
//!
//! # Border types:
//! * Land - travesible only by land units - connects a Land and a Land, Shore, or Strait region
//! * Sea - traversible only by sea units - connects a Sea and a Sea, Shore, or Strait region
//! * Shore - traversible by both land and sea units - connects Shore or Strait regions, but only
//! if the two shores share a [tripoint](https://en.wikipedia.org/wiki/Tripoint) with the same Sea
//! region.
//! * Strait - traversible by sea units - connects two Sea regions through a strait (Contains an Rc
//! of the Strait region for convenience.)
//!
//! # Bases
//!
//! Regions (except Sea) may contain bases, which are the sources of units for teams, the team
//! which owns the most at the end of the game will be the victor. Bases may be unowned, or owned
//! by a Team.

use std::{
    cell::{Ref, RefCell},
    fmt::Display,
    rc::Rc,
};

use super::team::Team;
use crate::draw::{Color, Point, Shape};
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
    pole: Point,
    color: Color,
}

impl Region {
    pub fn new(
        name: String,
        region_type: RegionType,
        base: Option<RefCell<Base>>,
        shape: Shape,
        pole: Point,
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
            pole,
            color,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn region_type(&self) -> RegionType {
        self.region_type
    }

    pub fn base(&self) -> Option<Ref<Base>> {
        self.base.as_ref().map(|base| base.borrow())
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

    pub fn pole(&self) -> Point {
        self.pole
    }

    pub fn color(&self) -> Color {
        self.color
    }
}

impl Display for RegionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for Border {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Border::Strait(strait) => write!(f, "Strait ({})", strait.name),
            _ => write!(f, "{:?}", self),
        }
    }
}
