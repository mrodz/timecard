use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use uuid::Uuid;

use crate::context::{
    clocks::{CreateClockInput, GetClocksInput},
    AuthError, Context, ContextError,
};

/// 1. Check if the cookies contain an `access_token` cookie
/// 2. Check if the `access_token` is valid
/// 3. Verify whether a `user_id` is associated with the session linked to `access_token`
pub async fn verify_session_claim_to_uuid(
    cookies: &CookieJar,
    state: &Context,
    user_id: &Uuid,
) -> Result<(), impl IntoResponse> {
    let Some(access_token) = cookies.get("access_token") else {
        return Err((
            StatusCode::BAD_REQUEST,
            ContextError::AuthError(AuthError::MissingAuthenticationCookie),
        )
            .into_response());
    };

    let user_data = match state.load_cognito_user(access_token.value()).await {
        Ok(x) => x,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e).into_response()),
    };

    let Ok(parsed_username) = Uuid::try_parse(user_data.username()) else {
        unreachable!("username is not a UUID: {}", user_data.username());
    };

    if user_id != &parsed_username {
        return Err((
            StatusCode::UNAUTHORIZED,
            ContextError::AuthError(AuthError::Unauthorized),
        )
            .into_response());
    }

    Ok(())
}

#[axum::debug_handler]
pub async fn get_clocks(
    cookies: CookieJar,
    State(state): State<Context>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    if let Err(reject) = verify_session_claim_to_uuid(&cookies, &state, &user_id).await {
        return reject.into_response();
    };

    let clocks = match state
        .clock_client()
        .get_clocks(GetClocksInput(user_id))
        .await
    {
        Ok(x) => x,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ContextError::ClockError(dbg!(e)),
            )
                .into_response()
        }
    };

    (StatusCode::OK, Json(clocks)).into_response()
}

#[derive(Deserialize)]
pub(crate) struct CreateClockBody {
    name: String,
}

#[axum::debug_handler]
pub async fn create_clock(
    cookies: CookieJar,
    State(state): State<Context>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<CreateClockBody>,
) -> impl IntoResponse {
    if let Err(reject) = verify_session_claim_to_uuid(&cookies, &state, &user_id).await {
        return reject.into_response();
    };

    let clock = match state
        .clock_client()
        .create_clock(CreateClockInput {
            identity_pool_user_id: user_id,
            name: payload.name,
        })
        .await
    {
        Ok(x) => x,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ContextError::ClockError(dbg!(e)),
            )
                .into_response()
        }
    };

    (StatusCode::OK, Json(clock)).into_response()
}
