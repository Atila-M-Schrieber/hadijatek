use chrono::offset::Utc;
use chrono::DateTime;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};

pub mod map;
pub mod user;

#[cfg(feature = "ssr")]
mod server;
#[cfg(feature = "ssr")]
pub use server::*;

/// Tokens
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Token {
    pub created: DateTime<Utc>,
    pub token: String,
}
