//! Pages that come directly from App
use leptos::ev::Event;
use leptos::*;
use leptos_router::*;

use crate::auth::*;
use crate::components::*;
use crate::error::*;
use crate::lang::*;

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Welcome to Hadijáték!"</h1>
        <button on:click=on_click><Lang hu="Nyomjá'meg" en="Click Me"/>": " {count}</button>
    }
}

/// The login page
#[component]
pub fn LoginPage(login: Action<Login, Result<(), ServerFnError>>) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (live_password, set_live_password) = create_signal(String::new());

    let set_name = move |ev: Event| set_name(event_target_value(&ev));
    let set_live_pw = move |ev: Event| {
        set_live_password(event_target_value(&ev));
    };

    let disable_submit = move || name().is_empty() || live_password().is_empty();

    view! {
        <div class="login-container">
            <h2><Lang hu="Bejelentkezés" en="Login"/></h2>
            <UserErrorBoundary action=login />
            <ActionForm action=login>
                <Input name="username" on:input=set_name >
                    <Lang hu="Felhasználónév" en="Username"/>
                </Input>
                <Input name="" on:input=set_live_pw password=true >
                    <Lang hu="Jelszó" en="Password"/>
                </Input>
                <RememberMe/>
                <Submit disable=disable_submit >
                        <Lang hu="Bejelentkezés" en="Log in"/>
                </Submit>
            </ActionForm>
            <p class="signup-text"><a href="/signup">
                <Lang hu="Regisztráció" en="Sign up"/>
            </a></p>
        </div>
        <Transition fallback=||()>
        <ErrorBoundary
            fallback=move |err|
                view!{
                        {move || err.get()
                            .into_iter()
                            .map(|(_, e)| view! {<p>{e.to_string()}</p>})
                            .collect_view()
                        }
                }
        >
        <p>{move || with_user(|user| user.username.clone())}</p>
        </ErrorBoundary>
        </Transition>
    }
}

/// The login page
#[component]
pub fn SignupPage(signup: Action<Signup, Result<(), ServerFnError>>) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());

    let set_name = move |ev: Event| set_name(event_target_value(&ev));

    let MIN_PW_LEN: usize = 6;

    let (password, set_password) = create_signal(String::new());
    let (live_password, set_live_password) = create_signal(String::new());
    let (live_password_confirm, set_live_password_confirm) = create_signal(String::new());

    let set_pw = move |ev: Event| {
        set_password(event_target_value(&ev));
    };
    let set_live_pw = move |ev: Event| {
        set_live_password(event_target_value(&ev));
    };
    let set_live_pw_cnf = move |ev: Event| {
        set_live_password_confirm(event_target_value(&ev));
    };

    let invalid_name = move || name() != name().trim();
    let valid_pw_len = move || live_password().len() >= MIN_PW_LEN;
    let valid_pw_chars = move || {
        !live_password()
            .chars()
            .any(|c| c.is_whitespace() || !c.is_ascii())
    };
    let valid_pw = move || valid_pw_len() && valid_pw_chars();
    let matching_pw = move || live_password() == live_password_confirm();

    let pw_strength = move || {
        let unit = match live_password().len() {
            0..=4 => ("Hajó", "Ship"),
            5..=7 => ("Tank", "Tank"),
            8..=10 => ("Repülő", "Plane"),
            11..=13 => ("Szupertank", "Supertank"),
            14..=16 => ("Tengeralattjáró", "Submarine"),
            _ => (
                "Tüzérség, \"A Csata Királynője\" (vagy a jelszavaké..)",
                "Artillery, \"The Queen of Battle\" (or of passwords..)",
            ),
        };
        view! {
            <Lang hu="Jelszó erőssége" en="Password strength" />": "<Lang hu=unit.0 en=unit.1 />
        }
    };

    let disable_submit = move || {
        invalid_name()
            || !valid_pw()
            || !matching_pw()
            || name().is_empty()
            || password().is_empty()
    };

    let name_problems = move || {
        view! {
            <Show when=invalid_name fallback=||()>
                <Alert header="">
                    <Lang hu="Név nem kezdődhet vagy végződhet szóközzel!"
                        en="Name must not start or end with whitespace!"/>
                </Alert>
            </Show>
        }
    };

    let pw_problems = move || {
        view! {
                <Show
                    when={move || !password().is_empty() && !valid_pw_len()}
                    fallback=||()
                >
                    <Alert header="">
                        <Lang hu="Túl rövid a jelszó!"
                            en="Password is too short!"/>
                    </Alert>
                </Show>
                <Show
                    when={move || !valid_pw_chars()}
                    fallback=||()
                >
                    <Alert header="">
                        <Lang hu="A jelszó nem tartalmazhat szóközt, illetve ASCII-n kívüli karaktert! (Pl. ékezetet)"
                            en="The password may not contain whitespace, or non-ASCII characters! (Eg. accents)"/>
                    </Alert>
                </Show>
        }
    };

    let pw_cnf_problems = move || {
        view! {
                <Show
                    when={move || !live_password_confirm().is_empty() && !matching_pw()}
                    fallback=||()
                >
                    <Alert header="">
                        <Lang hu="Nem egyeznek meg a jelszavak!"
                            en="Passwords do not match!"/>
                    </Alert>
                </Show>
        }
    };

    view! {
        <div class="login-container">
            <h2><Lang hu="Regisztráció" en="Signup"/></h2>
            <UserErrorBoundary action=signup />
            <ActionForm action=signup>
                <Input name="username" on:input=set_name >
                    <Lang hu="Felhasználónév" en="Username"/>
                </Input>
                {name_problems}
                <Input name="" on:change=set_pw on:input=set_live_pw password=true >
                    <Lang hu="Jelszó" en="Password"/>
                </Input>
                {pw_problems}
                <div class="pw-strength">{pw_strength}</div>
                <Input name="password_confirmation"
                    on:input=set_live_pw_cnf password=true >
                    <Lang hu="Jelszó újra" en="Password again"/>
                </Input>
                {pw_cnf_problems}
                <RememberMe/>
                <Submit disable=disable_submit>
                    <Lang hu="Regisztráció" en="Sign up"/>
                </Submit>
            </ActionForm>
        </div>
        <Transition fallback=||()>
        <ErrorBoundary
            fallback=move |err|
                view!{
                        {move || err.get()
                            .into_iter()
                            .map(|(_, e)| view! {<p>{e.to_string()}</p>})
                            .collect_view()
                        }
                }
        >
        <p>{move || with_user(|user| user.username.clone())}</p>
        </ErrorBoundary>
        </Transition>
    }
}

/// The admin settings page
#[component]
pub fn SettingsPage() -> impl IntoView {
    let user_role = move || with_user(|user| user.role).ok();
    let is_admin = move || user_role() == Some(UserRole::Admin);

    view! {
        <Show when=is_admin fallback=RegularSettingsPage>
            <AdminSettingsPage/>
        </Show>
    }
}

/// The regular settings page
#[component]
pub fn RegularSettingsPage() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Welcome to Regular Settings!"</h1>
        <button on:click=on_click><Lang hu="Nyomjá'meg" en="Click Me"/>": " {count}</button>
    }
}

/// The admin settings page
#[component]
pub fn AdminSettingsPage() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Welcome to Admin Settings!"</h1>
        <button on:click=on_click><Lang hu="Nyomjá'meg" en="Click Me"/>": " {count}</button>
    }
}
