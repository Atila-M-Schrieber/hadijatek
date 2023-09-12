//! Simple RGB color which can be read from hex strings

use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::{error, fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color(u8, u8, u8);
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorHSL(f32, f32, f32);

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color(r, g, b)
    }

    pub fn from((r, g, b): (u8, u8, u8)) -> Self {
        Color(r, g, b)
    }

    pub fn get(&self) -> (u8, u8, u8) {
        (self.0, self.1, self.2)
    }

    pub fn black() -> Self {
        Color(0, 0, 0)
    }

    pub fn white() -> Self {
        Color(255, 255, 255)
    }

    pub fn highlight(&self, amount: f32) -> Self {
        ColorHSL::from(*self).highlight(amount).into()
    }
}

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let r = r / 255.0;
    let g = g / 255.0;
    let b = b / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));

    let mut h = 0.0;
    let mut s = 0.0;
    let l = (max + min) / 2.0;

    if max != min {
        let d = max - min;

        s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };

        h = if max == r {
            (g - b) / d + (if g < b { 6.0 } else { 0.0 })
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };

        h /= 6.0;
    }

    (h, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let r;
    let g;
    let b;

    if s == 0.0 {
        r = l;
        g = l;
        b = l;
    } else {
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };

        let p = 2.0 * l - q;

        r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        g = hue_to_rgb(p, q, h);
        b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    }

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

impl From<Color> for ColorHSL {
    fn from(color: Color) -> Self {
        let Color(r, g, b) = color;
        let (h, s, l) = rgb_to_hsl(r as f32, g as f32, b as f32);
        ColorHSL(h, s, l)
    }
}

impl From<ColorHSL> for Color {
    fn from(hsl: ColorHSL) -> Self {
        let ColorHSL(h, s, l) = hsl;
        let (r, g, b) = hsl_to_rgb(h, s, l);
        Color(r, g, b)
    }
}

impl ColorHSL {
    pub fn highlight(&self, amount: f32) -> Self {
        let ColorHSL(h, s, l) = *self;
        // let diff = 1. - l;
        // this brightens relative to current brightness, and darkens if its too bright
        /*let mut new_l = if diff < amount {
            l - amount
        } else if diff < amount * 2. {
            l + amount / 2.
        } else {
            l + amount
        };*/
        let new_l = if l < 0.5 { l + amount } else { l - amount };
        // new_l = new_l.min(1.0).max(0.0);
        // let new_l = (l + amount).min(1.0).max(0.0); // Clamp to [0, 1]
        ColorHSL(h, s, new_l)
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
    type Err = Error;
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
