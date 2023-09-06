use std::{cmp::Ordering, collections::HashSet};

use leptos::ev::Event;
use leptos::*;
use map_utils::{Color, Point, PreRegion};
use petgraph::{csr::Csr, visit::IntoNodeReferences, Undirected};

use crate::lang::*;

#[component]
pub fn DisplayPreMap(
    pre_regions: Result<Csr<PreRegion, (), Undirected>, ServerFnError>,
    select: WriteSignal<Option<PreRegion>>,
    water_color: RwSignal<Color>,
    water_stroke: RwSignal<Color>,
    land_stroke: RwSignal<Color>,
    done: WriteSignal<bool>,
    // get water_color, water_stroke, and land_stroke as signals
) -> impl IntoView {
    let pre_regions = match pre_regions {
        Ok(prs) if prs.node_references().next().is_none() => return ().into_view(),
        Ok(prs) => prs,
        // TODO: much better error handling here
        Err(err) => return view! {<p>"Something's wrong:" {err.to_string()}</p>}.into_view(),
    };
    let pre_regions_vec: Vec<(usize, PreRegion)> = pre_regions
        .node_references()
        .map(|(i, pr)| (i as usize, pr.clone()))
        .collect();
    // don't want to re-calculate this
    let extent: (f32, f32) = pre_regions_vec
        .iter()
        .fold(Point::new(0., 0.), |p_max, (_, pr)| {
            pr.shape.points().iter().fold(p_max, |mut p_max, pt| {
                let (x0, y0) = p_max.get();
                let (x1, y1) = pt.get();
                p_max.move_abs(f32::max(x0, x1), f32::max(y0, y1));
                p_max
            })
        })
        .into();

    // make this a derived signal so auto-sort for water color & the like
    let pre_regions_vec = create_memo(move |_| {
        // sort signal based on the water_color (within the definition of the signal)
        let mut pre_regions_vec = pre_regions_vec.clone();
        let wc = water_color.get(); // for some reason this warns
        pre_regions_vec.sort_by(|(_, pr1), (_, pr2)| match (pr1.color, pr2.color) {
            (c, c_) if c == c_ && c == wc => Ordering::Equal,
            (c, _) if c == wc => Ordering::Less,
            (_, c) if c == wc => Ordering::Greater,
            (_, _) => pr1.has_base.cmp(&pr2.has_base),
        });
        pre_regions_vec
    });

    // TODO: optimize all the cloning (probably not a huge deal)
    let view_region = move |(_, pr): (_, PreRegion)| {
        let stroke_info = move || {
            if pr.color == water_color.get() {
                (water_stroke.get().to_string(), 2)
            } else if pr.has_base {
                ("black".into(), 2)
            } else {
                (land_stroke.get().to_string(), 2)
            }
        };
        let stroke = move || stroke_info().0;
        let stroke_width = move || stroke_info().1;

        let pr_clone = pr.clone();
        let (fill, set_fill) = create_signal(pr.color);
        let fill_str = move || fill().to_string();
        let highlight = move |_| set_fill.update(|col| *col = col.highlight(0.15));
        let unhighlight = move |_| set_fill(pr.color);

        view! {
            <path on:click=move|_|select(Some(pr_clone.clone())) on:mouseenter=highlight
                on:mouseleave=unhighlight d=pr.shape.to_data_string()
                stroke=stroke stroke-width=stroke_width stroke-linejoin="bevel"
                fill=fill_str >
                <title>{pr.name}</title>
            </path>
        }
    };

    let view_regions = move || {
        let view = pre_regions_vec()
            .into_iter()
            .map(view_region)
            .collect_view();
        done(true);
        view
    };

    // toggle borders
    let (checked, set_checked) = create_signal(true);
    let check = move |_ev: Event| {
        set_checked.update(|b| *b = !*b);
    };

    // Border between two regions, colored to reflect the type of border
    let view_border = move |pr1: &PreRegion, pr2: &PreRegion| {
        let (x1, y1) = pr1.pole.into();
        let (x2, y2) = pr2.pole.into();
        let color1 = pr1.color;
        let color2 = pr2.color;

        let wc = water_color();

        let sea = color1 == wc || color2 == wc;
        let border_color = if sea { water_stroke() } else { land_stroke() };
        let border_color = border_color.to_string();

        view! {
            <line x1=x1 y1=y1 x2=x2 y2=y2 stroke="black" stroke-width="3"/>
            <line x1=x1 y1=y1 x2=x2 y2=y2
                stroke=border_color stroke-width="1"/>
        }
    };

    let prev_colors = create_rw_signal((Color::black(), Color::black()));
    // This is an infinite loop, should fix, would be nice
    let need_new_borders = create_memo(move |prev| {
        let prevs = prev_colors();
        let currents = (water_stroke(), land_stroke());
        prev_colors.set(currents);
        if prev.is_none() {
            true
        } else {
            currents != prevs
        }
    });

    // create_effect(move |_| log!("{}", need_new_borders()));

    let view_borders = move || {
        //if !need_new_borders() {
        //    return ().into_view();
        //}

        let mut views = Vec::new();

        let mut visited = HashSet::new();

        for (i, pr) in pre_regions.node_references() {
            for &j in pre_regions.neighbors_slice(i) {
                let pair = if i < j { (i, j) } else { (j, i) };
                if !visited.contains(&pair) {
                    visited.insert(pair);
                    views.push(view_border(pr, &pre_regions[j]))
                }
            }
        }

        views.into_view()
    };
    let view_borders = store_value(view_borders); // damnit why

    let cover_lines = move |(_, pr): (_, PreRegion)| {
        let (cx, cy) = pr.pole.into();

        let color = if pr.color == water_color() {
            water_stroke()
        } else {
            land_stroke()
        }
        .to_string();

        view! {
            <circle cx=cx cy=cy r="1.5" fill=color />
        }
    };

    let cover_all_lines = move || {
        pre_regions_vec()
            .into_iter()
            .map(cover_lines)
            .collect_view()
    };

    view! {
        <div class="map-container" >
        <div class="svg-container" >
            <svg viewBox=format!("0 0 {} {}", extent.0, extent.1) >
                {view_regions}
                <Show when=checked fallback=||() >
                {view_borders.get_value()}
                {cover_all_lines}
                </Show>
                <rect x=0 y=0 width=extent.0 height=extent.1 stroke="black" fill="none" />
            </svg>
        </div>
        <div class="checkbox-group">
            <input type="checkbox" id="borders" name="borders" on:input=check checked/>
            <label for="borders" ><Show when=checked fallback=||view!{
                <Lang hu="Határok mutatása" en="Show borders"/>}>
                <Lang hu="Határok elrejtése" en="Hide borders"/>
            </Show></label>
        </div>
        </div>
    }
    .into_view()
}
