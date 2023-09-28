//! Vec of Points, with many helper methods and trait implementations

use anyhow::Result;
use geo::{LineString, Polygon};
use polylabel::polylabel;
use serde::{Deserialize, Serialize};
use std::{
    error,
    fmt::{self, Display},
};
use svg::node::element::path::{Command, Data, Parameters, Position};

use super::{point::Distance, Point};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape(Vec<Point>);

impl Shape {
    pub fn points(&self) -> &[Point] {
        &self.0
    }

    pub fn new(points: &[Point]) -> Self {
        Shape(points.to_owned())
    }

    // needs optimization for size, fine for speed
    pub fn to_data_string(self) -> String {
        self.0
            .into_iter()
            .enumerate()
            .map(|(n, p)| {
                let (x, y) = p.get();
                let c = match n {
                    0 => "M",
                    1 => "L",
                    _ => "",
                };
                format!("{c} {x},{y}")
            })
            .collect::<Vec<String>>()
            .join(" ")
            + " Z"
    }

    pub fn pole(&self) -> Result<Point> {
        let poly: Polygon = self.clone().into();
        let point: geo::Coord = polylabel(&poly, &0.01)?.into();
        Ok(point.into())
    }

    pub fn intersects_x(&self, y: f32) -> Vec<f32> {
        let mut xs = Vec::new();
        let points = self.points();
        let len = points.len();

        for i in 0..len {
            let j = (i + 1) % len;
            let (x1, y1) = points[i].get();
            let (x2, y2) = points[j].get();

            // Check if segment crosses the line y
            if (y1 <= y && y < y2) || (y2 <= y && y < y1) {
                let x = x1 + (x2 - x1) * (y - y1) / (y2 - y1);
                xs.push(x);
            }
        }

        xs
    }

    pub fn intersects_y(&self, x: f32) -> Vec<f32> {
        let mut ys = Vec::new();
        let points = self.points();
        let len = points.len();

        for i in 0..len {
            let j = (i + 1) % len;
            let (x1, y1) = points[i].get();
            let (x2, y2) = points[j].get();

            // Check if segment crosses the line x
            if (x1 <= x && x < x2) || (x2 <= x && x < x1) {
                let y = y1 + (y2 - y1) * (x - x1) / (x2 - x1);
                ys.push(y);
            }
        }

        ys
    }

    pub fn area_signed(&self) -> f32 {
        let len = self.0.len();
        let mut area = 0.;
        for i in 0..len {
            let (x0, y0) = self.0[i].get();
            let (x1, y1) = self.0[(i + 1) % len].get();

            area += x0 * y1 - x1 * y0;
        }
        area
    }

    pub fn area(&self) -> f32 {
        self.area_signed().abs()
    }

    pub fn centroid(&self) -> Point {
        let len = self.0.len();
        let (xs, ys): (Vec<_>, Vec<_>) = self.0.iter().map(Point::get_ref).unzip();
        Point::new(
            xs.into_iter().sum::<f32>() / len as f32,
            ys.into_iter().sum::<f32>() / len as f32,
        )
    }
}

/// Trait for types that may "physically" contain (other) types
pub trait Contains<T> {
    fn contains(&self, internal: &T) -> bool;
}

/// Shapes may contain points
impl Contains<&Point> for &Shape {
    fn contains(&self, internal: &&Point) -> bool {
        // logic
        let points = self.points();
        let len = points.len();
        let (x3, y3) = internal.get();

        let mut contains = false;
        for i in 0..len {
            let j = (i + 1) % len;
            let (x1, y1) = points[i].get();
            let (x2, y2) = points[j].get();
            // let x4 = f32::min(x1, f32::min(x2, x3));
            let x4 = 0.;
            let y4 = y3;
            let divisor = (x1 - x2) * (y3 - y4) - (x3 - x4) * (y1 - y2);
            let t = ((x1 - x3) * (y3 - y4) - (x3 - x4) * (y1 - y3)) / divisor;
            let u = ((x1 - x3) * (y1 - y2) - (x1 - x2) * (y1 - y3)) / divisor;
            let crosses = (0. ..=1.).contains(&u) && (0. ..=1.).contains(&t);
            // if crosses, flip the value of contains (XOR)
            contains ^= crosses
        }
        contains
    }
}

/// Shapes may contain other shapes
impl Contains<&Shape> for &Shape {
    fn contains(&self, internal: &&Shape) -> bool {
        internal.points().iter().all(|p| self.contains(&p))
    }
}

impl Distance<Point> for Shape {
    type DistanceType = Point;
    fn distance(&self, dist_to: &Point) -> Self::DistanceType {
        let len = self.0.len();

        let mut min_dist_sq = f32::MAX;

        let mut closest_point = Point::new(0., 0.);

        for i in 0..len {
            let j = (i + 1) % len;
            let p1 = self.0[i];
            let p2 = self.0[j];

            let segment = p2 - p1;
            let vec_to_point = *dist_to - p1;

            // closest point along segment's dist from p1 to p2 as lerp
            let t = (segment * vec_to_point / segment.square()).min(1.).max(0.);

            let projected_point = p1 + segment * t;
            let dist_sq = (projected_point - *dist_to).square();

            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                closest_point = projected_point;
            }
        }

