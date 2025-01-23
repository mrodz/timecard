use axum::{extract::Query, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tower_cookies::{cookie::SameSite, Cookie};

#[derive(Deserialize)]
pub struct RedirectParams {
    code: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AwsCognitoRedirect {
    access_token: String,
    expires_in: i64,
    id_token: String,
    refresh_token: String,
    token_type: String,
}

pub async fn aws_cognito_redirect(
    Query(params): Query<RedirectParams>,
    cookies: CookieJar,
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
        (
            "client_secret",
            &std::env::var("COGNITO_CLIENT_SECRET").unwrap(),
        ),
        ("code", &code),
        ("redirect_uri", "http://localhost:5173/auth/"),
    ];

    let response = client.post(token_url).form(&params).send().await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                let tokens: AwsCognitoRedirect = res.json().await.unwrap();

                let mut access_token_cookie =
                    Cookie::new("access_token", tokens.access_token.clone());

                access_token_cookie.set_same_site(SameSite::None);
                access_token_cookie.set_domain("localhost");
                access_token_cookie.set_path("/");

                let mut refresh_token_cookie =
                    Cookie::new("refresh_token", tokens.refresh_token.clone());

                refresh_token_cookie.set_same_site(SameSite::Strict);
                refresh_token_cookie.set_domain("localhost");
                refresh_token_cookie.set_path("refresh");
                refresh_token_cookie.set_http_only(true);
                refresh_token_cookie.set_secure(true);

                (
                    StatusCode::OK,
                    cookies.add(access_token_cookie).add(refresh_token_cookie),
                    Json::from(tokens),
                )
                    .into_response()
            } else {
                eprintln!("{:?}", res.text().await);
                (StatusCode::BAD_REQUEST, "Failed to exchange code for token").into_response()
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Cognito Request failed").into_response(),
    }
}
