use std::{error, fmt};

#[derive(Debug)]
pub enum RegionCreationError {
    BaseOnSea,
}

impl fmt::Display for RegionCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RegionCreationError::*;
        match self {
            BaseOnSea => write!(f, "Sea regions cannot have bases!"),
        }
    }
}

impl error::Error for RegionCreationError {}
