use anyhow::Result;
use axum::routing::{get, post};
use axum::{Router, routing::any};
use dotenv::dotenv;
use routes::broadcast::broadcast_handler;
use routes::health::{health_handler, stats_handler};
use routes::wire::ws_handler;
use shared::AppState;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;
mod shared;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    if env::var("SIGNING_KEY").is_err() {
        panic!("SIGNING_KEY is not set");
    }

    if env::var("BROADCAST_KEY").is_err() {
        panic!("BROADCAST_KEY is not set");
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(Mutex::new(AppState::default()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Client endpoints
        .route("/", any(ws_handler))
        // Serverless endpoints
        .route("/broadcast", post(broadcast_handler))
        // Health check
        .route("/health", get(health_handler))
        .route("/stats", get(stats_handler))
        .with_state(state)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let host = env::var("HOST").unwrap_or_else(|_| "[::]".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    if let Ok(limit) = env::var("CONNECTION_LIMIT") {
        tracing::debug!("connection limit set to: {}", limit);
    }

    if let Ok(url) = env::var("MESSAGE_WEBHOOK_URL") {
        tracing::debug!("message webhook url set to: {}", url);
    }

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
