use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::shared::AppState;

pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "all good!" })))
}

pub async fn stats_handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().await;
    let connections = state.clients.len();
    let rooms = state.rooms.len();

    (
        StatusCode::OK,
        Json(json!({ "connections": connections, "rooms": rooms })),
    )
}
