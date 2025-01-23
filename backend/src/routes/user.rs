use std::collections::HashMap;

use anyhow::Result;
use axum::{
    body::Body,
    extract::{rejection::JsonRejection, State},
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::context::{AuthError, Context, ContextError};

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUser {
    username: String,
    user_attributes: HashMap<String, Option<String>>,
}

#[derive(Debug, Error)]
pub enum GetUserError {
    #[error(transparent)]
    ContextError(#[from] ContextError),
}

impl IntoResponse for GetUserError {
    fn into_response(self) -> Response<Body> {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("fetching user failed: {}", self),
        )
            .into_response()
    }
}

#[axum::debug_handler]
pub async fn get_user(
    cookies: CookieJar,
    State(state): State<Context>,
    payload: Result<Json<Value>, JsonRejection>,
) -> impl IntoResponse {
    let Some(access_token) = cookies.get("access_token") else {
        return Err(GetUserError::ContextError(ContextError::AuthError(
            AuthError::MissingAuthenticationCookie,
        )));
    };

    let user_data = state.load_cognito_user(access_token.value()).await?;

    Ok(Json::from(GetUser {
        username: user_data.username().to_owned(),
        user_attributes: HashMap::from_iter(
            user_data
                .user_attributes()
                .iter()
                .map(|att| (att.name().to_owned(), att.value().map(str::to_owned))),
        ),
    }))
}
