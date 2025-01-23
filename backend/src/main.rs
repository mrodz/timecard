extern crate dotenv;

mod context;
mod routes;

use anyhow::{Context as AnyhowContext, Result};
use aws_config::{meta::region::RegionProviderChain, Region};
use axum::{routing::get, Router};
use context::Context;
use std::{net::SocketAddr, time::Duration};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{AllowHeaders, AllowMethods, CorsLayer};
use tower_http::timeout::TimeoutLayer;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().context("could not load environment file")?;

    let region_provider =
        RegionProviderChain::first_try("us-west-1").or_else(Region::from_static("us-west-1"));

    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::v2024_03_28()).await;

    println!("{:?}", std::env::var("AWS_PROFILE"));

    let context = Context::new(
        sdk_config,
        None::<&str>,
        "postgres://postgres:postgres@localhost:3588/postgres",
    )
    .await?;

    // let cors_origins = ["http://localhost:5173".parse().unwrap(), "http://localhost:4000".parse().unwrap()];

    // let cors = CorsLayer::new().allow_origin(cors_origins).allow_credentials(true).allow_methods([Method::GET, Method::HEAD, Method::PUT, Method::PATCH, Method::POST, Method::DELETE]).allow_headers(ContentType, "*");
    // let cors = CorsLayer::very_permissive();
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
