use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde_json::json;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::shared::{AppState, BroadcastMessage};

pub async fn broadcast_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(payload): Json<BroadcastMessage>,
) -> impl IntoResponse {
    let api_key = headers
        .get("x-broadcast-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let broadcast_key = env::var("BROADCAST_KEY").unwrap_or_else(|_| {
        panic!("BROADCAST_KEY is not set");
    });

    if api_key != broadcast_key {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"message": "Invalid broadcast key"})),
        )
            .into_response();
    }

    let state = state.lock().await;
    state
        .broadcast_to_room(&payload.room, &payload.message)
        .await;

    (StatusCode::OK, Json(json!({"message": "Broadcasted"}))).into_response()
}
