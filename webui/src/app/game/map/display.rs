use leptos::html::Div;
use leptos::svg::Svg;
use leptos::*;
use leptos::{ev::Event, svg::Text};
use map_utils::{Color, Goodness, Label, Point, PreRegion, Shape};
use petgraph::{csr::Csr, visit::IntoNodeReferences, Undirected};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};
use wasm_bindgen::JsCast;
use web_sys::SvgTextElement;

use crate::lang::*;

#[component]
pub fn DisplayPreMap(
    pre_regions: Signal<Csr<PreRegion, (), Undirected>>,
    goodnesses: Signal<HashMap<u32, Goodness>>,
    initial_labels: Signal<HashMap<u32, Label>>,
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

    let highlighted = create_rw_signal(HashSet::new()); // the highlighted fields

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
        let highlight = move |_| {
            highlighted.update(|hs| {
                hs.insert(i);
            });
        };
        let unhighlight = move |_| {
            highlighted.update(|hs| {
                hs.remove(&i);
            })
        }; //set_fill(pr.color);

        let fill = move || {
            if highlighted.with(|hs| hs.contains(&i)) {
                pr.color.highlight(0.15)
            } else {
                pr.color
            }
        };

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

        view! {
            <path on:click=move|_|select(Some(pr_clone.clone())) on:mouseenter=highlight
                on:mouseleave=unhighlight d=pr.shape.to_data_string()
                stroke=stroke stroke-width=stroke_width stroke-linejoin="bevel"
                fill=fill_str />
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

    let view_info = move |(i, pr): (usize, PreRegion)| {
        let name_ref = create_node_ref::<Text>();
        let info_ref = create_node_ref::<Div>();

        let (text_locked, set_text_locked) = create_signal(true);
        create_effect(move |_| request_animation_frame(move || set_text_locked(false)));

        let fitting_font_size = create_rw_signal(13);
        let name_wh = create_rw_signal((1., 1.)); // 1 to avoid inf/nan
        let times_info_wh_called = create_rw_signal(0);
        let info_wh = create_memo(move |prev: Option<&(f32, f32)>| {
            let mut wh = (1., 1.);
            if text_locked() {
                return wh;
            }
            if let Some(prev) = prev {
                if times_info_wh_called() > 2 {
                    return *prev;
                }
            }
            times_info_wh_called.update(|n| *n += 1);
            let _ = highlighted.with(|_| ()); // subscribe to highlighted
            if let Some(info_ref) = info_ref() {
                let b_box = info_ref.get_bounding_client_rect();
                wh = (b_box.width() as f32, b_box.height() as f32);
            }
            wh
        });
        let text_wh = move || {
            let (nw, nh): (f32, f32) = name_wh();
            let (iw, ih) = info_wh(); // info dims are 'regular'-scale, name dims are svg-scale
            (
                nw,                       // Extra space for info for short names
                1.25 * nh + ih * nw / iw, // Have to scale ih to match svg dims
            )
        };
        let prnc = pr.name.clone();
        //create_effect(move |_| {
        //    log!(
        //        "{}: text_wh: {:?}, name_wh: {:?}, what to min: {}",
        //        &prnc,
        //        text_wh(),
        //        name_wh(),
        //        name_wh().1 * 5.
        //    )
        //});

        let overflows = move || pr.pole.get().1 + text_wh().1 - name_wh().1 / 2. > extent.1;

        create_effect(move |_| {
            if overflows() {
                log!("{} overflows", prnc)
            }
        });

        let times_name_pos_called = create_rw_signal(0);
        let name_pos = create_memo(move |prev: Option<&(f32, f32)>| {
            if let Some(prev) = prev {
                if times_name_pos_called() > 1 {
                    return *prev;
                }
            }
            times_name_pos_called.update(|n| *n += 1);
            let mut base_pos = (
                pr.pole.get().0, // - len as f32 * 13. / 4.,
                pr.pole.get().1, // + 13. / 3.,
            );
            if text_locked() {
                return base_pos;
            }
            if let Some(name_ref) = name_ref() {
                if let Ok(b_box) = (*name_ref)
                    .clone()
                    .dyn_into::<SvgTextElement>()
                    .map(|tr| tr.get_bounding_client_rect())
                {
                    let (w, h) = (b_box.width() as f32, b_box.height() as f32);
                    name_wh.set((w.max(h * 4.), h)); // Initial wh + padding
                    let (new_x, new_font_size) = get_new_name_x(base_pos, (w, h), &pr.shape, 13);
                    fitting_font_size.set(new_font_size);
                    base_pos = (new_x, base_pos.1)
                }
            }
            base_pos
        });

        let font_size = move || {
            if highlighted.with(|hs| hs.contains(&i)) {
                14
            } else {
                fitting_font_size()
            }
        };

        let name_pos_x = move || name_pos().0;
        let name_pos_y = move || {
            name_pos().1
                - if highlighted.with(|hs| hs.contains(&i)) && overflows() {
                    info_wh().1 - name_wh().1
                } else {
                    0.
                }
        };
        let expanded_name_pos_y = move || {
            name_pos().1
                - if overflows() {
                    info_wh().1 - name_wh().1
                } else {
                    0.
                }
        };

        let style_string = move || format!("font: {}px serif;", font_size());

        let highlight_opacity = move || {
            let val = if highlighted.with(|hs| hs.contains(&i)) {
                0.9
            } else {
                0.
            };
            format!("opacity:{val};")
        };

        let rect_x = move || name_pos_x() - name_wh().0 / 2.;
        let rect_y = move || expanded_name_pos_y() - name_wh().1 / 2.;
        let rect_r = move || name_wh().1 / 8.;

        let info_y = move || expanded_name_pos_y() + name_wh().1 / 2.;

        view! {
            <svg>
                <rect class="highlight-rect" style=highlight_opacity x=rect_x y=rect_y rx=rect_r
                    width=move||text_wh().0 height=move||text_wh().1 />
                <text node_ref=name_ref x=name_pos_x y=name_pos_y
                    text-anchor="middle" dy="0.35em" style=style_string >
                    {pr.name}
                </text>
                <foreignObject
                    x=move||name_pos_x()-name_wh().0/2. y=info_y
                    width=move||text_wh().0 height=extent.1
                    style=highlight_opacity
                >
                    <div node_ref=info_ref style="font-size: 12px;" >
                        <hr/>
                        <p>Info thing that may wrap</p>
                        <p>More info</p>
                    </div>
                </foreignObject>
            </svg>
        }
        .into_view()
    };

    let view_infos = move || {
        let mut prv = pre_regions_vec();
        let len = prv.len() as f32;
        prv.sort_by(|(_, pr1), (_, pr2)| {
            let dist = |p: Point| (extent.1 * (1. - 0.5 / len.sqrt()) - p.get().1).abs();
            if (pr1.color == water_color()) == (pr2.color == water_color()) {
                dist(pr1.pole)
                    .partial_cmp(&dist(pr2.pole))
                    .unwrap_or(Ordering::Equal)
            } else {
                Ordering::Equal
            }
        });
        prv.into_iter().map(view_info).enumerate().collect_view()
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
            <line x1=x1 y1=y1 x2=x2 y2=y2 stroke="black" stroke-width="3"
                style="pointer-events: none;"/>
            <line x1=x1 y1=y1 x2=x2 y2=y2
                stroke=border_color stroke-width="1" style="pointer-events: none;"/>
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
                {/* initial_labels().values().map(|(p, _)| view!{
                    <circle cx={p.get().0} cy={p.get().1} r=3 fill="black"/>
                }).collect_view() */}
                <Show when=move||!goodness_checked().0 fallback=||()>
                    {view_infos}
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

// Sets the optimal x value, and the font size to go with it
fn get_new_name_x(
    (x, y): (f32, f32),
    (w, h): (f32, f32),
    shape: &Shape,
    initial_font_size: usize,
) -> (f32, usize) {
    let mut new_font_size = initial_font_size;
    let font_height_ratio = h / new_font_size as f32;
    let wh_ratio = w / h;

    let mut new_x = x;

    while new_font_size >= 1 {
        let dy = font_height_ratio * new_font_size as f32 / 2.;
        let upper_y = y + dy;
        let lower_y = y - dy;

        let upper_intersects = shape.intersects_x(upper_y);
        let lower_intersects = shape.intersects_x(lower_y);

        let left = move |v: &[f32]| {
            *v.iter()
                .filter(|&&ix| ix - x < 0.)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(&0.)
        };
        let right = move |v: &[f32]| {
            *v.iter()
                .filter(|&&ix| ix - x > 0.)
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(&0.)
        };

        // cloest intersects
        let ul = left(&upper_intersects);
        let ur = right(&upper_intersects);
        let ll = left(&lower_intersects);
        let lr = right(&lower_intersects);

        // Get the closest pair - this is the rectangle
        let (l, r) = (ul.max(ll), ur.min(lr));

        if (r - l) / (2. * dy) >= wh_ratio {
            let ideal_x = (l + r) / 2.;
            new_x = ideal_x;
            //let diff = ideal_x - x;
            //if diff.abs() > dy {
            //    new_x = ideal_x - diff.signum() * dy;
            //}
            break;
        }

        new_font_size -= 1;
    }

    (new_x, new_font_size)
}

// proper rendering, maybe?
/*let view_infos = create_memo(move |prev: Option<&Vec<(usize, View)>>| {
    let not_first = prev.is_some();
    if highlighted.with(|hs| hs.len() == 0) && not_first {
        return prev.unwrap().clone();
    }
    let mut prv = pre_regions_vec();
    if highlighted.with(|hs| hs.len() > 0) && prev.is_some() {
        highlighted.with(|hs| {
            hs.iter().for_each(|i| {
                log!("{i} is highlighted");
                let idx = prv.iter().position(|(idx, _)| idx == i).expect("valid i");
                let removed = prv.remove(idx);
                prv.push(removed)
            })
        });
    }
    prv.into_iter()
        .map(view_info)
        .enumerate()
        .collect::<Vec<_>>()
});*/
