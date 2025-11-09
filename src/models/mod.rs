//! Data models for MT5 integration

use serde::{Deserialize, Serialize};

/// MT5 Order representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MT5Order {
    pub ticket: u64,
    pub symbol: String,
    pub order_type: String, // "OP_BUY", "OP_SELL", "OP_BUYLIMIT", etc.
    pub volume: f64,
    pub price: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub comment: Option<String>,
    pub magic: u32,
    pub expiration: Option<i64>,
}

/// MT5 Position representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MT5Position {
    pub ticket: u64,
    pub symbol: String,
    pub position_type: String, // "OP_BUY" or "OP_SELL"
    pub volume: f64,
    pub price_open: f64,
    pub price_current: f64,
    pub profit: f64,
    pub swap: f64,
    pub commission: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub comment: Option<String>,
    pub magic: u32,
    pub time_open: i64,
}

/// MT5 Market Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MT5MarketData {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: f64,
    pub time: i64,
    pub spread: f64,
    pub digits: u32,
}

