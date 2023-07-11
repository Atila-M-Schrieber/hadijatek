use serde::{Deserialize, Serialize};
use std::{error, fmt, str::FromStr};

mod color;

pub use color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point(f32, f32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape(Vec<Point>);
