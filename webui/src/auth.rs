use std::{error::Error, fmt::Display};

use crate::app::lang::*;
use cfg_if::cfg_if;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

cfg_if! { if #[cfg(feature = "ssr")] {
    // use sqlx::SqlitePool;
    use surrealdb::{Surreal};
    use surrealdb::engine::remote::ws::Client;
    use axum_session_auth::{SessionSurrealPool, Authentication};
    use bcrypt::{hash, verify, DEFAULT_COST};
    pub type AuthSession = axum_session_auth::AuthSession<User, String, SessionSurrealPool<Client>, Surreal<Client>>;
}}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Guest,
    Regular,
    Admin,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub role: UserRole,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: "guest".to_string(),
            username: "Guest".into(),
            password: "".into(),
            role: UserRole::Regular,
        }
    }
}

cfg_if! { if #[cfg(feature = "ssr")] {
    use async_trait::async_trait;

    impl User {
        pub async fn get(id: String, db: &Surreal<Client>) -> Option<Self> {
            db.use_ns("hadijatek").use_db("auth").await.ok()?;

            db.select(("user", id)).await.ok()
        }

        pub async fn get_from_username(name: String, db: &Surreal<Client>) -> Option<Self> {
            db.use_ns("hadijatek").use_db("auth").await.ok()?;

            let mut result = db.query("SELECT * FROM user WHERE username = $username")
                .bind(("username", name))
                .await.ok()?;

            result.take(0).ok().and_then(|u|u)
        }
    }

    #[async_trait]
    impl Authentication<User, String, Surreal<Client>> for User {
        async fn load_user(
            userid: String,
            pool: Option<&Surreal<Client>>
        ) -> Result<User, anyhow::Error> {
            let pool = pool.unwrap();

            User::get(userid, pool)
                .await
                .ok_or_else(|| anyhow::anyhow!("Cannot get user"))
        }

        fn is_authenticated(&self) -> bool {
            true
        }

        fn is_active(&self) -> bool {
            true
        }

        fn is_anonymous(&self) -> bool {
            false
        }
    }

    pub fn auth(cx: Scope) -> Result<AuthSession, ServerFnError> {
        use_context::<AuthSession>(cx)
            .ok_or_else(|| ServerFnError::ServerError("Auth session missing.".into()))
    }

    pub fn db(cx: Scope) -> Result<Surreal<Client>, ServerFnError> {
        use_context::<Surreal<Client>>(cx)
            .ok_or_else(|| ServerFnError::ServerError("Pool missing.".into()))
    }
}}

#[server(GetUser, "/api")]
pub async fn get_user(cx: Scope) -> Result<Option<User>, ServerFnError> {
    let auth = auth(cx)?;

    Ok(auth.current_user)
}

#[server(Login, "/api")]
pub async fn login(
    cx: Scope,
    username: String,
    password: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    let db = db(cx)?;
    let auth = auth(cx)?;

    let user: User = User::get_from_username(username, &db)
        .await
        .ok_or_else(|| ServerFnError::ServerError("User does not exist.".into()))?;

    match verify(password, &user.password)? {
        true => {
            auth.login_user(user.id);
            auth.remember_user(remember.is_some());
            leptos_axum::redirect(cx, "/");
            Ok(())
        }
        false => Err(ServerFnError::ServerError(
            "Password does not match.".to_string(),
        )),
    }
}

#[server(Signup, "/api")]
pub async fn signup(
    cx: Scope,
    username: String,
    password: String,
    password_confirmation: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    let db = db(cx)?;
    let auth = auth(cx)?;

    if password != password_confirmation {
        return Err(ServerFnError::ServerError(
            "Passwords did not match.".to_string(),
        ));
    }

    let password_hashed = hash(password, DEFAULT_COST).unwrap();

    let id = surrealdb::sql::Id::rand().to_raw();

    db.create("user")
        .content(User {
            id,
            username: username.clone(),
            password: password_hashed,
            role: UserRole::Regular,
        })
        .await?;

    // this is clever, should prevent identical usernames
    let user = User::get_from_username(username, &db)
        .await
        .ok_or_else(|| ServerFnError::ServerError("Signup failed: User does not exist.".into()))?;

    auth.login_user(user.id);
    auth.remember_user(remember.is_some());

    leptos_axum::redirect(cx, "/");

    Ok(())
}

#[server(Logout, "/api")]
pub async fn logout(cx: Scope) -> Result<(), ServerFnError> {
    let auth = auth(cx)?;

    auth.logout_user();
    leptos_axum::redirect(cx, "/");

    Ok(())
}

type UserResource = Resource<(usize, usize, usize), Result<Option<User>, ServerFnError>>;

#[derive(Debug)]
pub enum UserError {
    Server(ServerFnError),
    GuestUser,
    NoneErr,
}

impl Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::Server(err) => write!(f, "{err}"),
            err => write!(f, "{err:?}"),
        }
    }
}

impl Error for UserError {}

pub fn with_user<F, T>(cx: Scope, closure: F) -> Result<T, UserError>
where
    F: Fn(&User) -> T,
{
    let user = expect_context::<UserResource>(cx);
    use UserError::*;
    user.with(cx, |user| match user {
        Ok(Some(user)) => Ok(closure(user)),
        Ok(None) => Err(GuestUser),
        Err(err) => Err(Server(err.clone())),
    })
    .ok_or(NoneErr)
    .and_then(|u| u)
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
        <ErrorBoundary
            fallback=move |_,_| view!{cx,
                <a href="/login"><Lang hu="Bejelentkezés" en="Log in"/></a>}
        >
        {
            move || { with_user(cx, |user| {
                let username = user.username.clone();
                dbg!(&username);
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
            })}
        }
        </ErrorBoundary>
        </Transition>
    }
}

/// The login page
#[component]
pub fn LoginPage(cx: Scope, login: Action<Login, Result<(), ServerFnError>>) -> impl IntoView {
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
        <p>{move || with_user(cx, |user| user.username.clone())}</p>
    }
}
