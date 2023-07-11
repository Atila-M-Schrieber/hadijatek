use serde::{Deserialize, Serialize};

use crate::draw::Color;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Team {
    name: String,
    color: Color,
}
