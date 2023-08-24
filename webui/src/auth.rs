use std::fmt::Debug;

use crate::error::*;
use crate::lang::*;
use cfg_if::cfg_if;
use chrono::offset::Utc;
use chrono::DateTime;
use leptos::*;
use serde::{Deserialize, Serialize};

cfg_if! { if #[cfg(feature = "ssr")] {
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

/// Tokens
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Token {
    pub created: DateTime<Utc>,
    pub token: String,
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

// gen/delete_token
cfg_if! { if #[cfg(feature = "ssr")] {

    /// Generates token on specified table
    pub async fn gen_token(table: &str, db: &Surreal<Client>) -> Result<String, ServerFnError> {
        db.use_ns("hadijatek").use_db("auth").await?;

        let token = Id::rand().to_raw();

        // log!("Creating token {token} on {table}");

        // Without catching the value, it returns a server error.
        // I assume it tries to become a Result<String, ...> insead of a Token
        let _the_token: Token = db.create((table, &token))
            .content(Token {
                created: Utc::now(),
                token: token.clone(),
            })
            .await?;

        // dbg!(the_token);
        // log!("Token created");

        Ok(token)
    }

    /// Deletes token on spedified table
    pub async fn delete_token(table: &str, token: &str, db: &Surreal<Client>) -> Result<(), ServerFnError> {
        db.use_ns("hadijatek").use_db("auth").await?;

        let _deleted_token: Option<Token> = db.delete((table, token)).await?;

        Ok(())
    }
}}

// get_consumer
cfg_if! { if #[cfg(feature = "ssr")] {
    /// Gets the consumer of a token
    pub async fn get_consumer<T: for<'a> Deserialize<'a> + Debug>(
        token_table: &str,
        token: &str,
        consumer_table: &str,
        db: &Surreal<Client>
    ) -> Result<Option<(T, DateTime<Utc>)>, ServerFnError> {
        let query = format!("SELECT <-consume<-{consumer_table}.* FROM {token_table}:{token}");
        let mut result = db
            .query(query)
            .await?;
        // log!("consumption time result: {result:?}");

        let consumer: Option<serde_json::Value> = result.take(0)?;
        // log!("consumer Value: {consumer:#?}");

        if let Some(consumer) = consumer {
            // closure to be able to use ? syntax
            let consumer = || {
                serde_json::from_value(
                    consumer
                    .get("<-consume")?
                    .get(format!("<-{consumer_table}"))?
                    .get(0)?
                    .clone()
                ).ok()
            };

            let consumer: Option<T> = consumer();

            // log!("consumer option: {consumer:?}");

            if consumer.is_some() {
                let query = format!("SELECT VALUE <-consume.time FROM {token_table}:{token}");
                let mut result = db
                    .query(query)
                    .await?;
                // log!("consumption time result: {result:?}");
                let time: Option<serde_json::Value> = result.take(0)?;
                let time: Option<DateTime<Utc>> = time.and_then(|value|
                    serde_json::from_value(value[0].clone()).ok()
                );
                // log!("time of consumption: {time:?}");
                let time = time.ok_or(ServerFnError::ServerError("Consumer found, but no time!?".into()))?;

                let consumer = consumer.map(|c| (c, time));

                Ok(consumer)
            } else {
                Ok(None)
            }
        } else {
            Err(ServerFnError::ServerError("No JSON Value returned!".into()))
        }
    }
}}

// verify/consume_token
cfg_if! { if #[cfg(feature = "ssr")] {
    pub async fn verify_token<T: for<'a> Deserialize<'a> + Debug>(
        token_table: &str,
        token: &str,
        consumer_table: &str,
        db: &Surreal<Client>
    ) -> Result<(), ServerFnError> {
        let token_exists: Option<Token> = db.select((token_table, token)).await?;

        if token_exists.is_none() {
            return Err(ServerFnError::ServerError("BAD_TOKEN: Token not found".into()));
        }

        if get_consumer::<T>(token_table, token, consumer_table, db).await?.is_some() {
            return Err(ServerFnError::ServerError("USED_TOKEN: This token has already been consumed".into()))
        }

        Ok(())
    }

    /// Consumes the appropriate token, by RELATE-ing it to the consumer
    pub async fn consume_token<T: for<'a> Deserialize<'a> + Debug>(
        token_table: &str,
        token: &str,
        consumer_table: &str,
        consumer_id: &str,
        db: &Surreal<Client>
    ) -> Result<(), ServerFnError> {
        db.use_ns("hadijatek").use_db("auth").await?;

        verify_token::<T>(token_table, token, consumer_table, db).await?;

        // go format! this
        let query = format!(
            "RELATE {consumer_table}:{consumer_id}->consume->{token_table}:{token} \
            SET time=\"{}\"",
            Utc::now(),
        );
        db.query(query).await?;

        Ok(())
    }
}}

// get_tokens
cfg_if! { if #[cfg(feature = "ssr")] {

    /// Fetches all tokens on a spicified table, with <T> as the type of relation.
    /// Option is None if the token has not been consumed yet,
    /// and the Some of the consumer and the time at which the consumption occured.
    pub async fn get_tokens<T>(
        token_table: &str,
        consumer_table: &str,
        db: &Surreal<Client>,
    ) -> Result<Vec<(Token, Option<(T, DateTime<Utc>)>)>, ServerFnError>
    where T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug {
        db.use_ns("hadijatek").use_db("auth").await?;

        let just_tokens: Vec<Token> = db.select(token_table).await?;

        let mut tokens: Vec<(Token, Option<(T, DateTime<Utc>)>)> =
            Vec::with_capacity(just_tokens.len());

        for token in just_tokens.into_iter() {
            let consumer = get_consumer(token_table, &token.token, consumer_table, db).await;
            tokens.push((token, consumer?));
        }

        tokens
            .sort_by(|(Token { created: t1, .. }, o1), (Token { created: t2, .. }, o2)|
                match (o1, o2) {
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    _ => t2.cmp(t1),
                });

        Ok(tokens)
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

#[server(CreateUserToken, "/api")]
pub async fn create_user_token() -> Result<String, ServerFnError> {
    let db = db()?;

    gen_token("user_token", &db).await
}

#[server(DeleteUserToken, "/api")]
pub async fn delete_user_token(token: String) -> Result<(), ServerFnError> {
    let db = db()?;

    delete_token("user_token", &token, &db).await
}

pub type UserCreationToken = (Token, Option<(User, DateTime<Utc>)>);

#[server(GetUserTokenInfo, "/api")]
pub async fn get_user_token_info() -> Result<Vec<UserCreationToken>, ServerFnError> {
    let db = db()?;

    pub type SurrealUserCreationToken = (Token, Option<(SurrealUser, DateTime<Utc>)>);

    let source_tokens: Vec<SurrealUserCreationToken> =
        get_tokens("user_token", "user", &db).await?;

    let mut tokens = Vec::with_capacity(source_tokens.len());

    for (token, maybeuser) in source_tokens.into_iter() {
        let user = if let Some((s_user, time)) = maybeuser {
            if let Some(user) = User::get_from_username(&s_user.username, &db).await {
                Some((user, time))
            } else {
                log!("Wtf"); // should never be reached
                None
            }
        } else {
            None
        };
        tokens.push((token, user));
    }
    Ok(tokens)
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
