use cfg_if::cfg_if;
pub mod app;
pub mod auth;
pub mod error;
pub mod fileserv;

cfg_if! { if #[cfg(feature = "ssr")] {
    use leptos::LeptosOptions;
    use surrealdb::Surreal;
    use surrealdb::engine::remote::ws::Client;
    use axum::extract::FromRef;

    /// This takes advantage of Axum's SubStates feature by deriving FromRef. This is the only way to have more than one
    /// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
    #[derive(FromRef, Debug, Clone)]
    pub struct AppState{
        pub leptos_options: LeptosOptions,
        pub db: Surreal<Client>
    }
}}

cfg_if! { if #[cfg(feature = "hydrate")] {
    use leptos::*;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::app::*;

    #[wasm_bindgen]
    pub fn hydrate() {
        // initializes logging using the `log` crate
        _ = console_log::init_with_level(log::Level::Debug);
        console_error_panic_hook::set_once();

        leptos::mount_to_body(move || {
            view! { <App/> }
        });
    }
}}
