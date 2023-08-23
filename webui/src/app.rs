use crate::error::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod game;
mod pages;

use crate::auth::*;
use crate::lang::*;
use game::*;
use pages::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let (lang, set_lang) = create_signal(Language::Hungarian);
    provide_context(lang);
    provide_context(set_lang);

    let login = create_server_action::<Login>();
    let logout = create_server_action::<Logout>();
    let signup = create_server_action::<Signup>();
    let change_info = create_server_action::<ChangeUserInfo>();

    provide_context(change_info);

    let user = create_resource(
        move || {
            (
                login.version().get(),
                logout.version().get(),
                signup.version().get(),
                change_info.version().get(),
            )
        },
        move |_| get_user(),
    );

    provide_context(user);

    let logged_in = move || with_user(|_| ()).is_ok();
    // let user_role = move || with_user(|user| user.role).ok();
    // let is_admin = move || user_role() == Some(UserRole::Admin);
    // let is_regular = move || user_role() == Some(UserRole::Regular);
    let is_guest = move || !logged_in();

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
                <li><UserButton logout=logout/></li>
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
                    <ProtectedRoute path="/login" redirect_path="/" condition=is_guest
                        view=move || view!{<LoginPage login=login/>}/>
                    <ProtectedRoute path="/signup" redirect_path="/" condition=is_guest
                        view=move || view!{<SignupPage signup=signup/>}/>
                    <ProtectedRoute path="/settings" redirect_path="/" condition=logged_in
                        view=SettingsPage />
                    <Route path="/game" view=GamesPage>
                        // <Route path="no-guests" view=NoGuestsPage/>
                        <Route path=":game" view=GamePage/> // redirect to no-guests if needed
                        <Route path="" view=NoGamePage/>
                    </Route>
                </Routes>
            </main>
        </Router>
    }
}
