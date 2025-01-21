use anyhow::Result;
use axum::{body::Body, extract::{rejection::JsonRejection, State}, http::{Response, StatusCode}, response::IntoResponse, Extension, Json};
use axum_extra::{
    extract::CookieJar, headers::{authorization::Bearer, Authorization}, TypedHeader
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::context::{AuthError, Context, ContextError};

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUser {
    user: String,
}

#[derive(Debug, Error)]
pub enum GetUserError {
    #[error(transparent)]
    ContextError(#[from] ContextError),
}


impl IntoResponse for GetUserError {
    fn into_response(self) -> Response<Body> {
        dbg!(&self);
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
    payload: Result<Json<Value>, JsonRejection>
) -> Result<Json<GetUser>, GetUserError> {
    let Some(access_token) = cookies.get("access_token") else {
        return Err(GetUserError::ContextError(ContextError::AuthError(AuthError::MissingAuthenticationCookie)));
    };

    let user_data = state.load_cognito_user(access_token.value()).await?;

    Ok(Json::from(GetUser {
        user: format!("{user_data:?}"),
    }))
}
