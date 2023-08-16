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
pub async fn login(username: String, password: String) -> Result<User, ServerFnError> {
    use axum::http::header;
    use axum_extra::extract::cookie::Cookie;
    use leptos_axum::{redirect, ResponseOptions};

    println!("logging in...");

    let mut user = User::default();

    if !username.is_empty() && &username != "Guest" {
        let response = expect_context::<ResponseOptions>();

        println!("user found...");
        let mut cookie = Cookie::new("session", "lmao");
        cookie.set_http_only(Some(true));
        if let Ok(cookie) = header::HeaderValue::from_str(&cookie.to_string()) {
            response.insert_header(header::SET_COOKIE, cookie);
        }

        redirect("/");

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
pub async fn logout() -> Result<(), ServerFnError> {
    use axum::http::header;
    use axum_extra::extract::cookie::Cookie;
    use cookie::time::Duration;
    use leptos_axum::extract;
    use leptos_axum::ResponseOptions;

    let response = expect_context::<ResponseOptions>();

    let mut cookie = Cookie::new("session", "");
    cookie.set_max_age(Duration::ZERO);
    if let Ok(cookie) = header::HeaderValue::from_str(&cookie.to_string()) {
        response.insert_header(header::SET_COOKIE, cookie);
    }
    Ok(())
}

#[server(SessionLogin, "/api")]
pub async fn session_login(
    
    user: Option<Result<User, ServerFnError>>,
) -> Result<User, ServerFnError> {
    use axum_extra::extract::cookie::CookieJar;
    use leptos_axum::extract;

    dbg!(&user);

    let session = if let Some(Ok(user)) = user {
        println!("user found");
        Ok(Some(user.password))
    } else {
        extract(|cookie_jar: CookieJar| async move {
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
