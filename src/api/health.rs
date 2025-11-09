//! Health check endpoints

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use crate::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub service: String,
    pub status: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub connected: bool,
    pub mt5_status: String,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "fks_meta".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn metrics() -> (StatusCode, &'static str) {
    // TODO: Implement Prometheus metrics
    (StatusCode::OK, "# Metrics endpoint - TODO: Implement Prometheus\n")
}

pub async fn mt5_status(State(state): State<AppState>) -> Json<StatusResponse> {
    let connected = state.mt5_client.is_connected().await;
    Json(StatusResponse {
        connected,
        mt5_status: if connected { "connected" } else { "disconnected" }.to_string(),
    })
}

