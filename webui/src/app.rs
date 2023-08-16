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
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let (lang, set_lang) = create_signal(Language::Hungarian);
    provide_context(lang);
    provide_context(set_lang);

    // Check if there is an existing session, if not, login as guest
    /*     let user = create_resource(move || session_login);
    let (user, set_user) =
        create_signal(user.with(|user| user.clone().unwrap_or_default())); */
    // let (user, set_user) = create_signal(User::default());
    // provide_context(user);

    let login = create_server_action::<Login>();
    let logout = create_server_action::<Logout>();
    let signup = create_server_action::<Signup>();

    let user = create_resource(
        move || {
            (
                login.version().get(),
                logout.version().get(),
                signup.version().get(),
            )
        },
        move |_| get_user(),
    );

    provide_context(user);

    // The navigation bar - Home, Games (dropdown), right - Login/User
    let nav_bar = view! {
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


        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/webui.css"/>

        // sets the document title
        <Title text="Hadijáték"/>

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <nav class="navbar">
                {nav_bar}
            </nav>
            <main>
                <Routes>
                    <Route path="/" view=HomePage/>
                    <Route path="/login" view=move || view!{<LoginPage login=login/>}/>
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
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Welcome to Hadijáték!"</h1>
        <button on:click=on_click><Lang hu="Nyomjá'meg" en="Click Me"/>": " {count}</button>
    }
}
