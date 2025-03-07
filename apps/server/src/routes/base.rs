use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

pub async fn root_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({ "server": "TurboWire", "version": env!("CARGO_PKG_VERSION")})),
    )
}

pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "all good!" })))
}
