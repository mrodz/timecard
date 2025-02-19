extern crate dotenv;

mod context;
mod routes;

use anyhow::{Context as AnyhowContext, Result};
use axum::{routing::{get, post}, Router};
use context::Context;
use std::{net::SocketAddr, time::Duration};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{AllowHeaders, AllowMethods, CorsLayer};
use tower_http::timeout::TimeoutLayer;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().context("could not load environment file")?;

    let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::v2024_03_28()).load().await;

    println!("{:?}", std::env::var("AWS_PROFILE"));

    let context = Context::new(sdk_config).await?;

    let cors = CorsLayer::new()
        .allow_headers(AllowHeaders::mirror_request())
        .allow_methods(AllowMethods::mirror_request())
        .allow_credentials(true)
        .allow_origin(["http://localhost:5173".parse().unwrap()]);
    let cookies = CookieManagerLayer::new();
    let timeout = TimeoutLayer::new(Duration::from_secs(10));

    let app = Router::new()
        .route("/", get(root))
        .route("/user", get(routes::user::get_user))
        .route("/redirect", get(routes::cognito::aws_cognito_redirect))
        .route("/clocks/{user_id}", get(routes::clocks::get_clocks))
        .route("/clocks/{user_id}", post(routes::clocks::create_clock))
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(timeout)
                .layer(cookies),
        )
        .with_state(context);

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 4000))).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}
