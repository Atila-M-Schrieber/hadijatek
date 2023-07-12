// use std::{error, fmt, num::ParseFloatError, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point(f32, f32);

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point(x, y)
    }

    pub fn move_abs(&mut self, x: f32, y: f32) {
        self.0 = x;
        self.1 = y;
    }

    pub fn move_abs_x(&mut self, x: f32) {
        self.0 = x;
    }

    pub fn move_abs_y(&mut self, y: f32) {
        self.0 = y;
    }

    pub fn move_rel(&mut self, x: f32, y: f32) {
        self.0 += x;
        self.1 += y;
    }
}

/* #[derive(Debug)]
pub enum ParsePointError {
    ParseFloatError,
    FormatError,
}

impl fmt::Display for ParsePointError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParsePointError::*;
        match self {
            ParseFloatError => write!(f, "Failed to parse float"),
            FormatError => write!(f, "Incorrect format: Point string is of format 'x y'"),
        }
    }
}

impl error::Error for ParsePointError {}

impl FromStr for Point {
    type Err = ParseFloatError;
    /// Must be SVG-like representation
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        "682.19941".parse()
    }
} */
