use std::{error, fmt};

#[derive(Debug)]
pub enum ReadEventError {
    NoNameError,
    NoStyleError,
    NoFillError,
    NoStrokeError,
    LastBitError,
    FirstBitError,
    NoPathError,
    NoDAttributeError,
}

impl fmt::Display for ReadEventError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ReadEventError::*;
        match self {
            NoNameError => write!(
                f,
                "Region's name not found: 'inkscape:label' attribute expected."
            ),
            NoStyleError => write!(f, "No style attribute found"),
            NoFillError => write!(f, "No 'fill' property found"),
            NoStrokeError => write!(f, "No 'stroke' property found"),
            LastBitError => write!(f, "A property's value could not be processed"),
            FirstBitError => write!(f, "A property's key could not be processed"),
            NoPathError => write!(f, "This SVG element is not a Path"),
            NoDAttributeError => write!(f, "Can't find the d attribute in this path elment"),
        }
    }
}

impl error::Error for ReadEventError {}
