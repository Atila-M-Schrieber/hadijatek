//! # Prelude
//!
//! The Hadijáték prelude is a collection of common functions used in the create_map, webui, and
//! adjudicator crates. It provides the structure underlying the project, defining the structures
//! necessary for constructing the state of a game: teams, regions, units, and orders; a set of
//! commonly used SVG tools, and provides the interface between other crates and the database. It
//! also provides a simple implementation of multiple language support.
//!
//! ## Game
//!
//! The game module provides access to the data directly related to the game: teams, regions,
//! units, and orders, whose current state makes up the game's state.
//!
//! ## Draw
//!
//! Contains some simple types to simplify working with SVG's, associated methods, and conversion
//! between SVG's and these types.
//!
//! ## DB
//!
//! Provides the Database trait and its associated methods. Currently only supports writing to
//! Legacy "databases", which are the .hmap files which were used in ye olden days of the Haskell
//! version of this program. I will need to create reading from .hmap-s as well for the sake of
//! testing based on the first online game, but I hope not to need it for too long.
//!
//! SurrealDB support is upcoming, and will be the primary database for the game.
//!
//! ## Lang
//!
//! This game, as far as I know, has only been played in Hungarian, but I want to be able to spread
//! this game internationally, so multilingual support is essential. Currently only Hungarian
//! (the default) and English are supported.

pub mod db;
pub mod draw;
mod game;
pub mod lang;
// pub mod misc;

pub use game::*;

#[cfg(test)]
mod tests {
    // use super::*;
}
