use crate::error::*;
use crate::lang::*;
use cfg_if::cfg_if;
use leptos::*;
use serde::{Deserialize, Serialize};

cfg_if! { if #[cfg(feature = "ssr")] {
    use surrealdb::Surreal;
    use surrealdb::sql::Value;
    use surrealdb::sql::Thing;
    use surrealdb::engine::remote::ws::Client;
    use axum_session_auth::{SessionSurrealPool, Authentication};
    use bcrypt::{hash, verify, DEFAULT_COST};
    pub type AuthSession = axum_session_auth::AuthSession<User, String, SessionSurrealPool<Client>, Surreal<Client>>;
}}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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


    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct SurrealUser {
        pub username: String,
        pub password: String,
        pub role: UserRole,
    }

    impl From<User> for SurrealUser {
        fn from(user: User) -> SurrealUser {
            SurrealUser {
                username: user.username,
                password: user.password,
                role: user.role,
            }
        }
    }

    impl User {
        pub fn from_surreal_user(s_user: SurrealUser, id: String) -> Self {
            User {
                id,
                username: s_user.username,
                password: s_user.password,
                role: s_user.role,
            }
        }

        pub async fn get(id: String, db: &Surreal<Client>) -> Option<Self> {
            db.use_ns("hadijatek").use_db("auth").await.ok()?;

            let s_user = db.select(("user", &id)).await.ok()?;

            Some(User::from_surreal_user(s_user, id))
        }

        pub async fn get_from_username(name: String, db: &Surreal<Client>) -> Option<Self> {
            db.use_ns("hadijatek").use_db("auth").await.ok()?;

            let mut id = db
                .query("SELECT id FROM user WHERE username = $username")
                .bind(("username", &name))
                .await.ok()?;
            // log!("reached query: {id:#?}");
            let id: Option<Thing> = id.take("id").ok()?;
            // log!("reached opt<string>: {id:?}");
            let id: String = Value::from(id?).to_string()[5..].to_string();
            // log!("reached string: {id:?}");
            Self::get(id, db).await
        }

        pub async fn name_taken(name: &str, db: &Surreal<Client>) -> Result<bool, surrealdb::Error> {
            db.use_ns("hadijatek").use_db("auth").await?;

            let mut id = db
                .query("SELECT id FROM user WHERE username = $username")
                .bind(("username", &name))
                .await?;

            let id: Option<Thing> = id.take("id")?;

            Ok(id.is_some())
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

    pub fn auth() -> Result<AuthSession, ServerFnError> {
        use_context::<AuthSession>()
            .ok_or_else(|| ServerFnError::ServerError("Auth session missing.".into()))
    }

    pub fn db() -> Result<Surreal<Client>, ServerFnError> {
        use_context::<Surreal<Client>>()
            .ok_or_else(|| ServerFnError::ServerError("DB missing.".into()))
    }
}}

#[server(GetUser, "/api")]
pub async fn get_user() -> Result<Option<User>, ServerFnError> {
    let auth = auth()?;

    Ok(auth.current_user)
}

#[server(Login, "/api")]
pub async fn login(
    username: String,
    password: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    let db = db()?;
    let auth = auth()?;

    let user: User = User::get_from_username(username, &db)
        .await
        .ok_or_else(|| ServerFnError::ServerError("NO_USER: User does not exist.".into()))?;

    match verify(password, &user.password)? {
        true => {
            auth.login_user(user.id);
            auth.remember_user(remember.is_some());
            leptos_axum::redirect("/");
            Ok(())
        }
        false => Err(ServerFnError::ServerError(
            "BAD_PASSWORD: Password does not match.".to_string(),
        )),
    }
}

#[server(Signup, "/api")]
pub async fn signup(
    username: String,
    password: String,
    password_confirmation: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    let db = db()?;
    let auth = auth()?;

    const MIN_PW_LEN: usize = 6;

    if password.len() < MIN_PW_LEN {
        return Err(ServerFnError::ServerError(
            "SHORT_PASSWORD: Password is too short.".to_string(),
        ));
    }
    if password.chars().any(|c| c.is_whitespace() || !c.is_ascii()) {
        return Err(ServerFnError::ServerError(
            "INVALID_PASSWORD: Password contains whitespace, or non-ASCII chars.".to_string(),
        ));
    }

    let user_exists = User::name_taken(&username, &db).await?;
    if user_exists {
        return Err(ServerFnError::ServerError(
            "TAKEN_NAME: User already exists!".to_string(),
        ));
    } else {
        log!("User {} does not yet exist, signing up...", &username);
    }

    if password != password_confirmation {
        return Err(ServerFnError::ServerError(
            "UNCONFIRMED_PASSWORD: Passwords did not match.".to_string(),
        ));
    }

    let password_hashed = hash(password, DEFAULT_COST).unwrap();

    let id = surrealdb::sql::Id::rand().to_raw();

    let s_user: Result<SurrealUser, surrealdb::Error> = db
        .create(("user", &id))
        .content(SurrealUser {
            username: username.clone(),
            password: password_hashed,
            role: UserRole::Regular,
        })
        .await;

    if let Ok(s_user) = s_user {
        let user: User = User::from_surreal_user(s_user, id);
        auth.login_user(user.id);
        auth.remember_user(remember.is_some());
        leptos_axum::redirect("/");
        Ok(())
    } else {
        Err(ServerFnError::ServerError(
            "User_ID already exists, contact administrator!".to_string(),
        ))
    }
}

#[server(Logout, "/api")]
pub async fn logout() -> Result<(), ServerFnError> {
    let auth = auth()?;

    auth.logout_user();
    leptos_axum::redirect("/");

    Ok(())
}

type UserResource = Resource<(usize, usize, usize), Result<Option<User>, ServerFnError>>;

/// Simplifies working with the user resource
pub fn with_user<F, T>(closure: F) -> Result<T, UserError>
where
    F: Fn(&User) -> T,
{
    let user = expect_context::<UserResource>();
    use UserError::*;
    user.with(|user| match user {
        Ok(Some(user)) => Ok(closure(user)),
        Ok(None) => Err(GuestUser),
        Err(err) => Err(err.clone().into()),
    })
    .ok_or(NoneErr)
    .and_then(|u| u)
}

/// The login button
#[component]
pub fn UserButton(logout: Action<Logout, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <Transition fallback=move||
            view!{
                <Lang hu="Bejelentkezés..." en="Loging in..."/>
            }
        >
        <ErrorBoundary
            fallback=move |_err| {
                view!{
                    <a href="/login"><Lang hu="Bejelentkezés" en="Log in"/></a>}
            }
        >
        {
            move || { with_user(|user| {
                view!{
                    <a href="/settings">
                        <Lang hu="Beállítások" en="Settings"/>" ("{user.username.clone()}")"
                    </a>
                    <div class="dropdown-content">
                        <a href="#" on:click=move|_|logout.dispatch(Logout {})>
                            <Lang hu="Kijelentkezés" en="Log out"/>
                        </a>
                    </div>
                }
            })}
        }
        </ErrorBoundary>
        </Transition>
    }
}
