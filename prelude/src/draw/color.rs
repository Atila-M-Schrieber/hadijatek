//! Simple RGB color which can be read from hex strings

use eyre::Report;
use serde::{Deserialize, Serialize};
use std::{error, fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(u8, u8, u8);

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color(r, g, b)
    }

    pub fn black() -> Self {
        Color(0, 0, 0)
    }

    pub fn white() -> Self {
        Color(255, 255, 255)
    }
}

#[derive(Debug)]
pub enum ColorParseError {
    BadLength(usize),
    BadFormat,
    ParseIntError,
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ColorParseError::*;
        match self {
            BadLength(l) => write!(f, "Failed to parse color: wrong length ({l})"),
            BadFormat => write!(f, "Failed to parse color: no '#' sign"),
            ParseIntError => write!(f, "Failed to parse color to integers"),
        }
    }
}

impl error::Error for ColorParseError {}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

impl FromStr for Color {
    type Err = Report;
    /// Must be valid hex color value, preceded by #. #000000 to #ffffff
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let l = s.len();

        use ColorParseError::*;
        if l == 7 && &s[0..1] != "#" {
            return Err(BadFormat.into());
        } else if !(l == 6 || l == 7) {
            return Err(BadLength(l).into());
        }

        let r = u8::from_str_radix(&s[l - 6..l - 4], 16).map_err(|_| ParseIntError)?;
        let g = u8::from_str_radix(&s[l - 4..l - 2], 16).map_err(|_| ParseIntError)?;
        let b = u8::from_str_radix(&s[l - 2..l], 16).map_err(|_| ParseIntError)?;

        Ok(Color(r, g, b))
    }
}
