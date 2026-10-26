mod controllers;
mod errors;
mod models;
mod repositories;
mod routes;
mod services;
mod state;

use std::{env, io, net::SocketAddr};

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();

    let database_url = env::var("DATABASE_URL").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "DATABASE_URL must be configured",
        )
    })?;

    let bind_address = match env::var("BIND_ADDR") {
        Ok(value) => value,
        Err(_) => "127.0.0.1:8080".to_string(),
    }
    .parse::<SocketAddr>()?;

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let state = AppState { db };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<axum::http::HeaderValue>()?)
        .allow_methods([axum::http::Method::POST, axum::http::Method::OPTIONS])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let app = Router::new()
        .merge(routes::accounts::router())
        .route("/health", axum::routing::get(|| async { "ok" }))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(bind_address).await?;

    tracing::info!(
        address = %bind_address,
        "Lexic API listening"
    );

    axum::serve(listener, app).await?;

    Ok(())
}
