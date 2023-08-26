use chrono::offset::Utc;
use chrono::DateTime;
use leptos::*;

use super::*;
use crate::auth::*;

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
