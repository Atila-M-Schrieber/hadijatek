mod read;

use std::io;

use eyre::Result;
use prelude::{draw::Color, lang, lang::Language, lang::LANGUAGE};

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
    io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
    let mut water_color = String::new();
    io::stdin().read_line(&mut water_color)?;
    let water_color: Color = water_color.trim().parse()?;

    let teams = get_teams(&path)?;

    get_regions(&path, teams).unwrap();

    Ok(())
}
