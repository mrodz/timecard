use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse, Json,
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::context::{clocks::GetClocksInput, AuthError, Context, ContextError};

#[axum::debug_handler]
pub async fn get_clocks(
    cookies: CookieJar,
    State(state): State<Context>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let Some(access_token) = cookies.get("access_token") else {
        return (
            StatusCode::BAD_REQUEST,
            ContextError::AuthError(AuthError::MissingAuthenticationCookie),
        )
            .into_response();
    };

    let user_data = match state.load_cognito_user(access_token.value()).await {
        Ok(x) => x,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    let Ok(parsed_username) = Uuid::try_parse(user_data.username()) else {
        unreachable!("username is not a UUID: {}", user_data.username());
    };

    if user_id != parsed_username {
        return (
            StatusCode::UNAUTHORIZED,
            ContextError::AuthError(AuthError::Unauthorized),
        )
            .into_response();
    }

    let clocks = match state.clock_client().get_clocks(GetClocksInput(user_id)).await {
        Ok(x) => x,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ContextError::ClockError(dbg!(e))).into_response()
    };

    (StatusCode::OK, Json(clocks)).into_response()
}
