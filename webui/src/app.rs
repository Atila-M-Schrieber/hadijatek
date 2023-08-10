use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod lang {
    use leptos::*;

    #[derive(Debug, Clone, Copy)]
    pub enum Language {
        Hungarian,
        English,
    }

    #[component]
    pub fn Lang<S>(cx: Scope, hu: S, en: S) -> impl IntoView
    where
        S: ToString + 'static,
    {
        use Language::*;
        let lang = expect_context::<ReadSignal<Language>>(cx);

        let text = move || match lang() {
            Hungarian => hu.to_string(),
            English => en.to_string(),
        };

        view! {cx,
            <div>{text}</div>
        }
    }
}

use lang::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    let (lang, set_lang) = create_signal(cx, Language::Hungarian);

    provide_context(cx, lang);
    provide_context(cx, set_lang);

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/webui.css"/>

        // sets the document title
        <Title text="Hadijáték"/>


        // content for this welcome page
        <Router fallback=|cx| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { cx,
                <ErrorTemplate outside_errors/>
            }
            .into_view(cx)
        }>
            <nav class="navbar">
                <NavBar/>
            </nav>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                </Routes>
            </main>
        </Router>
    }
}

/// The navigation bar - Home, Games (dropdown), right - Login/User
#[component]
fn NavBar(cx: Scope) -> impl IntoView {
    let set_lang = expect_context::<WriteSignal<Language>>(cx);

    view! {cx,
        <div class="container">
            <div class="left-section">
            <a href="/" class="logo">Hadijáték</a>
            <ul class="nav-list">
                <li class="dropdown">
                    <a href="#"><Lang hu="Játékok" en="Games"/></a>
                    <div class="dropdown-content">
                        <a href="/game1">Game 1</a>
                        <a href="/game2">Game 2</a>
                    </div>
                </li>
            </ul>
            </div>
            <div class="right-section">
            <ul class="nav-list">
                <li class="dropdown">
                    <a href="#"><Lang hu="Nyelv" en="Language"/></a>
                    <div class="dropdown-content">
                        <a on:click=move |_| set_lang(Language::Hungarian)>
                            <Lang hu="Magyar" en="Hungarian"/>
                        </a>
                        <a on:click=move |_| set_lang(Language::English)>
                            <Lang hu="Angol" en="English"/>
                        </a>
                    </div>
                </li>
                <li class="right"><a href="/login">Login/User</a></li>
            </ul>
            </div>
        </div>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(cx, 0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! { cx,
        <h1>"Welcome to Hadijáték!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}
