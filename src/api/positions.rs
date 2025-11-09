//! Position management endpoints

use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::Serialize;
use crate::AppState;
use crate::models::MT5Position;

pub async fn list_positions(
    State(state): State<AppState>,
) -> Result<Json<Vec<MT5Position>>, (StatusCode, String)> {
    match state.mt5_client.get_positions().await {
        Ok(positions) => Ok(Json(positions)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn get_position(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<MT5Position>, (StatusCode, String)> {
    match state.mt5_client.get_position(&symbol).await {
        Ok(Some(position)) => Ok(Json(position)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Position not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn close_position(
    State(state): State<AppState>,
    Path(ticket): Path<u64>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state.mt5_client.close_position(ticket).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

