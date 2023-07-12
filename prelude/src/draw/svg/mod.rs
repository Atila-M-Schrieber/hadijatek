use std::rc::Rc;

use svg::node::element::path::{Command, Data};
use svg::{node::element::tag::Path, parser::Event};

use eyre::Result;

use super::{Color, Shape};
use crate::game::region as rg;
use crate::team::Team;

mod error;

pub fn read_event(
    event: Event<'_>,
    teams: Vec<Rc<Team>>,
) -> Result<(String, Option<rg::Base>, Shape, Color)> {
    use error::ReadEventError::*;
    if let Event::Tag(Path, _, attributes) = event {
        let name = attributes
            .get("inkscape:label")
            .ok_or(NoNameError)?
            .to_string();

        let style = attributes.get("style").ok_or(NoStyleError)?;
        let mut fill_color: Result<Color> = Err(NoFillError.into());
        let mut stroke_color: Result<Color> = Err(NoStrokeError.into());
        for property in style.split(";") {
            let mut bits: Vec<_> = property.split(":").map(|s| s.trim()).collect();
            match (
                bits.pop().ok_or(LastBitError)?,
                bits.pop().ok_or(FirstBitError)?,
            ) {
                (color, "fill") => fill_color = color.parse(),
                (color, "stroke") => stroke_color = color.parse(),
                _ => (),
            }
        }
        let color = fill_color?;
        let stroke_color = stroke_color?;

        let mut base = None;
        if stroke_color == Color::new(0, 0, 0) {
            base = Some(rg::Base::new())
        }
        // TODO take teams, and if color of team matches color of region, it is the default owner

        let data = Data::parse(attributes.get("d").ok_or(NoDAttributeError)?)?;
        let shape = data.try_into()?;

        println!(
            "{name}: fill:{color}; stroke:{stroke_color}; shape: {:?}",
            shape
        );
        return Ok((name, base, shape, color));
    }
    Err(NoPathError.into())
}
