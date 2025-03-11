use std::ops::Deref;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::context::{
    clocks::{
        ClockError, ClockSchema, CreateClockInput, DeleteClockInput, EditClockInput,
        EditClockInputStrategy, GetClocksInput, ValidateUserClaimsToClockInput,
    },
    AuthError, Context, ContextError,
};

/// 1. Check if the cookies contain an `access_token` cookie
/// 2. Check if the `access_token` is valid
/// 3. Verify whether a `user_id` is associated with the session linked to `access_token`
pub async fn verify_session_claim_to_uuid(
    cookies: &CookieJar,
    state: &Context,
    user_id: &Uuid,
) -> Result<(Uuid, String), impl IntoResponse> {
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

    Ok((parsed_username, access_token.value().to_owned()))
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

#[derive(Deserialize, Debug)]
struct OptionalDateTime(#[serde(with = "chrono::serde::ts_seconds_option")] Option<DateTime<Utc>>);

impl Deref for OptionalDateTime {
    type Target = Option<DateTime<Utc>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Deserialize, Debug)]
pub struct EditClockBody {
    name: Option<String>,
    active: Option<bool>,
    clock_in_time: Option<OptionalDateTime>,
}

#[derive(Serialize)]
pub struct EditClockResponse {
    clock: Option<ClockSchema>,
}

#[derive(Serialize)]
pub struct DeleteClockResponse {
    clock: ClockSchema,
}

pub async fn edit_clock(
    cookies: CookieJar,
    State(state): State<Context>,
    Path((user_id, clock_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<EditClockBody>,
) -> impl IntoResponse {
    if let Err(reject) = verify_session_claim_to_uuid(&cookies, &state, &user_id).await {
        return reject.into_response();
    };

    match state
        .clock_client()
        .validate_user_claims_to_clock(ValidateUserClaimsToClockInput {
            identity_pool_user_id: user_id,
            uuid: clock_id,
        })
        .await
    {
        Ok(..) => (),
        Err(e @ ClockError::ClockNotFound(..)) => {
            return (StatusCode::FORBIDDEN, ContextError::ClockError(dbg!(e))).into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ContextError::ClockError(dbg!(e)),
            )
                .into_response()
        }
    };

    let edited_clock = match state
        .clock_client()
        .edit_clock(EditClockInput {
            uuid: clock_id,
            update: EditClockInputStrategy::Fields {
                identity_pool_user_id: user_id,
                active: payload.active,
                name: payload.name,
                clock_in_time: payload.clock_in_time.as_deref().cloned(),
            },
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

    (
        StatusCode::OK,
        Json(EditClockResponse {
            clock: edited_clock,
        }),
    )
        .into_response()
}

pub async fn delete_clock(
    cookies: CookieJar,
    State(state): State<Context>,
    Path((user_id, clock_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    if let Err(reject) = verify_session_claim_to_uuid(&cookies, &state, &user_id).await {
        return reject.into_response();
    };

    match state
        .clock_client()
        .validate_user_claims_to_clock(ValidateUserClaimsToClockInput {
            identity_pool_user_id: user_id,
            uuid: clock_id,
        })
        .await
    {
        Ok(..) => (),
        Err(e @ ClockError::ClockNotFound(..)) => {
            return (StatusCode::FORBIDDEN, ContextError::ClockError(dbg!(e))).into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ContextError::ClockError(dbg!(e)),
            )
                .into_response()
        }
    };

    let deleted_clock = match state
        .clock_client()
        .delete_clock(DeleteClockInput {
            identity_pool_user_id: user_id,
            uuid: clock_id,
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

    (
        StatusCode::OK,
        Json(DeleteClockResponse {
            clock: deleted_clock,
        }),
    )
        .into_response()
}
