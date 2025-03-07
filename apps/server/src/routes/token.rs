use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use std::env;

use crate::shared::TokenRequest;

pub async fn create_connection_token(
    headers: HeaderMap,
    Json(payload): Json<TokenRequest>,
) -> impl IntoResponse {
    let api_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let signing_key = env::var("SIGNING_KEY").expect("SIGNING_KEY must be set");

    if api_key != signing_key {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid API key"})),
        )
            .into_response();
    }

    if !payload
        .room
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-')
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                json!({"error": "Room name must contain only alphanumeric characters and hyphens"}),
            ),
        )
            .into_response();
    }

    let mut mac = Hmac::<Sha256>::new_from_slice(signing_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload.room.as_bytes());
    let signature = URL_SAFE.encode(mac.finalize().into_bytes());

    let base_url = env::var("BASE_URL").unwrap_or("ws://localhost:8080".to_string());

    (
        StatusCode::OK,
        Json(json!({
            "signed_url": format!("{}/wire?room={}&signature={}",
                base_url,
                payload.room,
                signature
            )
        })),
    )
        .into_response()
}
