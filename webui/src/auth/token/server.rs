use super::*;
use crate::auth::User;
use axum_session_auth::SessionSurrealPool;
use leptos::*;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Id;
use surrealdb::Surreal;
pub type AuthSession =
    axum_session_auth::AuthSession<User, String, SessionSurrealPool<Client>, Surreal<Client>>;

/// Generates token on specified table
pub async fn gen_token(table: &str, db: &Surreal<Client>) -> Result<String, ServerFnError> {
    db.use_ns("hadijatek").use_db("auth").await?;

    let token = Id::rand().to_raw();

    // log!("Creating token {token} on {table}");

    // Without catching the value, it returns a server error.
    // I assume it tries to become a Result<String, ...> insead of a Token
    let _the_token: Token = db
        .create((table, &token))
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
pub async fn delete_token(
    table: &str,
    token: &str,
    db: &Surreal<Client>,
) -> Result<(), ServerFnError> {
    db.use_ns("hadijatek").use_db("auth").await?;

    let _deleted_token: Option<Token> = db.delete((table, token)).await?;

    Ok(())
}

/// Gets the consumer of a token
pub async fn get_consumer<T: for<'a> Deserialize<'a> + Debug>(
    token_table: &str,
    token: &str,
    consumer_table: &str,
    db: &Surreal<Client>,
) -> Result<Option<(T, DateTime<Utc>)>, ServerFnError> {
    let query = format!("SELECT <-consume<-{consumer_table}.* FROM {token_table}:{token}");
    let mut result = db.query(query).await?;
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
                    .clone(),
            )
            .ok()
        };

        let consumer: Option<T> = consumer();

        // log!("consumer option: {consumer:?}");

        if consumer.is_some() {
            let query = format!("SELECT VALUE <-consume.time FROM {token_table}:{token}");
            let mut result = db.query(query).await?;
            // log!("consumption time result: {result:?}");
            let time: Option<serde_json::Value> = result.take(0)?;
            let time: Option<DateTime<Utc>> =
                time.and_then(|value| serde_json::from_value(value[0].clone()).ok());
            // log!("time of consumption: {time:?}");
            let time = time.ok_or(ServerFnError::ServerError(
                "Consumer found, but no time!?".into(),
            ))?;

            let consumer = consumer.map(|c| (c, time));

            Ok(consumer)
        } else {
            Ok(None)
        }
    } else {
        Err(ServerFnError::ServerError("No JSON Value returned!".into()))
    }
}

pub async fn verify_token<T: for<'a> Deserialize<'a> + Debug>(
    token_table: &str,
    token: &str,
    consumer_table: &str,
    db: &Surreal<Client>,
) -> Result<(), ServerFnError> {
    let token_exists: Option<Token> = db.select((token_table, token)).await?;

    if token_exists.is_none() {
        return Err(ServerFnError::ServerError(
            "BAD_TOKEN: Token not found".into(),
        ));
    }

    if get_consumer::<T>(token_table, token, consumer_table, db)
        .await?
        .is_some()
    {
        return Err(ServerFnError::ServerError(
            "USED_TOKEN: This token has already been consumed".into(),
        ));
    }

    Ok(())
}

/// Consumes the appropriate token, by RELATE-ing it to the consumer
pub async fn consume_token<T: for<'a> Deserialize<'a> + Debug>(
    token_table: &str,
    token: &str,
    consumer_table: &str,
    consumer_id: &str,
    db: &Surreal<Client>,
) -> Result<(), ServerFnError> {
    db.use_ns("hadijatek").use_db("auth").await?;

    verify_token::<T>(token_table, token, consumer_table, db).await?;

    let query = format!(
        "RELATE {consumer_table}:{consumer_id}->consume->{token_table}:{token} \
            SET time=\"{}\"",
        Utc::now(),
    );
    db.query(query).await?;

    Ok(())
}

type Consumer<T> = Option<(T, DateTime<Utc>)>;

/// Fetches all tokens on a spicified table, with <T> as the type of relation.
/// Option is None if the token has not been consumed yet,
/// and the Some of the consumer and the time at which the consumption occured.
pub async fn get_tokens<T>(
    token_table: &str,
    consumer_table: &str,
    db: &Surreal<Client>,
) -> Result<Vec<(Token, Option<(T, DateTime<Utc>)>)>, ServerFnError>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug,
{
    db.use_ns("hadijatek").use_db("auth").await?;

    let just_tokens: Vec<Token> = db.select(token_table).await?;

    let mut tokens: Vec<(Token, Consumer<T>)> = Vec::with_capacity(just_tokens.len());

    for token in just_tokens.into_iter() {
        let consumer = get_consumer(token_table, &token.token, consumer_table, db).await;
        tokens.push((token, consumer?));
    }

    tokens.sort_by(
        |(Token { created: t1, .. }, o1), (Token { created: t2, .. }, o2)| match (o1, o2) {
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            _ => t2.cmp(t1),
        },
    );

    Ok(tokens)
}
