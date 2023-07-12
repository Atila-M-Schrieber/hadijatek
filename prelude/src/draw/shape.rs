use serde::{Deserialize, Serialize};
use std::{error, fmt, str::FromStr};
use svg::node::element::path::{Command, Data, Position};

use super::Point;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape(Vec<Point>);

#[derive(Debug)]
pub enum ShapeFromDataError {
    FirstNotMoveError,
    LastNotCloseError,
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
            for (i, &param) in params.to_vec().iter().enumerate() {
                use Position::*;
                match *move_type {
                    Absolute => pos.move_abs_x(param),
                    Relative => pos.move_rel(param, 0.),
                }
                vec.push(*pos);
            }
        }
        VerticalLine(move_type, params) => {
            for (i, &param) in params.to_vec().iter().enumerate() {
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
                    if let (Some(_), Some(_)) = (params.get(0), params.get(1)) {
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
            }
            let _ = push_command(command, &mut pos, &mut vec);
        }
        Ok(Shape(vec))
    }
}
