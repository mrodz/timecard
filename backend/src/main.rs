mod context;
mod routes;

use anyhow::{anyhow, Result};
use aws_config::{meta::region::RegionProviderChain, Region};
use axum::{error_handling::HandleErrorLayer, http::{Method, StatusCode, Uri}, routing::get, BoxError, Json, Router};
use context::Context;
use sea_orm::{Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use tokio::time::error::Elapsed;
use tower_cookies::CookieManagerLayer;
use std::{collections::HashMap, net::SocketAddr, time::Duration};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;

#[tokio::main]
async fn main() -> Result<()> {
    let region_provider =
        RegionProviderChain::first_try("us-west-1").or_else(Region::from_static("us-west-1"));

    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::v2024_03_28()).await;

    let context = Context::new(sdk_config, None::<&str>, "postgres://postgres:postgres@localhost:3588/postgres").await?;

    context.test_database_connection().await?;
    
    let cors = CorsLayer::new().allow_origin(Any);
    let cookies = CookieManagerLayer::new();
    let timeout = TimeoutLayer::new(Duration::from_secs(10));

    let app = Router::new()
        .route("/", get(root))
        .route("/user", get(routes::user::get_user))
        .layer(ServiceBuilder::new()
            .layer(cors)
            .layer(timeout)
            .layer(cookies)
        )
        .with_state(context);

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 3000))).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}
