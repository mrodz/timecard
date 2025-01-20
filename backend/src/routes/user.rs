use anyhow::Result;
use axum::{body::Body, extract::{rejection::JsonRejection, State}, http::{Response, StatusCode}, response::IntoResponse, Extension, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::context::{Context, ContextError};

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
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("fetching user failed: {}", self),
        )
            .into_response()
    }
}

#[axum::debug_handler]
pub async fn get_user(
    TypedHeader(access_token): TypedHeader<Authorization<Bearer>>,
    State(state): State<Context>,
    payload: Result<Json<Value>, JsonRejection>
) -> Result<Json<GetUser>, GetUserError> {
    let user_data = state.load_cognito_user(access_token.token()).await?;

    Ok(Json::from(GetUser {
        user: format!("{user_data:?}"),
    }))
}
