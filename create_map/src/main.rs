mod map;
mod read;

use std::io;

use eyre::eyre;
use eyre::Result;
use prelude::{draw::Color, lang, lang::Language, lang::LANGUAGE, State};

use crate::map::*;
use crate::read::*;

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
