use crate::error::*;
use cfg_if::cfg_if;
use leptos::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod token;

cfg_if! { if #[cfg(feature = "ssr")] {
    use self::token::consume_token;
    use surrealdb::Surreal;
    use surrealdb::sql::Value;
    use surrealdb::sql::Thing;
    use surrealdb::sql::Id;
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

        pub async fn get(id: &str, db: &Surreal<Client>) -> Option<Self> {
            db.use_ns("hadijatek").use_db("auth").await.ok()?;

            let s_user = db.select(("user", id)).await.ok()?;

            Some(User::from_surreal_user(s_user, id.to_string()))
        }

        pub async fn get_from_username(name: &str, db: &Surreal<Client>) -> Option<Self> {
            db.use_ns("hadijatek").use_db("auth").await.ok()?;

            let mut id = db
                .query("SELECT id FROM user WHERE username = $username")
                .bind(("username", name))
                .await.ok()?;
            // log!("reached query: {id:#?}");
            let id: Option<Thing> = id.take("id").ok()?;
            // log!("reached opt<string>: {id:?}");
            let id: String = Value::from(id?).to_string()[5..].to_string();
            // log!("reached string: {id:?}");
            Self::get(&id, db).await
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

            User::get(&userid, pool)
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

    let user: User = User::get_from_username(&username, &db)
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
    user_creation_token: String,
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
    }

    if password != password_confirmation {
        return Err(ServerFnError::ServerError(
            "UNCONFIRMED_PASSWORD: Passwords did not match.".to_string(),
        ));
    }

    let password_hashed = hash(password, DEFAULT_COST).unwrap();

    let id = Id::rand().to_raw();

    consume_token::<SurrealUser>("user_token", &user_creation_token, "user", &id, &db).await?;

    let s_user: Result<SurrealUser, surrealdb::Error> = db
        .create(("user", &id))
        .content(SurrealUser {
            username: username.clone(),
            password: password_hashed,
            role: UserRole::Regular,
        })
        .await;

    if s_user.is_ok() {
        auth.login_user(id);
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

#[server(ChangeUserInfo, "/api")]
pub async fn change_user_info(
    username: String,
    password: String,
    new_username: Option<String>,
    new_password: Option<String>,
    new_password_confirmation: Option<String>,
    as_admin: bool,
) -> Result<(), ServerFnError> {
    let db = db()?;
    let auth = auth()?;

    // shouldn't happen
    let user = User::get_from_username(&username, &db)
        .await
        .ok_or(ServerFnError::ServerError(format!(
            "USER_NOT_FOUND: no user by the name of {username} exists"
        )))?;

    let current_user = &auth.current_user;

    if !as_admin && Some(&username) != current_user.clone().map(|user| user.username).as_ref() {
        return Err(ServerFnError::ServerError(
            "WRONG_USER: There is a mismatch between the username and the current user.".into(),
        ));
    }

    if as_admin
        && current_user
            .clone()
            .map(|user| user.role == UserRole::Admin)
            != Some(true)
    {
        return Err(ServerFnError::ServerError(
            "NICE_TRY: You're not the admin, dumbass.".into(),
        ));
    }

    if !verify(password, &user.password)? {
        return Err(ServerFnError::ServerError(
            "BAD_PASSWORD: Password does not match.".into(),
        ));
    }

    if let Some(new_username) = new_username {
        log!("{username} is changing their username to {new_username}");
        let user_exists = User::name_taken(&new_username, &db).await?;
        if user_exists {
            return Err(ServerFnError::ServerError(
                "TAKEN_NAME: User already exists!".to_string(),
            ));
        }

        #[derive(Serialize)]
        struct Username {
            username: String,
        }

        let _new_user: Option<SurrealUser> = db
            .update(("user", &user.id))
            .merge(Username {
                username: new_username,
            })
            .await?;
    }

    if let (Some(new_password), Some(new_password_confirmation)) =
        (new_password, new_password_confirmation)
    {
        log!("{username} is changing their password");
        const MIN_PW_LEN: usize = 6;

        if new_password.len() < MIN_PW_LEN {
            return Err(ServerFnError::ServerError(
                "SHORT_PASSWORD: Password is too short.".to_string(),
            ));
        }
        if new_password
            .chars()
            .any(|c| c.is_whitespace() || !c.is_ascii())
        {
            return Err(ServerFnError::ServerError(
                "INVALID_PASSWORD: Password contains whitespace, or non-ASCII chars.".to_string(),
            ));
        }

        if new_password != new_password_confirmation {
            return Err(ServerFnError::ServerError(
                "UNCONFIRMED_PASSWORD: Passwords did not match.".to_string(),
            ));
        }

        let new_password_hashed = hash(new_password, DEFAULT_COST).unwrap();

        #[derive(Serialize)]
        struct Password {
            password: String,
        }

        let _new_user: Option<SurrealUser> = db
            .update(("user", &user.id))
            .merge(Password {
                password: new_password_hashed,
            })
            .await?;
    }

    auth.logout_user();

    leptos_axum::redirect("/");

    Ok(())
}

type UserResource = Resource<(usize, usize, usize, usize), Result<Option<User>, ServerFnError>>;

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
