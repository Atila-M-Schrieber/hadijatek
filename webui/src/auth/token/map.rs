#[cfg(feature = "ssr")]
use std::collections::HashMap;

use chrono::offset::Utc;
use chrono::DateTime;
use leptos::*;
use serde::{Deserialize, Serialize};

use super::*;
use crate::auth::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map(pub String);

pub type MapCreationToken = (
    Token,
    Option<(Map, DateTime<Utc>)>,
    Option<(User, DateTime<Utc>)>,
);

#[server(CreateMapToken, "/api")]
pub async fn create_map_token() -> Result<String, ServerFnError> {
    let db = db()?;

    gen_token("map_token", &db).await
}

#[server(DeleteMapToken, "/api")]
pub async fn delete_map_token(token: String) -> Result<(), ServerFnError> {
    let db = db()?;

    delete_token("map_token", &token, &db).await
}

#[server(ClaimMapToken, "/api")]
pub async fn claim_map_token(token: String) -> Result<(), ServerFnError> {
    let db = db()?;
    let auth = auth()?;

    let user = auth.current_user.ok_or(ServerFnError::ServerError(
        "NO_USER: You must be logged in to create a map.".into(),
    ))?;

    consume_token::<SurrealUser>("map_token", &token, "user", &user.id, &db).await
}

#[server(GetMapTokenInfo, "/api")]
pub async fn get_map_token_info() -> Result<Vec<MapCreationToken>, ServerFnError> {
    let db = db()?;

    pub type SurrealUserCreationToken = (Token, Option<(SurrealUser, DateTime<Utc>)>);
    let source_tokens_user: Vec<SurrealUserCreationToken> =
        get_tokens("map_token", "user", &db).await?;

    pub type JustMapCreationToken = (Token, Option<(Map, DateTime<Utc>)>);
    let source_tokens_map: Vec<JustMapCreationToken> = get_tokens("map_token", "map", &db).await?;
    let mut source_tokens_map: HashMap<Token, Option<(Map, DateTime<Utc>)>> =
        source_tokens_map.into_iter().collect();

    let mut tokens = Vec::with_capacity(source_tokens_user.len());

    for (token, maybeuser) in source_tokens_user.into_iter() {
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

        let map = source_tokens_map
            .remove(&token)
            .expect("to have the exact same token");

        tokens.push((token, map, user));
    }
    Ok(tokens)
}
