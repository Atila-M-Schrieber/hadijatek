use leptos::ev::Event;
use leptos::*;
use map_utils::{Color, Goodness, Point, PreRegion};
use petgraph::{csr::Csr, visit::IntoNodeReferences, Undirected};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use crate::lang::*;

#[component]
pub fn DisplayPreMap(
    pre_regions: Signal<Csr<PreRegion, (), Undirected>>,
    goodnesses: Signal<HashMap<u32, Goodness>>,
    select: WriteSignal<Option<PreRegion>>,
    water_color: RwSignal<Color>,
    water_stroke: RwSignal<Color>,
    land_stroke: RwSignal<Color>,
    done: WriteSignal<bool>,
    // get water_color, water_stroke, and land_stroke as signals
) -> impl IntoView {
    let pre_regions = match pre_regions() {
        prs if prs.node_references().next().is_none() => return ().into_view(),
        prs => prs,
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

    // Good-ness (how good a region is based on how well distributed its borders are)
    let goodness_checked = create_rw_signal((false, false, true, true, true));
    let show_goodness_checked = move || goodness_checked().0;
    let check_goodness = move |_ev: Event| {
        goodness_checked.update(|(b, _, _, _, _)| *b = !*b);
    };
    let check_goodness_abs = move |_ev| {
        goodness_checked.update(|(_, b, _, _, _)| *b = !*b);
    };
    let check_goodness_ang = move |_ev: Event| {
        goodness_checked.update(|(_, _, b, _, _)| *b = !*b);
    };
    let check_goodness_num = move |_ev: Event| {
        goodness_checked.update(|(_, _, _, b, _)| *b = !*b);
    };
    let check_goodness_len = move |_ev: Event| {
        goodness_checked.update(|(_, _, _, _, b)| *b = !*b);
    };

    // TODO: optimize all the cloning (probably not a huge deal)
    let view_region = move |(i, pr): (usize, PreRegion)| {
        let stroke_info = move || {
            if goodness_checked().0 {
                ("none".into(), 0)
            } else if pr.color == water_color.get() {
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
        let highlight = move |_| set_fill.update(|col| *col = col.highlight(0.15));
        let unhighlight = move |_| set_fill(pr.color);

        // If goodness is shown
        let fill_str = move || {
            if !goodness_checked().0 {
                fill().to_string()
            } else {
                let (abs, rel) = goodnesses.with(|hm| hm[&(i as u32)].0);
                let color = if goodness_checked().1 { abs } else { rel };
                let (_, _, ang, num, len) = goodness_checked();
                let (r, g, b) = color.get();
                Color::new(ang as u8 * r, num as u8 * g, len as u8 * b).to_string()
            }
        };

        let title = move || {
            let name = pr.name.clone();
            if !goodness_checked().0 {
                name
            } else {
                let (abs, rel) = goodnesses.with(|hm| hm[&(i as u32)].0);
                format!("{name}: abs: {abs}; rel: {rel}; id: {i}")
            }
        };

        view! {
            <path on:click=move|_|select(Some(pr_clone.clone())) on:mouseenter=highlight
                on:mouseleave=unhighlight d=pr.shape.to_data_string()
                stroke=stroke stroke-width=stroke_width stroke-linejoin="bevel"
                fill=fill_str >
                <title>{title}</title>
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
    let (border_checked, set_border_checked) = create_signal(true);
    let check_border = move |_ev: Event| {
        set_border_checked.update(|b| *b = !*b);
    };

    // Border between two regions, colored to reflect the type of border
    // Also pushes the angle of the line onto the border_info
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

    let view_borders = create_memo(move |prev| {
        if let Some(prev) = prev {
            // only re-calculate if colors have changed / first time
            let (_, pwc, pws, pls) = prev;
            if pwc == &water_color() && pws == &water_stroke() && pls == &land_stroke() {
                return prev.clone();
            }
        }

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

        (
            views.into_view(),
            water_color(),
            water_stroke(),
            land_stroke(),
        )
    });
    // let view_borders = store_value(view_borders); // damnit why

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
                <Show when=border_checked fallback=||() >
                {view_borders().0/* .get_value() */}
                {cover_all_lines}
                </Show>
                <rect x=0 y=0 width=extent.0 height=extent.1 stroke="black" fill="none" />
            </svg>
        </div>
        <div class="checkbox-group">
            <input type="checkbox" id="borders" name="borders" on:input=check_border checked/>
            <label for="borders" ><Show when=border_checked fallback=||view!{
                <Lang hu="Határok mutatása" en="Show borders"/>}>
                <Lang hu="Határok elrejtése" en="Hide borders"/>
            </Show></label>
        </div>
        <div class="checkbox-group">
            <input type="checkbox" id="goodnesses" name="goodnesses" on:input=check_goodness />
            <label for="goodnesses" ><Show when=show_goodness_checked fallback=||view!{
                <Lang hu="Értékelés mutatása" en="Show evaluation"/>}>
                <Lang hu="Értékelés elrejtése" en="Hide evaluation"/>
            </Show></label>
        </div>
        <Show when=show_goodness_checked fallback=||() >
        <div class="goodness-checkboxes">
            <div class="button-group">
                <button type="button" id="goodness_abs" name="goodness_abs"
                    on:click=check_goodness_abs >
                <Show when=move||goodness_checked().1 fallback=||view!{
                    <Lang hu="Abszolút értékelés mutatása" en="Show absolute evaluation"/>}>
                    <Lang hu="Relatív értékelés mutatása" en="Show relative evaluation"/>
                </Show></button>
            </div>
            <div class="checkbox-group">
                <input type="checkbox" id="goodness_ang" name="goodness_ang"
                    on:input=check_goodness_ang checked=move||goodness_checked().2/>
                <label for="goodness_abs" >
                    <Lang hu="Szögek jósága" en="Angular goodness"/>
                </label>
            </div>
            <div class="checkbox-group">
                <input type="checkbox" id="goodness_num" name="goodness_num"
                    on:input=check_goodness_num checked=move||goodness_checked().3/>
                <label for="goodness_num" >
                    <Lang hu="Szomszédok jósága" en="Neighbors goodness"/>
                </label>
            </div>
            <div class="checkbox-group">
                <input type="checkbox" id="goodness_len" name="goodness_len"
                    on:input=check_goodness_len checked=move||goodness_checked().4/>
                <label for="goodness_len" >
                    <Lang hu="Hosszak jósága" en="Lengthwise goodness"/>
                </label>
            </div>
        </div>
        </Show>
        </div>
    }
    .into_view()
}
