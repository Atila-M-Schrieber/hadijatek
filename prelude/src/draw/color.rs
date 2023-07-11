use serde::{Deserialize, Serialize};
use std::{error, fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(u8, u8, u8);
#[derive(Debug)]
pub enum ColorParseError {
    LengthError,
    FormatError,
    ParseIntError,
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ColorParseError::*;
        match self {
            LengthError => write!(f, "Failed to parse color: wrong length"),
            FormatError => write!(f, "Failed to parse color: no '#' sign"),
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
    type Err = ColorParseError;
    /// Must be valid hex color value, preceded by #. #000000 to #ffffff
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 7 {
            return Err(ColorParseError::LengthError);
        } else if &s[0..1] != "#" {
            return Err(ColorParseError::FormatError);
        }

        let r = u8::from_str_radix(&s[1..3], 16).map_err(|_| ColorParseError::ParseIntError)?;
        let g = u8::from_str_radix(&s[3..5], 16).map_err(|_| ColorParseError::ParseIntError)?;
        let b = u8::from_str_radix(&s[5..7], 16).map_err(|_| ColorParseError::ParseIntError)?;

        Ok(Color(r, g, b))
    }
}
