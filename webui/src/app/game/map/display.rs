use leptos::*;
use map_utils::{Color, Point, PreRegion};

// use crate::lang::*;

#[component]
pub fn DisplayPreMap(
    pre_regions: Result<Vec<PreRegion>, ServerFnError>,
    select: WriteSignal<Option<PreRegion>>,
) -> impl IntoView {
    let mut pre_regions: Vec<PreRegion> = match pre_regions {
        Ok(pr) => pr,
        Err(err) => return view! {<p>"Something's wrong:" {err.to_string()}</p>}.into_view(),
    };

    let extent: (f32, f32) = pre_regions
        .iter()
        .fold(Point::new(0., 0.), |p_max, pr| {
            pr.shape.points().iter().fold(p_max, |mut p_max, pt| {
                let (x0, y0) = p_max.get();
                let (x1, y1) = pt.get();
                p_max.move_abs(f32::max(x0, x1), f32::max(y0, y1));
                p_max
            })
        })
        .get();

    pre_regions.sort_by(|pr1, pr2| pr1.has_base.cmp(&pr2.has_base));

    // TODO: optimize all the cloning (probably not a huge deal)
    let show = |pr: PreRegion| {
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
                {pre_regions.into_iter().map(show).collect_view()}
                <rect x=0 y=0 width=extent.0 height=extent.1 stroke="black" fill="none" />
            </svg>
        </div>
    }
    .into_view()
}
