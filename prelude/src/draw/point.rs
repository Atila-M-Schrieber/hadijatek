// use std::{error, fmt, num::ParseFloatError, str::FromStr};

use std::fmt::Display;

use serde::{Deserialize, Serialize};
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
        self.0 = y;
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

impl From<Point> for Parameters {
    fn from(point: Point) -> Parameters {
        let (x, y) = point.into();
        vec![x, y].into()
    }
}

// Why the fuck is this necessary. Damn orphan rule.
pub struct MyParameters(Parameters);

impl From<Vec<Point>> for MyParameters {
    fn from(points: Vec<Point>) -> MyParameters {
        let mut vec = Vec::new();
        for Point(x, y) in points {
            vec.push(x);
            vec.push(y);
        }
        MyParameters(vec.into())
    }
}

impl From<MyParameters> for Parameters {
    fn from(why: MyParameters) -> Parameters {
        why.0
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
