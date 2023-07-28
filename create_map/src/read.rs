use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::{io, rc::Rc};

use prelude::{
    db::{legacy::Legacy, Database},
    draw::{svg::*, Color, Shape},
    game::{region::Base, team::Team},
    lang,
    lang::*,
};

use eyre::eyre;
use eyre::Result;

pub type PreRegion = (String, Option<RefCell<Base>>, Shape, Color, Color);

macro_rules! print_flush {
    ($($arg:tt)*) => {{
        use std::io::Write;
        print!($($arg)*);
        std::io::stdout().flush().expect("Failed to flush stdout");
    }};
}

pub fn get_lang() -> Result<()> {
    let mut s = String::new();
    print_flush!("Válassz nyelvet (enter = magyar) / Select language (enter = hungarian): ");

    io::stdin().read_line(&mut s)?;

    while match_set_language(&s).is_err() {
        // If it's just empty, stick with default
        if s == "\n" {
            println!(
                "{}: {}",
                lang!["Alapértelmezett nyelv használata", "Using default language"],
                get_language()
            );
            return Ok(());
        }

        print_flush!("Invalid opció, próbálkozz újra / Invalid option, try again: ");
        io::stdin().read_line(&mut s)?;
    }

    println!(
        "{}: {}",
        lang!["Kiválasztott nyelv", "Chosen language"],
        get_language()
    );

    Ok(())
}

pub fn get_path() -> Result<String> {
    println!(
        "{}",
        lang![
            "Az ebben a mappában található .svg file-ok közül válaszd ki a térkép SVG-ét:",
            "Select the map's SVG from the .svg files found in this folder: "
        ]
    );

    let mut paths = HashMap::new();
    for (i, entry) in fs::read_dir(".")?.enumerate() {
        let path = entry?.path();
        if path.is_file()
            && path.extension().map(|s| s.to_string_lossy().to_lowercase()) == Some("svg".into())
        {
            paths.insert(i, path);
        }
    }

    match paths.len() {
        // If only one .svg, it must be that
        1 => {
            let path: String = paths.into_iter().next().unwrap().1.to_string_lossy().into();
            println!(
                "{}: {}",
                lang!["Csak egy .svg file található", "Only one .svg file found"],
                path
            );
            print_flush!(
                "{}: ",
                lang!["Nyomj Entert ha ez jó", "Press Enter to confirm"]
            );
            let mut c = [0; 1];
            io::stdin().read_exact(&mut c)?;
            match c[0] as char {
                '\n' => Ok(path),
                _ => {
                    Err(io::Error::new(io::ErrorKind::NotFound, "Single file not selected").into())
                }
            }
        }
        0 => {
            println!(
                "{}",
                lang![
                "Nincs .svg file ebben a mappában. Futtasd ezt a programot ott, ahol a térkép van.",
                "No .svg files in this directory. Run this program where the map file is."
            ]
            );
            Err(io::Error::new(io::ErrorKind::NotFound, "No .svg file found.").into())
        }
        _ => {
            for (i, path) in &paths {
                println!("{i}: {}", path.display());
            }

            print_flush!(
                "{}",
                lang!["A térkép file száma: ", "The map file's number: "]
            );
            let mut s = String::new();
            io::stdin().read_line(&mut s)?;

            loop {
                let pass = match s.parse::<usize>() {
                    Err(_) => false,
                    Ok(i) if i > paths.len() => false,
                    _ => true,
                };

                if pass {
                    break;
                }

                print_flush!(
                    "{}",
                    lang![
                        "Ilyen számú file nem található, próbáld újra: ",
                        "No such file number found, try again: "
                    ]
                );
                io::stdin().read_line(&mut s)?;
            }

            let path = paths.get(&s.parse::<usize>()?).unwrap().to_string_lossy();

            Ok(path.into())
        }
    }
}

