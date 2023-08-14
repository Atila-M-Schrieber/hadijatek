use cfg_if::cfg_if;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use super::lang::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Guest,
    Regular,
    Admin,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    password: String,
    pub role: UserRole,
}

type UserResource =
    Resource<(Option<Result<User, ServerFnError>>, usize), Result<User, ServerFnError>>;

impl User {
    pub fn default() -> Self {
        User {
            username: "Guest".to_string(),
            password: String::new(),
            role: UserRole::Guest,
        }
    }
}

impl Default for User {
    fn default() -> Self {
        User::default()
    }
}

cfg_if! { if #[cfg(feature = "ssr")] {
    async fn thing(){
        todo!()
    }
    async fn verify_session(session: String) -> Option<User> {
        println!("session login attempted");
        Some(User{username: "admin".to_string(), password: String::new(), role: UserRole::Admin})
    }
}}

#[server(Login, "/api")]
pub async fn login(cx: Scope, username: String, password: String) -> Result<User, ServerFnError> {
    use axum::http::header;
    use axum_extra::extract::cookie::Cookie;
    use leptos_axum::{redirect, ResponseOptions};

    println!("logging in...");

    let mut user = User::default();

    if !username.is_empty() && &username != "Guest" {
        let response = expect_context::<ResponseOptions>(cx);

        println!("user found...");
        let mut cookie = Cookie::new("session", "lmao");
        cookie.set_http_only(Some(true));
        if let Ok(cookie) = header::HeaderValue::from_str(&cookie.to_string()) {
            response.insert_header(header::SET_COOKIE, cookie);
        }

        redirect(cx, "/");

        user = User {
            username,
            password: "lmao".to_string(), // return token as "password"
            role: UserRole::Regular,
        };
    }
    dbg!(&user);
    Ok(user)
}

#[server(Logout, "/api")]
pub async fn logout(cx: Scope) -> Result<(), ServerFnError> {
    use axum::http::header;
    use axum_extra::extract::cookie::Cookie;
    use cookie::time::Duration;
    use leptos_axum::extract;
    use leptos_axum::ResponseOptions;

    let response = expect_context::<ResponseOptions>(cx);

    let mut cookie = Cookie::new("session", "");
    cookie.set_max_age(Duration::ZERO);
    if let Ok(cookie) = header::HeaderValue::from_str(&cookie.to_string()) {
        response.insert_header(header::SET_COOKIE, cookie);
    }
    Ok(())
}

#[server(SessionLogin, "/api")]
pub async fn session_login(
    cx: Scope,
    user: Option<Result<User, ServerFnError>>,
) -> Result<User, ServerFnError> {
    use axum_extra::extract::cookie::CookieJar;
    use leptos_axum::extract;

    dbg!(&user);

    let session = if let Some(Ok(user)) = user {
        println!("user found");
        Ok(Some(user.password))
    } else {
        extract(cx, |cookie_jar: CookieJar| async move {
            cookie_jar.get("session").map(|s| s.value().to_owned())
        })
        .await
    };

    println!("session logging in...");
    dbg!(&session);

    let mut session_user = User::default();

    if let Ok(Some(session)) = session {
        if let Some(user) = verify_session(session).await {
            session_user = user;
        }
    }

    dbg!(&session_user);

    // Err(ServerFnError::ServerError("No user found".to_string()))
    Ok(session_user)
}

/// The login button
#[component]
pub fn UserButton(
    cx: Scope,
    user: UserResource,
    logout: Action<Logout, Result<(), ServerFnError>>,
) -> impl IntoView {
    view! {cx,
        <Transition fallback=move||
            view!{cx,
                <Lang hu="Bejelentkezés..." en="Loging in..."/>
            }
        >
        {
            move || { user.with(cx, |u| u.clone().map(|user| {
                let username = user.username.clone();
                dbg!(&username);
                if username != "Guest" {
                    view!{cx,
                        <a href="/settings">Settings ({username})</a>
                        <div class="dropdown-content">
                            <ActionForm action=logout>
                                <button type="submit">
                                    <Lang hu="Kijelentkezés" en="Log out"/>
                                </button>
                            </ActionForm>
                        </div>
                    }.into_view(cx)
                } else {
                    view!{cx,
                        <a href="/login"><Lang hu="Bejelentkezés" en="Log in"/></a>
                    }.into_view(cx)
                }
            }))}
        }
        </Transition>
    }
}

/// The login page
#[component]
pub fn LoginPage(cx: Scope, login: Action<Login, Result<User, ServerFnError>>) -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(cx, 0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    let user = expect_context::<UserResource>(cx);

    view! { cx,
        <div class="login-container">
            <h2><Lang hu="Bejelentkezés" en="Login"/></h2>
            <ActionForm action=login>
                <div class="input-group">
                    <label for="username"><Lang hu="Felhasználónév" en="Username"/></label>
                    <input type="text" id="username" name="username" required/>
                </div>
                <div class="input-group">
                    <label for="password"><Lang hu="Jelszó" en="Password"/></label>
                    <input type="password" id="password" name="password" required/>
                </div>
                <div class="input-group">
                    <button type="submit"><Lang hu="Bejelentkezés" en="Log in"/></button>
                </div>
            </ActionForm>
            <p class="signup-text"><a href="/signup">
                <Lang hu="Feljelentkezés" en="Sign up"/>
            </a></p>
        </div>
        <button on:click=on_click><Lang hu="Nyomjá'meg he" en="Click Me"/>": " {count}</button>
        <p>{move || user.with(cx, |u| u.clone().map(|u| u.username.clone()))}</p>
    }
}
