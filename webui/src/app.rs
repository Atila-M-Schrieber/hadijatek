use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod game;
pub mod lang;

use crate::auth::*;
use game::*;
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
    let signup = create_server_action::<Signup>(cx);

    let user = create_local_resource(
        cx,
        move || {
            (
                login.version().get(),
                logout.version().get(),
                signup.version().get(),
            )
        },
        move |_| get_user(cx),
    );

    provide_context(cx, user);

    // The navigation bar - Home, Games (dropdown), right - Login/User
    let nav_bar = view! {cx,
        <div class="container">
            <div class="left-section">
            <a href="/" class="logo">Hadijáték</a>
            <ul class="nav-list">
                <li>
                    <a href="/game"><Lang hu="Játékok" en="Games"/></a>
                    <div class="dropdown-content">
                        <a href="/game/test">test</a>
                        <a href="/game/game2">Game 2</a>
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
                    <Route path="/game" view=GamesPage>
                        <Route path=":game" view=GamePage/>
                        <Route path="" view=NoGamePage/>
                    </Route>
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