/// Gets teams from user input, or *path*.teams file, in which case it asks the user if the teams
/// look good or not.
pub fn get_teams(path: &str) -> Result<Vec<Rc<Team>>> {
    let teams = fs::read_to_string(path.to_owned() + ".teams");
    // If teams file exists, load from there
    if let Ok(teams) = teams {
        println!(
            "{}",
            lang![
                "Előző .teams file létezik, beolvasás...",
                "Existing .teams file found, loading..."
            ]
        );
        let teams: serde_json::Result<Vec<Team>> = serde_json::from_str(&teams);
        if let Ok(teams) = teams {
            let teams: Vec<Rc<Team>> = teams.into_iter().map(Rc::new).collect();
            println!("{}", lang!["Beolvasott csapatok:", "Parsed teams:"]);
            for team in &teams {
                println!("{:?}", *team);
            }
            print_flush!(
                "{}",
                lang![
                    "Nyomj Enter-t, ha jók a csapatok. Ha nem, írj be bármi mást: ",
                    "Press Enter to continue with these teams. If not, then press any other key: "
                ]
            );
            let mut c = [0; 1];
            io::stdin().read_exact(&mut c)?;
            if c[0] as char == '\n' {
                return Ok(teams);
            }
        } else {
            print_flush!(
                "{}",
                lang![
                    "Nem sikerült a .teams file-t beolvasni. ",
                    "Could not parse .teams file. "
                ]
            );
        }
    } else {
        print_flush!("{}", lang!["Nincs .teams file. ", "No .teams file found. "]);
    }

    println!(
        "{}",
        lang![
            "Minden csapatot adj meg, majd amikor kész vagy, név helyett nyomj Enter-t.",
            "Provide each team's info, press Enter instead of giving a name when done."
        ]
    );

    let mut s = String::new();
    let mut teams = Vec::new();
    loop {
        print_flush!("{}", lang!["A csapat neve: ", "The name of the team: "]);
        s.clear();
        io::stdin().read_line(&mut s)?;
        let name = s.clone().trim().to_string();

        if teams.iter().any(|t: &Team| t.name() == &name) {
            println!(
                "{}",
                lang![
                    "Két csapatnak nem lehet azonos neve!",
                    "Team names must be unique!"
                ]
            );
            continue;
        }

        dbg!(&name);
        if s == "\n" {
            break;
        }

        print_flush!("{}", lang!["A csapat színe: ", "The team's color: "]);
        s.clear();
        io::stdin().read_line(&mut s)?;

        let color;
        loop {
            if let Ok(col) = s.trim().parse::<Color>() {
                color = col;
                break;
            }

            print_flush!(
                "{}",
                lang![
                    "Ilyen szín nincs, próbáld újra: ",
                    "No such color, try again: "
                ]
            );
            io::stdin().read_line(&mut s)?;
        }

        teams.push(Team::new(name, color));
    }

    let json = serde_json::to_string(&teams)?;
    fs::write(path.to_owned() + ".teams", json)?;

    let teams: Vec<Rc<Team>> = teams.into_iter().map(Rc::new).collect();
    Ok(teams)
}

pub fn get_regions(path: &str, teams: &[Rc<Team>]) -> Result<Vec<PreRegion>> {
    let mut content = String::new();
    let mut pre_regs = svg::open(path, &mut content)?
        .filter_map(|event| read_event(event, teams).ok())
        .collect::<Vec<_>>();
    pre_regs.sort_by_key(|(name, _, _, _, _)| name.clone());
    Ok(pre_regs)
}

pub fn get_db(path: &str, water_stroke: Color, land_stroke: Color) -> Result<Box<dyn Database>> {
    print_flush!("{}: ",
        lang![
        "Standard (SurrealDB) használatához nyomj entert, régi .hmap-ba kiíráshoz írdd be, hogy \"L\"",
        "To use the default (SurrealDB) database, press enter. To output to legacy .hmap file, press \"L\""
        ]);

    let mut s = String::new();
    io::stdin().read_line(&mut s)?;

    let _legacy = match s.chars().next() {
        Some('\n') => Err(eyre!("db not implemented")),
        Some('L') => Ok(true),
        _ => Err(eyre!("No such db")),
    }?;

    // path will be .svg, so this is ok
    let mut name = path[2..]
        .split_once('.')
        .unzip()
        .0
        .ok_or(eyre!("How could this happen to me?"))?;

    print_flush!(
        "\"{}\" {}: ",
        name,
        lang![
            "lesz az alapértelmezett név, ha megfelel, nyomj entert, ha nem, írj új nevet",
            "will be the default name, if this is correct, press Enter, if not, write a new name"
        ]
    );

    s.clear();
    io::stdin().read_line(&mut s)?;

    if s != "\n" {
        name = s.trim();
    }

    Ok(Box::new(Legacy::new(
        name.to_owned(),
        water_stroke,
        land_stroke,
    )))
}
