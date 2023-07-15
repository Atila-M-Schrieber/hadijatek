use std::cell::RefCell;
use std::rc::Rc;

use svg::node::element::path::Data;
use svg::parser::Event;

use eyre::Result;

use super::{Color, Shape};
use crate::game::region::Base;
use crate::team::Team;

mod error;

pub fn read_event(
    event: Event<'_>,
    teams: &[Rc<Team>],
) -> Result<(String, Option<RefCell<Base>>, Shape, Color)> {
    use error::ReadEventError::*;
    if let Event::Tag("path", _, attributes) = event {
        let name = attributes.get("inkscape:label").ok_or(NoName)?.to_string();

        let style = attributes.get("style").ok_or(NoStyle)?;
        let mut fill_color: Result<Color> = Err(NoFill.into());
        let mut stroke_color: Result<Color> = Err(NoStroke.into());
        for property in style.split(';') {
            let mut bits: Vec<_> = property.split(':').map(|s| s.trim()).collect();
            match (
                bits.pop().ok_or(BadLastBit)?,
                bits.pop().ok_or(BadFirstBit)?,
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
            base = Some(RefCell::new(Base::new()))
        }
        // if this is a home base set it to the home base
        if let Some(team) = teams.iter().find(|t| color == t.color()) {
            if let Some(ref base) = base {
                base.borrow_mut().set(Rc::clone(team))
            } else {
                return Err(TeamColorOnNonBaseRegion(color).into());
            }
        }

        let data = Data::parse(attributes.get("d").ok_or(NoDAttribute)?)?;
        let shape: Shape = data.try_into()?;

        return Ok((name, base, shape, color));
    }
    Err(NoPath.into())
}
