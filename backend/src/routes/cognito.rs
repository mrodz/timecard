use axum::{extract::Query, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use serde_json::Value;
use tower_cookies::{Cookie, Cookies};

#[derive(Deserialize)]
pub struct RedirectParams {
    code: Option<String>,
}
pub async fn aws_cognito_redirect(
    Query(params): Query<RedirectParams>,
    cookies: Cookies,
) -> impl IntoResponse {
    let code = if let Some(code) = params.code {
        code
    } else {
        return (StatusCode::BAD_REQUEST, "No code found in query string").into_response();
    };

    let client = reqwest::Client::new();
    let token_url = format!("{}/oauth2/token", std::env::var("COGNITO_DOMAIN").unwrap());
    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", &std::env::var("COGNITO_CLIENT_ID").unwrap()),
        ("code", &code),
        ("redirect_uri", "http://localhost:3000/aws_cognito_redirect"),
    ];

    let response = client.post(token_url).form(&params).send().await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                let tokens: Value = res .json().await.unwrap();
                dbg!(&tokens);
                if let Some(access_token) = tokens.get("access_token").and_then(|t| t.as_str()) {
                    cookies.add(Cookie::new("access_token", access_token.to_string()));
                }
                (StatusCode::SEE_OTHER, [("Location", "/")]).into_response()
            } else {
                (StatusCode::BAD_REQUEST, "Failed to exchange code for token").into_response()
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Cognito Request failed").into_response(),
    }
}
