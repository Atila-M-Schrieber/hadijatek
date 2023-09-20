//! 2D f32 tuple point.

use geo::Coord;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{Add, Mul, Neg, Sub},
};
use svg::node::element::path::Parameters;

use super::Contains;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point(f32, f32);

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point(x, y)
    }

    pub fn from((x, y): (f32, f32)) -> Self {
        Point(x, y)
    }

    pub fn get(self) -> (f32, f32) {
        (self.0, self.1)
    }

    pub fn square(self) -> f32 {
        let Point(x, y) = self;
        x * x + y * y
    }

    pub fn to_polar(self) -> (f32, f32) {
        let angle = self.1.atan2(self.0);
        let radius = self.square().sqrt();
        (radius, angle)
    }

    pub fn from_polar(rad: f32, ang: f32) -> Self {
        let x = rad * ang.cos();
        let y = rad * ang.sin();
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
        self.1 = y;
    }

    pub fn move_rel(&mut self, x: f32, y: f32) {
        self.0 += x;
        self.1 += y;
    }
}

impl Add for Point {
    type Output = Self;
    fn add(self, Point(x2, y2): Point) -> Self::Output {
        let Point(x1, y1) = self;
        Point(x1 + x2, y1 + y2)
    }
}

impl Add<(f32, f32)> for Point {
    type Output = Self;
    fn add(self, (x2, y2): (f32, f32)) -> Self::Output {
        let Point(x1, y1) = self;
        Point(x1 + x2, y1 + y2)
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, Point(x2, y2): Self) -> Self::Output {
        let Point(x1, y1) = self;
        Point(x1 - x2, y1 - y2)
    }
}

impl Neg for Point {
    type Output = Self;
    fn neg(self) -> Self::Output {
        let Point(x1, y1) = self;
        Point(-x1, -y1)
    }
}

impl Mul<f32> for Point {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        let Point(x1, y1) = self;
        Point(x1 * rhs, y1 * rhs)
    }
}

impl Mul<Point> for f32 {
    type Output = Point;
    fn mul(self, Point(x, y): Point) -> Self::Output {
        Point(x * self, y * self)
    }
}

// Dot product
impl Mul<Point> for Point {
    type Output = f32;
    fn mul(self, rhs: Point) -> Self::Output {
        let (x1, y1) = self.get();
        let (x2, y2) = rhs.get();
        x1 * x2 + y1 * y2
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

pub trait Distance<T> {
    type DistanceType;
    fn distance(&self, dist_to: &T) -> Self::DistanceType;
}

impl Distance<Point> for Point {
    type DistanceType = f32;
    fn distance(&self, dist_to: &Point) -> Self::DistanceType {
        (*dist_to - *self).square().sqrt()
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
