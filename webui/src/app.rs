use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod auth;
mod lang;

use auth::*;
use lang::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    let (lang, set_lang) = create_signal(cx, Language::Hungarian);
    provide_context(cx, lang);
    provide_context(cx, set_lang);

    // Check if there is an existing session, if not, login as guest
    /*     let user = create_resource(cx, move || cx, session_login);
    let (user, set_user) =
        create_signal(cx, user.with(cx, |user| user.clone().unwrap_or_default())); */
    // let (user, set_user) = create_signal(cx, User::default());
    // provide_context(cx, user);

    let login = create_server_action::<Login>(cx);
    let logout = create_server_action::<Logout>(cx);

    let user = create_resource(
        cx,
        move || (login.value().get(), logout.version().get()),
        move |(maybeuser, _)| {
            dbg!(&maybeuser);
            session_login(cx, maybeuser)
        },
    );

    provide_context(cx, user);

    // The navigation bar - Home, Games (dropdown), right - Login/User
    let nav_bar = view! {cx,
        <div class="container">
            <div class="left-section">
            <a href="/" class="logo">Hadijáték</a>
            <ul class="nav-list">
                <li>
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
                <li>
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
                <li><UserButton user=user logout=logout/></li>
            </ul>
            </div>
        </div>
    };

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
                {nav_bar}
            </nav>
            <main>
                <Routes>
                    <Route path="/" view=HomePage/>
                    <Route path="/login" view=move |cx| view!{cx, <LoginPage login=login/>}/>
                </Routes>
            </main>
        </Router>
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
        <button on:click=on_click><Lang hu="Nyomjá'meg he" en="Click Me"/>": " {count}</button>
    }
}
