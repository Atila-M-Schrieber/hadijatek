use leptos::*;
use map_utils::{Color, Point, PreRegion};
use petgraph::{csr::Csr, visit::IntoNodeReferences, Undirected};

// use crate::lang::*;

#[component]
pub fn DisplayPreMap(
    pre_regions: Result<Csr<PreRegion, (), Undirected>, ServerFnError>,
    select: WriteSignal<Option<PreRegion>>,
    // get water_color, water_stroke, and land_stroke as signals
) -> impl IntoView {
    // make this a signal, derive a 'neighbors' signal
    let mut pre_regions_vec: Vec<(usize, PreRegion)> = match pre_regions {
        Ok(prs) => prs
            .node_references()
            .map(|(i, pr)| (i as usize, pr.clone()))
            .collect(),
        Err(err) => return view! {<p>"Something's wrong:" {err.to_string()}</p>}.into_view(),
    };

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
        .get();

    // sort signal based on the water_color (within the definition of the signal)
    pre_regions_vec.sort_by(|(_, pr1), (_, pr2)| pr1.has_base.cmp(&pr2.has_base));

    // TODO: optimize all the cloning (probably not a huge deal)
    let show = |(_, pr): (_, PreRegion)| {
        let stroke = if pr.has_base { "black" } else { "#bbbbbb" };
        let pr_clone = pr.clone();
        let (fill, set_fill) = create_signal(pr.color);
        let fill_str = move || fill().to_string();
        let highlight = move |_| set_fill.update(|col| *col = col.highlight(0.15));
        let unhighlight = move |_| set_fill(pr.color);
        view! {
            <path on:click=move|_|select(Some(pr_clone.clone())) on:mouseenter=highlight
                on:mouseleave=unhighlight d=pr.shape.to_data_string()
                stroke=stroke fill=fill_str >
                <title>{pr.name}</title>
            </path>
        }
    };

    view! {
        <div class="svg-container" >
            <svg viewBox=format!("0 0 {} {}", extent.0, extent.1) >
                {pre_regions_vec.into_iter().map(show).collect_view()}
                <rect x=0 y=0 width=extent.0 height=extent.1 stroke="black" fill="none" />
            </svg>
        </div>
    }
    .into_view()
}
