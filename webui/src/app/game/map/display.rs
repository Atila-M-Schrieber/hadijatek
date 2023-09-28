use leptos::html::Div;
use leptos::svg::Svg;
use leptos::*;
use leptos::{ev::Event, svg::Text};
use map_utils::team::Team;
use map_utils::unit::UnitType;
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
    teams: RwSignal<Vec<(usize, RwSignal<Color>, RwSignal<String>)>>,
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
    let pre_regions_signal = pre_regions.clone(); // TODO: optimize this
    let pre_regions_signal = Signal::derive(move || pre_regions_signal.clone());
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

    let index_map = create_rw_signal(HashMap::new());
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

        // update index map
        index_map.set(HashMap::new());
        pre_regions_vec.iter().enumerate().for_each(|(i, (j, _))| {
            index_map.update(|m| {
                m.insert(*j, i);
            })
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

    let test_unit = create_rw_signal(None);

    let select_unit = move || {
        use UnitType as UT;
        let change = move |ev: Event| {
            test_unit.set(match event_target_value(&ev).as_str() {
                "t" => Some(UT::Tank),
                "s" => Some(UT::Ship),
                "p" => Some(UT::Plane),
                "st" => Some(UT::Supertank),
                "su" => Some(UT::Submarine),
                "a" => Some(UT::Artillery),
                _ => None,
            })
        };

        view! {
            <select on:change=change >
                <option value=""><Lang hu="Nincs Egység" en="No Unit" /></option>
                <option value="t"><Lang hu="Tank" en="Tank" /></option>
                <option value="s"><Lang hu="Hajó" en="Ship" /></option>
                <option value="p"><Lang hu="Repülő" en="Plane" /></option>
                <option value="st"><Lang hu="Szupertank" en="Supertank" /></option>
                <option value="su"><Lang hu="Tengeralattjáró" en="Submarine" /></option>
                <option value="a"><Lang hu="Tüzérség" en="Artillery" /></option>
            </select>
        }
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

    let view_test_unit = move |(_, pr): (_, PreRegion)| {
        let color = pr.color.invert();

        // logic
        let (pos, set_pos) = create_signal(pr.pole);

        test_unit().map(|tu| {
            view! {
                <UnitSVG ut=tu color=color pos=pos />
            }
        })
    };

    let view_test_units = move || {
        pre_regions_vec()
            .into_iter()
            .map(view_test_unit)
            .collect_view()
    };

    let refs_locked = create_rw_signal(true);
    create_effect(move |_| request_animation_frame(move || refs_locked.set(false)));

    let view_name = move |(i, pr): (usize, PreRegion)| {
        const BASE_FONT: usize = 13;
        const EXPANDED_FONT: usize = 14;

        let name_ref = create_node_ref::<Text>();

        let fitting_font_size = create_rw_signal(BASE_FONT);

        // indirect highlighted sub - memoized
        let highlighted = create_memo(move |_| highlighted.with(|hs| hs.contains(&i)));

        // Wh at expanded font size
        let name_wh = create_memo(move |_prev: Option<&(f32, f32)>| {
            let mut wh = (EXPANDED_FONT as f32 * 4., EXPANDED_FONT as f32); // approx this
            if refs_locked() {
                return wh;
            }
            if let Some(name_ref) = name_ref() {
                if let Ok(b_box) = (*name_ref)
                    .clone()
                    .dyn_into::<SvgTextElement>()
                    .map(|tr| tr.get_bounding_client_rect())
                {
                    wh = (b_box.width() as f32, b_box.height() as f32)
                }
            }
            wh
        });

        let name_x = Signal::derive(move || {
            if refs_locked() {
                pr.pole.get().0
            } else {
                let (x0, y) = pr.pole.get();
                let (x, font) = get_new_name_x((x0, y), name_wh(), &pr.shape, BASE_FONT);
                fitting_font_size.set(font);
                x
            }
        });
        let name_y = move || pr.pole.get().1;

        let rect_w = move || {
            // max of all the infos
            name_wh().0
        };
        let rect_h = move || name_wh().1;
        // rect_h: sum of names / infos, maybe make rect_wh for less compute

        let rect_x = move || name_x() - rect_w() / 2.;
        let rect_y = move || name_y() - name_wh().1 / 2.;
        let rect_r = move || name_wh().1 / 8.;

        let font_style = move || {
            let font_size = if highlighted() {
                EXPANDED_FONT
            } else {
                fitting_font_size()
            };
            format!("font: {}px serif;", font_size)
        };

        let highlight_opacity = move || {
            let val = if highlighted() { 0.9 } else { 0. };
            format!("opacity:{val};")
        };

        view! {
            <rect class="highlight-rect" style=highlight_opacity x=rect_x y=rect_y rx=rect_r
                width=rect_w height=rect_h />
            <text node_ref=name_ref x=name_x y=name_y
                text-anchor="middle" dy="0.35em" style=font_style >
                {pr.name}
            </text>
        }
    };

    let view_names = move || {
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
        prv.into_iter().map(view_name).collect_view()
    };

    let view_info = move |(i, is_shore): (&usize, bool)| {
        let (_, pr) = pre_regions_vec.with(|prs| prs[index_map.with(|im| im[i])].clone());

        let region = move || {
            if pr.color == water_color() {
                view! {<Lang hu="Tenger" en="Sea" />}
            } else if is_shore {
                if is_strait(&pr.shape) {
                    view! {<Lang hu="Szoros" en="Strait" />}
                } else {
                    view! {<Lang hu="Tengerpart" en="Shore" />}
                }
            } else {
                view! {<Lang hu="Szárazföld" en="Land" />}
            }
        };

        let base = move || {
            if pr.has_base {
                if let Some((_, _, teamname)) =
                    teams.with(|ts| ts.iter().find(|(_, tc, _)| tc() == pr.color).cloned())
                {
                    Some(view! {"- "{teamname()}" "<Lang hu="anyabázis" en="home base" />})
                } else {
                    Some(view! {"- "<Lang hu="Foglalatlan bázis" en="Unconquered base" />})
                }
            } else {
                None
            }
        };

        let unit = move || {
            let color = Signal::derive(|| Color::white());
            test_unit().map(|tu| {
                view! {
                    <hr/>
                    <UnitSVG ut=tu color=color />
                    <p><Lang hu="Teszt" en="Test" />" "<UnitName ut=tu /></p>
                }
            })
        };

        view! {
            <div class="info" >
                <p class="name" >{pr.name}</p>
                <hr/>
                <p>{region}" "{base}</p>
                {unit}
            </div>
        }
    };

    let view_infos = Signal::derive(move || {
        highlighted()
            .iter()
            .map(|i| {
                (i, {
                    pre_regions_signal.with(|prs| {
                        prs.neighbors_slice(*i as u32).iter().any(|&j| {
                            pre_regions_signal.with(|prss| prss[j].color == water_color())
                        })
                    })
                })
            })
            .map(view_info)
            .collect_view()
    });

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
                <Show when={/* move||!goodness_checked().0 */ ||true} fallback=||()>
                    {view_test_units}
                    {view_names}
                </Show>
                <rect x=0 y=0 width=extent.0 height=extent.1 stroke="black" fill="none" />
            </svg>
        </div>
        <div class="info-container" >
            {view_infos}
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
        {select_unit}
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

    while new_font_size > 1 {
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

fn is_strait(shape: &Shape) -> bool {
    let points = shape.points();
    // By default reference to empty slice
    let mut strait_points = &points[0..0];

    for i in 0..points.len() - 3 {
        let (p1, p2) = (points[i], points[i + 1]);
        for j in i..points.len() - 1 {
            let (p3, p4) = (points[j], points[j + 1]);
            if p1 == p4 && p2 == p3 {
                strait_points = &points[i..=i + 1];
            }
        }
    }

    !strait_points.is_empty()
}

#[component]
fn UnitName(ut: UnitType) -> impl IntoView {
    use UnitType as UT;
    match ut {
        UT::Tank => view! {<Lang hu="Tank" en="Tank" />},
        UT::Ship => view! {<Lang hu="Hajó" en="Ship" />},
        UT::Plane => view! {<Lang hu="Repülő" en="Plane" />},
        UT::Supertank => view! {<Lang hu="Szupertank" en="Supertank" />},
        UT::Submarine => view! {<Lang hu="Tengeralattjáró" en="Submarine" />},
        UT::Artillery => view! {<Lang hu="Tüzérség" en="Artillery" />},
    }
}

#[component]
fn UnitSVG(
    ut: UnitType,
    #[prop(into)] color: Signal<Color>,
    #[prop(optional)] pos: Option<ReadSignal<Point>>, // these to if it's in an SVG
    #[prop(optional)] fit_inside: Option<Shape>,
) -> impl IntoView {
    let size = 1.; // Will come from fit_inside (largest supertank that can fit)

    let viewBox = create_rw_signal("-1.1 -1.1 2.2 2.2");
    let scale = create_rw_signal(1.);

    let sq32: f32 = (3f32 / 2f32).sqrt();
    let scale_points = move |pts: Vec<(f32, f32)>| {
        let pts: Vec<Point> = pts.into_iter().map(Point::from).collect();
        let centroid = Shape::new(&pts).centroid();
        let pts: Vec<Point> = pts.into_iter().map(|p| p - centroid).collect();

        // furthest from centroid (which is now 0,0 )
        let furthest_point = pts
            .iter()
            .max_by(|a, b| {
                a.square()
                    .partial_cmp(&b.square())
                    .unwrap_or(Ordering::Equal)
            })
            .expect("a max");
        let dist = furthest_point.square().sqrt(); // this should be scaled to 1

        pts.into_iter()
            .map(|p| p * (scale() / dist))
            .collect::<Vec<Point>>()
    };
    let points_to_string = move |pts: Vec<Point>| {
        pts.into_iter()
            .map(|p| {
                let (x, y) = p.get();
                format!("{x},{y}")
            })
            .collect::<Vec<String>>()
            .join(" ")
    };

    let style = move || {
        format!(
            "fill:{};stroke:black;stroke-width:{};stroke-linejoin:miter;",
            color(),
            0.1 * scale().sqrt() / size,
        )
    };

    use UnitType as UT;
    let shape = move || match ut {
        UT::Tank => {
            let points = vec![(0., -1.3), (sq32, 0.5), (-sq32, 0.5)];
            let points = points_to_string(scale_points(points));
            view! {<polygon points=points style=style />}.into_view()
        }
        UT::Ship => view! {<circle cx=0 cy=0 r=1 style=style />}.into_view(),
        UT::Plane => view! {<rect x=-1 y=-1 width=2 height=2 style=style />}.into_view(),
        UT::Supertank => {
            viewBox.set("-1.9 -1.1 3.8 2.2");
            scale.set(2.);
            let points = vec![
                (0., -1.3),
                (-sq32, 0.6),
                (2. * sq32, 0.6),
                (sq32, -1.3),
                (sq32 / 2., -0.25),
            ];
            let points = points_to_string(scale_points(points));
            view! {<polygon points=points style=style />}.into_view()
        }
        UT::Submarine => {
            viewBox.set("-1.1 -0.45 2.2 0.9");
            view! {<rect x=-1 y=-0.35 width=2 height=0.7 rx=0.35 style=style />}.into_view()
        }
        UT::Artillery => {
            let points = vec![
                (-1.3, 0.),
                (-0.6, sq32),
                (0.6, sq32),
                (1.3, 0.),
                (0.6, -sq32),
                (-0.6, -sq32),
            ];
            let points = points_to_string(scale_points(points));
            view! {<polygon points=points style=style />}.into_view()
        }
    };

    if let Some(pos) = pos {
        let side_len = 60; // will come from fit_inside
        let (mut x, mut y) = pos().get();
        x -= side_len as f32 / 2.;
        y -= side_len as f32 / 2.;
        view! {
            <svg style="pointer-events:none;"
                x=x y=y width=side_len height=side_len viewBox=viewBox >
                {shape}
            </svg>
        }
    } else {
        view! {
            <svg viewBox=viewBox >
                {shape}
            </svg>
        }
    }
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
