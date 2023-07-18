mod map;
mod read;
mod write;

use std::io;

use eyre::Result;
use prelude::{draw::Color, lang, lang::Language, lang::LANGUAGE, State};

use crate::map::*;
use crate::read::*;
use crate::write::*;

fn main() -> Result<()> {
    get_lang()?;

    let path = get_path()?;

    print!(
        "{}",
        lang![
            "Mi a tengeri mezők színe",
            "What is the color of the sea regions"
        ] + ": "
    );
    io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
    let mut water_color = String::new();
    io::stdin().read_line(&mut water_color)?;
    let water_color: Color = water_color.trim().parse()?;

    let teams = get_teams(&path)?;
    let pre_regions = get_regions(&path, &teams).unwrap();
    let map = mapify(pre_regions, water_color)?;
    let state = State::new(teams, map);

    let db = get_db()?;

    write_state(state, db)?;

    Ok(())
}
