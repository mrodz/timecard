mod context;

use anyhow::Result;
use aws_config::{meta::region::RegionProviderChain, Region};
use axum::{routing::get, Router};
use context::Context;
use sea_orm::{Database, DatabaseConnection};
use std::{collections::HashMap, net::SocketAddr};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> Result<()> {
    let region_provider =
        RegionProviderChain::first_try("us-west-1").or_else(Region::from_static("us-west-1"));

    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::v2024_03_28()).await;

    let context = Context::new(sdk_config, None::<&str>, "postgres://postgres:postgres@localhost:3588/postgres").await?;
    
    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/", get(root))
        .layer(ServiceBuilder::new().layer(cors))
        .with_state(context);

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 3000))).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}
