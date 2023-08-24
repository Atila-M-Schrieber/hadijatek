//! Pages that come directly from App

use chrono::offset::Local;
use chrono::offset::Utc;
use chrono::DateTime;
use leptos::ev::Event;
use leptos::*;
use leptos_router::*;

use crate::app::game::map::*;
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
    // Bool to see if it's been changed
    let (token, set_token) = create_signal((String::new(), false)); // prevents empty token warning
    let (live_token, set_live_token) = create_signal(String::new());

    let activate_token = move || set_token.update(|(_, active)| *active = true); // enable warning
    let set_token = move |ev: Event| set_token((event_target_value(&ev), true));
    let set_live_token = move |ev: Event| set_live_token(event_target_value(&ev));

    let (name, set_name) = create_signal(String::new());

    let set_name = move |ev: Event| {
        activate_token();
        set_name(event_target_value(&ev))
    };

    let MIN_PW_LEN: usize = 6;

    let (password, set_password) = create_signal(String::new());
    let (live_password, set_live_password) = create_signal(String::new());
    let (live_password_confirm, set_live_password_confirm) = create_signal(String::new());

    let set_pw = move |ev: Event| {
        set_password(event_target_value(&ev));
    };
    let set_live_pw = move |ev: Event| {
        activate_token();
        set_live_password(event_target_value(&ev));
    };
    let set_live_pw_cnf = move |ev: Event| {
        activate_token();
        set_live_password_confirm(event_target_value(&ev));
    };

    let empty_token = move || token().0.is_empty() && token().1;
    let bad_length_token = move || token().0.len() != 20 && token().1;
    let bad_live_length_token = move || live_token().len() != 20;
    let invalid_token = move || {
        live_token()
            .chars()
            .any(|c| !('a'..='z').contains(&c) && !('0'..='9').contains(&c))
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
    let wont_be_matching_pw = move || {
        let live_password_confirm = live_password_confirm();
        let len = live_password_confirm.len();
        let live_password = live_password();
        len > live_password.len() || &live_password[..len] != &live_password_confirm
    };

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
        empty_token()
            || bad_live_length_token()
            || invalid_token()
            || invalid_name()
            || !valid_pw()
            || !matching_pw()
            || name().is_empty()
            || password().is_empty()
    };

    let token_problems = move || {
        view! {
            <Show when=empty_token fallback=||()>
                <Alert header="" warning=true >
                    <Lang hu="Regisztrációhoz kell token! Ha nincs, kérj az adminisztrátortól!"
                        en="You may only sign up with a user creation token! You may ask the administrator to issue you a token."/>
                </Alert>
            </Show>
            <Show when=move || !empty_token() && bad_length_token() fallback=||()>
                <Alert header="">
                    <Lang hu="A token hossza 20 karakter!"
                        en="The token's length must be 20 characters!"/>
                </Alert>
            </Show>
            <Show when=invalid_token fallback=||()>
                <Alert header="">
                    <Lang hu="A token csak a-z közötti ékezet nélküli karaktereket, és 0-9 közötti karaktereket tartalmazhat!"
                        en="The token may only contain characters a-z or 0-9 (ASCII - no accents)!"/>
                </Alert>
            </Show>
        }
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
                    when={move || !live_password_confirm().is_empty() && wont_be_matching_pw()}
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
                <Input name="user_creation_token" on:change=set_token on:input=set_live_token >
                    <Lang hu="Regisztrációs token" en="User creation token"/>
                </Input>
                {token_problems}
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

/// The settings page - admin / regular
#[component]
pub fn SettingsPage() -> impl IntoView {
    let is_admin = move || with_user(|user| user.role) == Ok(UserRole::Admin);
    let logged_in = move || with_user(|_| ()).is_ok();

    view! {
        <Show when=logged_in fallback=||()>
            <Show when=is_admin fallback=RegularSettingsPage>
                <AdminSettingsPage/>
            </Show>
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

        <h2><Lang hu="Felhasználói beállítások" en="User settings"/></h2>
        <ErrorBoundary fallback=|_| "Some user error occurred" >
        {move || with_user(|user| view!{
            <UserSettings user=user.clone() />
        })}
        </ErrorBoundary>
    }
}

/// The admin settings page
#[component]
pub fn AdminSettingsPage() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    // User tokens
    let create_user_token = create_server_action::<CreateUserToken>();
    let delete_user_token = create_server_action::<DeleteUserToken>();

    let user_tokens = create_resource(
        move || {
            (
                create_user_token.version().get(),
                delete_user_token.version().get(),
            )
        },
        move |_| get_user_token_info(),
    );

    let list_user_tokens = move || -> Result<View, ServerFnError> {
        let mut tokens = user_tokens
            .read()
            .ok_or(ServerFnError::ServerError("bruh".into()))??;

        let used_tokens =
            if let Some(pos) = tokens.iter().position(|(_, consumer)| consumer.is_some()) {
                tokens.split_off(pos)
            } else {
                Vec::new()
            };

        let tokens_store = store_value(tokens);
        let used_tokens_store = store_value(used_tokens);

        let to_row = move |(token, consumer): UserCreationToken| {
            let consumer = store_value(consumer);
            let consumer = move || consumer.get_value();
            let token = store_value(token);
            let token = move || token.get_value();

            let class = if consumer().is_none() {
                "active"
            } else {
                "consumed"
            };

            let time =
                |t: DateTime<Utc>| t.with_timezone(&Local).format("%d/%m/%Y %H:%M").to_string();

            // copy-on-click
            let copy_token = move |_| {
                let window = window();
                let _written = window
                    .navigator()
                    .clipboard()
                    .expect("Clipboard not found")
                    .write_text(&token().token);
            };

            view! {
                <tr>
                    <td class=class title="Copy" on:click=copy_token >
                            {token().token}
                    </td>
                    <td>{time(token().created)}</td>
                    <td>{consumer().map(|(user, _)| user.username)}</td>
                    <td><Show when=move||consumer().is_none()
                            fallback=move||consumer().map(|(_, t)| time(t)) >
                            <ActionForm action=delete_user_token>
                                <input type="hidden" name="token" value=token().token />
                                <Submit disable=||false >
                                    <Lang hu="Törlés"
                                        en="Delete" />
                                </Submit>
                            </ActionForm>
                        </Show>
                    </td>
                </tr>
            }
        };

        Ok(view! {
            <Table items=tokens_store to_row=to_row >
                <th><Lang hu="Token" en="Token" /></th>
                <th><Lang hu="Létrehozás ideje" en="Creation Time" /></th>
                <th><Lang hu="" en="" /></th>
                <th><Lang hu="Törlés" en="Delete" /></th>
            </Table>
            <Table items=used_tokens_store to_row=to_row >
                <th><Lang hu="Token" en="Token" /></th>
                <th><Lang hu="Létrehozás ideje" en="Creation Time" /></th>
                <th><Lang hu="Felhasználó" en="User" /></th>
                <th><Lang hu="Felhasználás ideje" en="Consumption Time" /></th>
            </Table>
        }
        .into_view())
    };

    // Map tokens
    let create_map_token = create_server_action::<CreateMapToken>();
    let delete_map_token = create_server_action::<DeleteMapToken>();

    let map_tokens = create_resource(
        move || {
            (
                create_map_token.version().get(),
                delete_map_token.version().get(),
            )
        },
        move |_| get_map_token_info(),
    );

    let list_map_tokens = move || -> Result<View, ServerFnError> {
        let mut tokens = map_tokens
            .read()
            .ok_or(ServerFnError::ServerError("bruh".into()))??;

        let mut claimed_tokens = if let Some(pos) = tokens
            .iter()
            .position(|(_, _, consumer)| consumer.is_some())
        {
            tokens.split_off(pos)
        } else {
            Vec::new()
        };

        let used_tokens = if let Some(pos) = tokens
            .iter()
            .position(|(_, consumer, _)| consumer.is_some())
        {
            claimed_tokens.split_off(pos)
        } else {
            Vec::new()
        };

        let tokens_store = store_value(tokens);
        let used_tokens_store = store_value(used_tokens);

        let to_row = move |(token, map_consumer, user_consumer): MapCreationToken| {
            let token = store_value(token);
            let token = move || token.get_value();
            let map_consumer = store_value(map_consumer);
            let map_consumer = move || map_consumer.get_value();
            let user_consumer = store_value(user_consumer);
            let user_consumer = move || user_consumer.get_value();

            let class = if user_consumer().is_none() {
                "active"
            } else {
                "consumed"
            };

            let time =
                |t: DateTime<Utc>| t.with_timezone(&Local).format("%d/%m/%Y %H:%M").to_string();

            // copy-on-click
            let copy_token = move |_| {
                let window = window();
                let _written = window
                    .navigator()
                    .clipboard()
                    .expect("Clipboard not found")
                    .write_text(&token().token);
            };

            view! {
                <tr>
                    <td class=class title="Copy" on:click=copy_token >
                            {token().token}
                    </td>
                    <td>{time(token().created)}</td>
                    <td>{map_consumer().map(|(map, _)| map.0)}</td>
                    <td>{map_consumer().map(|(_, t)| time(t))}</td>
                    <td>{user_consumer().map(|(user, _)| user.username)}</td>
                    <td><Show when=move||user_consumer().is_none()
                            fallback=move||user_consumer().map(|(_, t)| time(t)) >
                            <ActionForm action=delete_map_token>
                                <input type="hidden" name="token" value=token().token />
                                <Submit disable=||false >
                                    <Lang hu="Törlés"
                                        en="Delete" />
                                </Submit>
                            </ActionForm>
                        </Show>
                    </td>
                </tr>
            }
        };

        Ok(view! {
            <Table items=tokens_store to_row=to_row >
                <th><Lang hu="Token" en="Token" /></th>
                <th><Lang hu="Létrehozás ideje" en="Creation Time" /></th>
                <th><Lang hu="" en="" /></th>
                <th><Lang hu="" en="" /></th>
                <th><Lang hu="" en="" /></th>
                <th><Lang hu="Törlés" en="Delete" /></th>
            </Table>
            <Table items=used_tokens_store to_row=to_row >
                <th><Lang hu="Token" en="Token" /></th>
                <th><Lang hu="Létrehozás ideje" en="Creation Time" /></th>
                <th><Lang hu="Térkép" en="Map" /></th>
                <th><Lang hu="Felhasználás ideje" en="Consumption Time" /></th>
                <th><Lang hu="Felhasználó" en="User" /></th>
                <th><Lang hu="Igénybevétel ideje" en="Claim Time" /></th>
            </Table>
            <Table items=used_tokens_store to_row=to_row >
                <th><Lang hu="Token" en="Token" /></th>
                <th><Lang hu="Létrehozás ideje" en="Creation Time" /></th>
                <th><Lang hu="Térkép" en="Map" /></th>
                <th><Lang hu="Felhasználás ideje" en="Consumption Time" /></th>
                <th><Lang hu="Felhasználó" en="User" /></th>
                <th><Lang hu="Igénybevétel ideje" en="Claim Time" /></th>
            </Table>
        }
        .into_view())
    };

    let (is_user_tokens_expanded, set_is_user_tokens_expanded) = create_signal(false);
    let (is_map_tokens_expanded, set_is_map_tokens_expanded) = create_signal(false);

    let expand_user_tokens = move |_| set_is_user_tokens_expanded.update(|i_e| *i_e = !*i_e);
    let expand_map_tokens = move |_| set_is_map_tokens_expanded.update(|i_e| *i_e = !*i_e);

    view! {
        <h1>"Welcome to Admin Settings!"</h1>
        <button on:click=on_click><Lang hu="Nyomjá'meg" en="Click Me"/>": " {count}</button>
        <div class="panel" class:expanded=is_user_tokens_expanded >
            <div class="panel-heading" on:click=expand_user_tokens >
                <span class="arrow-icon">">"</span>" "
                <Lang hu="Regisztrációs tokenek" en="User creation tokens"/>
            </div>
            <div class="panel-content" >
                <ActionForm action=create_user_token class="create-token-form" >
                    <Submit disable=||false >
                        <Lang hu="Új regisztrációs token létrehozása" en="Create signup token" />
                    </Submit>
                </ActionForm>
                <Transition fallback=move || view! {
                    <p><Lang hu="Tokenek betöltése.." en="Loading tokens..."/></p>
                }>
                    <ErrorBoundary fallback=|_| view!{<p>"Something's gone wrong :("</p>}>
                        {list_user_tokens}
                    </ErrorBoundary>
                </Transition>
            </div>
        </div>
        <div class="panel" class:expanded=is_map_tokens_expanded >
            <div class="panel-heading" on:click=expand_map_tokens >
                <span class="arrow-icon">">"</span>" "
                <Lang hu="Térkép-előállítási tokenek" en="Map creation tokens"/>
            </div>
            <div class="panel-content" >
                <ActionForm action=create_map_token class="create-token-form" >
                    <Submit disable=||false >
                        <Lang hu="Új térkép token létrehozása" en="Create map token" />
                    </Submit>
                </ActionForm>
                <Transition fallback=move || view! {
                    <p><Lang hu="Tokenek betöltése.." en="Loading tokens..."/></p>
                }>
                    <ErrorBoundary fallback=|_| view!{<p>"Something's gone wrong :("</p>}>
                        {list_map_tokens}
                    </ErrorBoundary>
                </Transition>
            </div>
        </div>
    }
}
