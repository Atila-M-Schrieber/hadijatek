use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::{io, rc::Rc};

use prelude::lang;
use prelude::{
    draw::{svg::*, Color},
    lang::*,
    team::Team,
};

use eyre::Result;

pub fn get_lang() -> Result<()> {
    let mut s = String::new();
    print!("Válassz nyelvet (enter = magyar) / Select language (enter = hungarian): ");
    io::stdin().read_line(&mut s)?;

    while match_set_language(&s).is_err() {
        // If it's just empty, stick with default
        if s.is_empty() {
            println!(
                "{}: {}",
                lang!["Alapértelmezett nyelv használata", "Using default language"],
                get_language()
            );
            return Ok(());
        }

        print!("Invalid opció, próbálkozz újra / Invalid option, try again: ");
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
            println!("{i}: {}", path.display());
            paths.insert(i, path);
        }
    }

    print!(
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

        print!(
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
            print!(
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
            print!(
                "{}",
                lang![
                    "Nem sikerült a .teams file-t beolvasni. ",
                    "Could not parse .teams file. "
                ]
            );
        }
    } else {
        print!("{}", lang!["Nincs .teams file. ", "No .teams file found. "]);
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
        print!("{}", lang!["A csapat neve: ", "The name of the team: "]);
        io::stdin().read_line(&mut s)?;
        let name = s.clone();

        if s.is_empty() {
            break;
        }

        print!("{}", lang!["A csapat színe: ", "The team's color: "]);
        io::stdin().read_line(&mut s)?;

        let color;
        loop {
            if let Ok(col) = s.parse::<Color>() {
                color = col;
                break;
            }

            print!(
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

pub fn get_regions(path: &str, teams: Vec<Rc<Team>>) -> Result<()> {
    let mut content = String::new();

    let mut stuff = Vec::new();
    for event in svg::open(path, &mut content)? {
        stuff.push(read_event(event, &teams));
    }

    let num_succ = stuff.iter().flatten().count();

    println!("Succeded: {}, Failed: {}", num_succ, stuff.len() - num_succ);

    Ok(())
}
