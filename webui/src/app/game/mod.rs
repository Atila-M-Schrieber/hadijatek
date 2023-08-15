use leptos::{ev::MouseEvent, *};
use leptos_router::*;
use wasm_bindgen::JsCast;
use web_sys::SvgElement;

use super::lang::*;

#[component]
pub fn GamesPage(cx: Scope) -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(cx, 0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! { cx,
        <h1>"Welcome to the Games Page!"</h1>
        <button on:click=on_click><Lang hu="Nyomjá'meg he" en="Click Me"/>": " {count}</button>
        <Outlet/>
    }
}

#[component]
pub fn NoGamePage(cx: Scope) -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(cx, 0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! { cx,
        <h1>"No game :<"</h1>
        <button on:click=on_click><Lang hu="Nyomjá'meg he" en="Click Me"/>": " {count}</button>
    }
}

#[component]
pub fn GamePage(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);
    let game = move || params.with(|p| p.get("game").cloned());

    let (mouse_pos, set_mouse_pos) = create_signal(cx, (0, 0));

    let (from, set_from) = create_signal(cx, (0, 0));
    let (to, set_to) = create_signal(cx, (0, 0));

    let (bound, set_bound) = create_signal(cx, (true, true));

    let on_move = move |mouse: MouseEvent| {
        if let Some(Ok(Some(svg_element))) = mouse
            .target()
            .map(|t| t.dyn_into::<SvgElement>().map(|se| se.owner_svg_element()))
        {
            let pt = svg_element.create_svg_point();
            pt.set_x(mouse.client_x() as f32); // - rect.left() as f32);
            pt.set_y(mouse.client_y() as f32); //- rect.top() as f32);

            let inverted_matrix = svg_element.get_screen_ctm().map(|ctm| ctm.inverse());
            if let Some(Ok(inverted_matrix)) = inverted_matrix {
                let transformed_pt = pt.matrix_transform(&inverted_matrix);

                let x = transformed_pt.x();
                let y = transformed_pt.y();

                set_mouse_pos((x as i32, y as i32));
            } else {
                log!("Matrix trouble: {:?}", inverted_matrix);
            }
        } else {
            log!("Parent SVG element not found: {:?}", mouse.target());
        }

        if bound().0 {
            set_from(mouse_pos());
        }
        if bound().1 {
            set_to(mouse_pos());
        }
    };

    let on_click = move |_: MouseEvent| match bound() {
        (true, true) => {
            log!("From is set");
            set_from(mouse_pos());
            set_bound((false, true));
        }
        (false, true) => {
            log!("Both set");
            set_to(mouse_pos());
            set_bound((false, false));
        }
        (_, false) => {
            log!("Both unset, bound");
            set_bound((true, true));
        }
    };

    // let from_x = move || from.with(|n| n.map(|n| n.0));
    // let from_y = move || from.with(|n| n.map(|n| n.1));
    // let to_x = move || from.with(|n| n.map(|_| to().0));
    // let to_y = move || from.with(|n| n.map(|_| to().1));
    let from_x = move || from().0;
    let from_y = move || from().1;
    let to_x = move || to().0;
    let to_y = move || to().1;

    view! { cx,
        <h1>"Welcome to "{game}" Game Page!"</h1>
        <p>{move || format!("{:?}, {:?}, {:?}", mouse_pos(), from(), to())}</p>
        <div> // on:click=on_click on:mousemove=on_move>
        <svg viewBox="0 0 1000 500" xmlns="http://www.w3.org/2000/svg"
            on:click=on_click on:mousemove=on_move>
                <defs>
                    <marker id="arrow" viewBox="0 0 10 10" refX="10" refY="5"
                            markerWidth="10" markerHeight="10"
                            orient="auto-start-reverse">
                        <path d="M 0 0 L 10 5 L 0 10"
                            fill="none"
                            stroke-linejoin="arcs"
                            stroke="currentColor"
                            stroke-width="1" />
                    </marker>
                </defs>
            <rect width="100%" height="100%" fill="grey"/>
            <Show when=move||!bound.get().0 || !bound.get().1 fallback=|_|()>
            <line x1=from_x y1=from_y x2=to_x y2=to_y stroke="black" marker-end="url(#arrow)"/>
            </Show>
        </svg>
        </div>
    }
}
