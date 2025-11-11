//! Market data endpoints

use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::AppState;
use crate::models::MT5MarketData;

pub async fn get_market_data(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<MT5MarketData>, (StatusCode, String)> {
    match state.mt5_client.get_market_data(&symbol).await {
        Ok(data) => Ok(Json(data)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

