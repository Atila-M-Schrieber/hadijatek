use prelude::draw::svg::*;
use prelude::draw::Color;
use svg::node::element::path::{Command, Data};
use svg::{node::element::tag::Path, parser::Event};

use eyre::Result;

pub fn get_regions(path: &str) -> Result<()> {
    let mut content = String::new();

    let mut stuff = Vec::new();
    for event in svg::open(path, &mut content)? {
        stuff.push(read_event(event, vec![]));
    }

    let num_succ = stuff.iter().flatten().count();

    println!("Succeded: {}, Failed: {}", num_succ, stuff.len() - num_succ);

    Ok(())
}
