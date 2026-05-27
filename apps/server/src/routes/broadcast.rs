use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde_json::json;
use std::env;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tokio::sync::Mutex;

use crate::shared::{AppState, BroadcastMessage, append_broadcast_to_redis_stream};

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

    if api_key
        .as_bytes()
        .ct_eq(broadcast_key.as_bytes())
        .unwrap_u8()
        == 0
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"message": "Invalid broadcast key"})),
        )
            .into_response();
    }

    let redis = {
        let state = state.lock().await;
        state
            .redis_client()
            .map(|redis| (redis, state.redis_stream_config()))
    };

    if let Some((redis, stream_config)) = redis {
        let redis_payload = payload.clone();
        match tokio::task::spawn_blocking(move || {
            append_broadcast_to_redis_stream(&redis, stream_config, &redis_payload)
        })
        .await
        {
            Ok(Ok(_)) => {
                return (StatusCode::OK, Json(json!({"message": "Broadcasted"}))).into_response();
            }
            Ok(Err(error)) => {
                tracing::error!("Failed to append broadcast to Redis stream: {error}");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({"message": "Redis broadcast unavailable"})),
                )
                    .into_response();
            }
            Err(error) => {
                tracing::error!("Redis stream write task failed: {error}");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({"message": "Redis broadcast unavailable"})),
                )
                    .into_response();
            }
        }
    }

    let state = state.lock().await;
    state.broadcast_to_local_room(&payload.room, &payload.message);

    (StatusCode::OK, Json(json!({"message": "Broadcasted"}))).into_response()
}