        if self.contains(&dist_to) {
            *dist_to - closest_point
        } else {
            closest_point - *dist_to
        }
    }
}

impl Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"")?;
        for point in self.points() {
            write!(f, "M {},{} ", point.get().0, point.get().1)?;
        }
        write!(f, "Z\"")
    }
}

fn push_command(
    command: &Command,
    pos: &mut Point,
    vec: &mut Vec<Point>,
) -> Result<(), ShapeFromDataError> {
    use Command::*;
    use ShapeFromDataError::*;
    match command {
        Move(move_type, params) | Line(move_type, params) => {
            if params.len() % 2 != 0 {
                return Err(OddParamsErr(command.clone()));
            }
            let mut temp_x = 0.;
            for (i, &param) in params.to_vec().iter().enumerate() {
                let even = i % 2 == 0;
                // if x coordinate, assign to temp_x, else move point & add to vec
                if even {
                    temp_x = param;
                } else {
                    use Position::*;
                    match *move_type {
                        Absolute => {
                            pos.move_abs(temp_x, param);
                        }
                        Relative => {
                            pos.move_rel(temp_x, param);
                        }
                    }
                    vec.push(*pos);
                }
            }
        }
        HorizontalLine(move_type, params) => {
            for (_, &param) in params.to_vec().iter().enumerate() {
                use Position::*;
                match *move_type {
                    Absolute => pos.move_abs_x(param),
                    Relative => pos.move_rel(param, 0.),
                }
                vec.push(*pos);
            }
        }
        VerticalLine(move_type, params) => {
            for (_, &param) in params.to_vec().iter().enumerate() {
                use Position::*;
                match *move_type {
                    Absolute => pos.move_abs_y(param),
                    Relative => pos.move_rel(0., param),
                }
                vec.push(*pos);
            }
        }
        Close => return Err(EarlyCloseError),
        cmd => return Err(UnsupportedCommandError(cmd.clone())),
    }
    Ok(())
}

impl TryFrom<Data> for Shape {
    type Error = ShapeFromDataError;
    fn try_from(data: Data) -> Result<Self, Self::Error> {
        use ShapeFromDataError::*;
        let mut pos = Point::new(0., 0.);
        let mut vec = Vec::new();
        for (i, command) in data.iter().enumerate() {
            use Command::*;
            // Ensures that first command is a move
            if i == 0 {
                if let Move(_, params) = command {
                    if let (Some(_), Some(_)) = (params.first(), params.get(1)) {
                    } else {
                        return Err(NoFirstParamsError);
                    }
                } else {
                    return Err(FirstNotMoveError);
                }
            // Makes sure last command is a close, and breaks early
            } else if i == data.len() {
                // bruh no Eq impl
                if let Close = command {
                    break;
                } else {
                    return Err(LastNotCloseError);
                }
            } else if let Move(_, _) = command {
                return Err(NonFirstMoveError(i));
            }
            let _ = push_command(command, &mut pos, &mut vec);
        }
        Ok(Shape(vec))
    }
}

impl From<Shape> for Vec<Point> {
    fn from(shape: Shape) -> Vec<Point> {
        shape.0
    }
}

impl From<Shape> for LineString {
    fn from(shape: Shape) -> Self {
        let mut points = shape.0;
        let first = points[0];
        points.push(first);
        points.into()
    }
}

impl From<Shape> for Polygon {
    fn from(shape: Shape) -> Self {
        Polygon::new(shape.into(), Vec::new())
    }
}

impl From<Shape> for Parameters {
    fn from(shape: Shape) -> Self {
        let mut vec = Vec::new();
        for (x, y) in shape.points().iter().map(|p| p.get()) {
            vec.push(x);
            vec.push(y);
        }
        vec.into()
    }
}

impl From<Shape> for Data {
    /// Subject to future optimization, currently is always just one big move (absolute) command
    fn from(shape: Shape) -> Data {
        let mut data = Data::new();
        data = data.move_to::<Parameters>(shape.into());
        data.close()
    }
}

#[derive(Debug)]
pub enum ShapeFromDataError {
    FirstNotMoveError,
    LastNotCloseError,
    NonFirstMoveError(usize),
    NoFirstParamsError,
    EarlyCloseError,
    UnsupportedCommandError(Command),
    OddParamsErr(Command),
    Impossible,
}

impl fmt::Display for ShapeFromDataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ShapeFromDataError::*;
        match self {
            FirstNotMoveError => write!(f, "First path command is not a move"),
            LastNotCloseError => write!(f, "Last path command is not a close"),
            NonFirstMoveError(pos) => write!(
                f,
                "The {pos}th command is a move, but only the first command can be a move"
            ),
            NoFirstParamsError => write!(f, "The first parameter(s) of a command can't be found"),
            EarlyCloseError => write!(f, "There is a Close command too early in the SVG path"),
            UnsupportedCommandError(cmd) => write!(f, "This command is unsupported: {:?}", cmd),
            OddParamsErr(cmd) => write!(
                f,
                "This command type needs an even number of parameters: {:?}",
                cmd
            ),
            Impossible => write!(f, "How???"),
        }
    }
}

impl error::Error for ShapeFromDataError {}
