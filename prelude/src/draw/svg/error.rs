use std::{error, fmt};

use crate::draw::Color;

#[derive(Debug)]
pub enum ReadEventError {
    NoName,
    NoStyle,
    NoFill,
    NoStroke,
    BadLastBit,
    BadFirstBit,
    TeamColorOnNonBaseRegion(Color),
    NoPath,
    NoDAttribute,
}

impl fmt::Display for ReadEventError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ReadEventError::*;
        match self {
            NoName => write!(
                f,
                "Region's name not found: 'inkscape:label' attribute expected."
            ),
            NoStyle => write!(f, "No style attribute found"),
            NoFill => write!(f, "No 'fill' property found"),
            NoStroke => write!(f, "No 'stroke' property found"),
            BadLastBit => write!(f, "A property's value could not be processed"),
            BadFirstBit => write!(f, "A property's key could not be processed"),
            TeamColorOnNonBaseRegion(color) => write!(f, "Team color {color} found on a region without a base. Only based regions can fly the team color."),
            NoPath => write!(f, "This SVG element is not a Path"),
            NoDAttribute => write!(f, "Can't find the d attribute in this path elment"),
        }
    }
}

impl error::Error for ReadEventError {}
