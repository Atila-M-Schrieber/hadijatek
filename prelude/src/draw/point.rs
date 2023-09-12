//! 2D f32 tuple point.

use geo::Coord;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use svg::node::element::path::Parameters;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point(f32, f32);

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point(x, y)
    }

    pub fn get(self) -> (f32, f32) {
        (self.0, self.1)
    }

    pub fn move_abs(&mut self, x: f32, y: f32) {
        self.0 = x;
        self.1 = y;
    }

    pub fn move_abs_x(&mut self, x: f32) {
        self.0 = x;
    }

    pub fn move_abs_y(&mut self, y: f32) {
        self.1 = y;
    }

    pub fn move_rel(&mut self, x: f32, y: f32) {
        self.0 += x;
        self.1 += y;
    }
}

impl From<Point> for (f32, f32) {
    fn from(point: Point) -> (f32, f32) {
        (point.0, point.1)
    }
}

impl From<Point> for Coord {
    fn from(point: Point) -> Self {
        Coord {
            x: point.0 as f64,
            y: point.1 as f64,
        }
    }
}

impl From<Coord> for Point {
    fn from(coord: Coord) -> Self {
        Point::new(coord.x as f32, coord.y as f32)
    }
}

impl From<Point> for Parameters {
    fn from(point: Point) -> Parameters {
        let (x, y) = point.into();
        vec![x, y].into()
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0, self.1)
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
