use prelude::draw::Color;
use svg::node::element::path::{Command, Data};
use svg::{node::element::tag::Path, parser::Event};

use eyre::Result;

fn read_event(event: Event<'_>) -> Option<()> {
    match event {
        Event::Tag(Path, _, attributes) => {
            let name = attributes.get("inkscape:label")?.to_string();
            let style = attributes.get("style")?;
            let mut fill_color: Option<Color> = None;
            let mut stroke_color: Option<Color> = None;
            for property in style.split(";") {
                let mut bits: Vec<_> = property.split(":").map(|s| s.trim()).collect();
                match (bits.pop()?, bits.pop()?) {
                    (color, "fill") => fill_color = color.parse().ok(),
                    (color, "stroke") => stroke_color = color.parse().ok(),
                    _ => (),
                }
            }
            let fill_color = fill_color?;
            let stroke_color = stroke_color?;
            println!("{name}: fill:{fill_color}; stroke:{stroke_color}");
            Some(())
        }
        _ => None,
    }
}

pub fn get_regions(path: &str) -> Result<()> {
    let mut content = String::new();

    let mut stuff = Vec::new();
    for event in svg::open(path, &mut content)? {
        stuff.push(read_event(event));
    }

    let num_succ = stuff.iter().flatten().count();

    println!("Succeded: {}, Failed: {}", num_succ, stuff.len() - num_succ);

    Ok(())
}
