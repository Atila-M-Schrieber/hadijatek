use leptos::*;
use leptos_router::*;

use crate::lang::*;

mod create;
pub use create::*;

#[component]
pub fn MapsPage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Welcome to the Maps Page!"</h1>
        <button on:click=on_click><Lang hu="Nyomj meg" en="Click Me"/>": " {count}</button>
        <Outlet/>
    }
}

#[component]
pub fn MapPage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Map here"</h1>
        <button on:click=on_click><Lang hu="Nyomjá'meg" en="Click Me"/>": " {count}</button>
    }
}

#[component]
pub fn NoMapPage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"No map :<"</h1>
        <button on:click=on_click><Lang hu="Nyomjá'meg" en="Click Me"/>": " {count}</button>
    }
}
