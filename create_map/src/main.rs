//! # create_map
//!
//! Command line tool for taking an Inkscape svg of a valid Hadijáték map, and writing an initial
//! game state from it.
//!
//! ## Rules for valid maps:
//! * No Bèzier curves - subject to change
//! * All Regions are closed paths
//! * No repeating points (except Straits)
//! * Straits must have a pair of successive colocated points - e.g. have a thin line connecting
//! the two land parts of the strait, which necessarily has 4 points: (**first land part** - a
//! (first point on connecting line) - b (point on the other end of the connecting line) - **other
//! land part** - c (point colocated with b) - d (point colocated with a) - **rest of first land
//! part**)
//! * Regions which border each other must have points in the same places along their border
//! * Sea Regions cannot contain bases
//! * The stroke of Regions with bases must be black, all other Regions must have non-black strokes
//! * All sea Regions must be of the same color.
//! * Land Regions should have a variety of colors (preferably low blue values).
//! * Teams' home Regions must have the given Team's color.
//! * Neighboring Regions may only have one type of border - for example, two Sea regions may not
//! be connected both directly and through a strait.

mod map;
mod read;

use std::io;

use eyre::eyre;
use eyre::Result;
use prelude::{draw::Color, game::State, lang};

use crate::map::*;
use crate::read::*;

fn main() -> Result<()> {
    get_lang()?;

    let path = get_path()?;

    // maybe add auto-detection of water_color
    print!(
        "{}",
        lang![
            "Mi a tengeri mezők színe",
            "What is the color of the sea regions"
        ] + ": "
    );
    io::Write::flush(&mut io::stdout())?;
    let mut water_color = String::new();
    io::stdin().read_line(&mut water_color)?;
    let water_color: Color = water_color.trim().parse()?;

    let teams = get_teams(&path)?;
    let pre_regions = get_regions(&path, &teams)?;

    let water_stroke = pre_regions
        .iter()
        .find(|t| t.3 == water_color)
        .ok_or(eyre!("No water regions"))?
        .4;

    let land_stroke = pre_regions
        .iter()
        .find(|t| t.3 != water_color && t.4 != Color::black())
        .ok_or(eyre!("No baseless land regions"))?
        .4;

    let map = mapify(pre_regions, water_color)?;
    let state = State::new(teams, map);

    let mut db = get_db(&path, water_stroke, land_stroke)?;

    db.read_from_state(state)?;
    db.write()?;

    Ok(())
}
