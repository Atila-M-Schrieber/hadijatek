use serde::{Deserialize, Serialize};

use crate::draw::Color;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
