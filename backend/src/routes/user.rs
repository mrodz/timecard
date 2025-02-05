use std::collections::HashMap;

use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
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
pub async fn get_user(cookies: CookieJar, State(state): State<Context>) -> impl IntoResponse {
    let Some(access_token) = cookies.get("access_token") else {
        return (
            StatusCode::BAD_REQUEST,
            GetUserError::ContextError(ContextError::AuthError(
                AuthError::MissingAuthenticationCookie,
            )),
        )
            .into_response();
    };

    let user_data = match state.load_cognito_user(access_token.value()).await {
        Ok(x) => x,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    (
        StatusCode::OK,
        Json::from(GetUser {
            username: user_data.username().to_owned(),
            user_attributes: HashMap::from_iter(
                user_data
                    .user_attributes()
                    .iter()
                    .map(|att| (att.name().to_owned(), att.value().map(str::to_owned))),
            ),
        }),
    ).into_response()
}
