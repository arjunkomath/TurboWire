use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde_json::json;
use std::env;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tokio::sync::Mutex;

use crate::shared::{AppState, BroadcastMessage, publish_broadcast_to_redis};

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
            .map(|redis| (redis, state.instance_id()))
    };

    if let Some((redis, instance_id)) = redis {
        let redis_payload = payload.clone();
        match tokio::task::spawn_blocking(move || {
            publish_broadcast_to_redis(&redis, &instance_id, &redis_payload)
        })
        .await
        {
            Ok(Ok(0)) => {
                tracing::error!("Redis broadcast unavailable: no active subscribers");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({"message": "Redis broadcast unavailable"})),
                )
                    .into_response();
            }
            Ok(Ok(_)) => {
                let state = state.lock().await;
                state.broadcast_to_local_room(&payload.room, &payload.message);
                return (StatusCode::OK, Json(json!({"message": "Broadcasted"}))).into_response();
            }
            Ok(Err(error)) => {
                tracing::error!("Failed to publish broadcast to Redis: {error}");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({"message": "Redis broadcast unavailable"})),
                )
                    .into_response();
            }
            Err(error) => {
                tracing::error!("Redis publish task failed: {error}");
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
