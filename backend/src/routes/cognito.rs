use axum::{extract::Query, http::{HeaderMap, HeaderValue, StatusCode}, response::IntoResponse, Json};
use axum_extra::{extract::CookieJar, headers::Authorization};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_cookies::{cookie::SameSite, Cookie, Cookies};

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
		("client_secret", &std::env::var("COGNITO_CLIENT_SECRET").unwrap()),
        ("code", &code),
        ("redirect_uri", "http://localhost:5173/auth/"),
    ];

    let response = client.post(token_url).form(&params).send().await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                let tokens: AwsCognitoRedirect = res.json().await.unwrap();

                let mut ready_cookie = Cookie::new("access_token", tokens.access_token.clone());

                ready_cookie.set_same_site(SameSite::None);
                ready_cookie.set_domain("localhost");
                ready_cookie.set_path("/");

                (StatusCode::OK, cookies.add(ready_cookie), Json::from(tokens)).into_response()
            } else {
				eprintln!("{:?}", res.text().await);
                (StatusCode::BAD_REQUEST, "Failed to exchange code for token").into_response()
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Cognito Request failed").into_response(),
    }
}
