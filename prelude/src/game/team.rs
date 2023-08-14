//! Teams represent the players / groups of players of the game
//!
//! Teams have a name, and a color.

#[cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};

use crate::draw::Color;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", derive(Serialize, Deserialize))]
pub struct Team {
    name: String,
    color: Color,
}

impl Team {
    pub fn new(name: String, color: Color) -> Self {
        Team { name, color }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn color(&self) -> Color {
        self.color
    }
}
