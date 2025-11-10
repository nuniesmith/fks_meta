//! Order management endpoints

use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::MT5Order;

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub order_type: String,
    pub volume: f64,
    pub price: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub comment: Option<String>,
}

#[derive(Serialize)]
pub struct OrderResponse {
    pub ticket: u64,
    pub symbol: String,
    pub status: String,
}

pub async fn create_order(
    State(state): State<AppState>,
    Json(request): Json<CreateOrderRequest>,
) -> Result<Json<OrderResponse>, (StatusCode, String)> {
    let order = MT5Order {
        ticket: 0,
        symbol: request.symbol,
        order_type: request.order_type,
        volume: request.volume,
        price: request.price,
        stop_loss: request.stop_loss,
        take_profit: request.take_profit,
        comment: request.comment,
        magic: 123456,
        expiration: None,
    };
    
    match state.mt5_client.execute_order(&order).await {
        Ok(ticket) => Ok(Json(OrderResponse {
            ticket,
            symbol: order.symbol,
            status: "pending".to_string(),
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn get_order(
    State(state): State<AppState>,
    Path(ticket): Path<u64>,
) -> Result<Json<MT5Order>, (StatusCode, String)> {
    match state.mt5_client.get_order(ticket).await {
        Ok(order) => Ok(Json(order)),
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

pub async fn cancel_order(
    State(state): State<AppState>,
    Path(ticket): Path<u64>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state.mt5_client.cancel_order(ticket).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

