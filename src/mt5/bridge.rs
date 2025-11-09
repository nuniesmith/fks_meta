//! HTTP Bridge Client for MT5 Integration
//!
//! This module provides an HTTP client to communicate with an MT5 bridge service.
//! The bridge service (Python/Node.js) handles actual MT5 API calls via MQL5.

use crate::config::Settings;
use crate::models::{MT5MarketData, MT5Order, MT5Position};
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Response from MT5 bridge service
#[derive(Debug, Deserialize)]
struct BridgeResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

/// Order response from bridge
#[derive(Debug, Deserialize)]
struct OrderResponse {
    ticket: u64,
    retcode: Option<u32>,
}

/// Position data from bridge
#[derive(Debug, Deserialize)]
struct PositionData {
    ticket: u64,
    symbol: String,
    #[serde(rename = "type")]
    position_type: u32, // 0 = buy, 1 = sell
    volume: f64,
    price_open: f64,
    price_current: f64,
    profit: f64,
    swap: f64,
    commission: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
    comment: Option<String>,
    magic: u32,
    time_open: i64,
}

/// Market data from bridge
#[derive(Debug, Deserialize)]
struct MarketDataResponse {
    symbol: String,
    bid: f64,
    ask: f64,
    last: f64,
    volume: f64,
    time: i64,
    spread: f64,
    digits: u32,
}

/// HTTP Bridge Client for MT5
///
/// Communicates with an external MT5 bridge service (Python/Node.js)
/// that handles actual MT5 API calls via MQL5.
pub struct MT5BridgeClient {
    settings: Arc<Settings>,
    bridge_url: String,
    http_client: Client,
    connected: Arc<RwLock<bool>>,
}

impl MT5BridgeClient {
    /// Create new bridge client
    pub async fn new(settings: Arc<Settings>) -> Result<Self> {
        let bridge_url = std::env::var("MT5_BRIDGE_URL")
            .unwrap_or_else(|_| "http://localhost:8006".to_string());
        
        let http_client = Client::builder()
            .timeout(Duration::from_secs(
                settings.mt5_timeout_ms / 1000
            ))
            .build()
            .context("Failed to create HTTP client")?;
        
        let client = Self {
            settings,
            bridge_url: bridge_url.clone(),
            http_client,
            connected: Arc::new(RwLock::new(false)),
        };
        
        // Test connection
        if let Err(e) = client.connect().await {
            warn!("Failed to connect to MT5 bridge: {}", e);
            // Don't fail initialization, will retry on first use
        }
        
        Ok(client)
    }
    
    /// Connect to bridge service
    async fn connect(&self) -> Result<()> {
        let health_url = format!("{}/health", self.bridge_url);
        let response = self.http_client
            .get(&health_url)
            .send()
            .await
            .context("Failed to reach MT5 bridge service")?;
        
        if response.status().is_success() {
            *self.connected.write().await = true;
            info!(bridge_url = %self.bridge_url, "Connected to MT5 bridge service");
            Ok(())
        } else {
            *self.connected.write().await = false;
            Err(anyhow::anyhow!(
                "MT5 bridge service returned status: {}",
                response.status()
            ))
        }
    }
    
    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }
    
    /// Execute order via bridge
    pub async fn execute_order(&self, order: &MT5Order) -> Result<u64> {
        if !self.is_connected().await {
            // Try to reconnect
            if let Err(e) = self.connect().await {
                return Err(anyhow::anyhow!("Not connected to MT5 bridge: {}", e));
            }
        }
        
        let url = format!("{}/orders", self.bridge_url);
        
        // Map MT5 order type to bridge format
        let action = self.map_order_type_to_action(&order.order_type)?;
        
        let payload = serde_json::json!({
            "symbol": order.symbol,
            "action": action,
            "volume": order.volume,
            "price": order.price,
            "stop_loss": order.stop_loss,
            "take_profit": order.take_profit,
            "comment": order.comment,
            "magic": order.magic,
        });
        
        info!(
            url = %url,
            symbol = %order.symbol,
            "Sending order to MT5 bridge"
        );
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send order to bridge")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Bridge returned error: {} - {}",
                response.status(),
                error_text
            ));
        }
        
        let result: BridgeResponse<OrderResponse> = response
            .json()
            .await
            .context("Failed to parse bridge response")?;
        
        if result.success {
            if let Some(data) = result.data {
                info!(ticket = data.ticket, "Order executed successfully");
                Ok(data.ticket)
            } else {
                Err(anyhow::anyhow!("Bridge returned success but no ticket"))
            }
        } else {
            Err(anyhow::anyhow!(
                "Order execution failed: {}",
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }
    
    /// Get order status
    pub async fn get_order(&self, ticket: u64) -> Result<MT5Order> {
        let url = format!("{}/orders/{}", self.bridge_url, ticket);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Order not found: {}", ticket));
        }
        
        let result: BridgeResponse<MT5Order> = response.json().await?;
        
        if result.success {
            result.data.ok_or_else(|| anyhow::anyhow!("No order data returned"))
        } else {
            Err(anyhow::anyhow!(
                "Failed to get order: {}",
                result.error.unwrap_or_default()
            ))
        }
    }
    
    /// Cancel order
    pub async fn cancel_order(&self, ticket: u64) -> Result<()> {
        let url = format!("{}/orders/{}", self.bridge_url, ticket);
        
        let response = self.http_client
            .delete(&url)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to cancel order: {}", ticket))
        }
    }
    
    /// Get all positions
    pub async fn get_positions(&self) -> Result<Vec<MT5Position>> {
        let url = format!("{}/positions", self.bridge_url);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await?;
        
        let result: BridgeResponse<Vec<PositionData>> = response.json().await?;
        
        if result.success {
            if let Some(positions) = result.data {
                Ok(positions.into_iter().map(|p| self.position_data_to_model(p)).collect())
            } else {
                Ok(vec![])
            }
        } else {
            Err(anyhow::anyhow!(
                "Failed to get positions: {}",
                result.error.unwrap_or_default()
            ))
        }
    }
    
    /// Get position for symbol
    pub async fn get_position(&self, symbol: &str) -> Result<Option<MT5Position>> {
        let url = format!("{}/positions/{}", self.bridge_url, symbol);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await?;
        
        if response.status() == 404 {
            return Ok(None);
        }
        
        let result: BridgeResponse<PositionData> = response.json().await?;
        
        if result.success {
            if let Some(data) = result.data {
                Ok(Some(self.position_data_to_model(data)))
            } else {
                Ok(None)
            }
        } else {
            Err(anyhow::anyhow!(
                "Failed to get position: {}",
                result.error.unwrap_or_default()
            ))
        }
    }
    
    /// Close position
    pub async fn close_position(&self, ticket: u64) -> Result<()> {
        let url = format!("{}/positions/{}", self.bridge_url, ticket);
        
        let response = self.http_client
            .delete(&url)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to close position: {}", ticket))
        }
    }
    
    /// Get market data
    pub async fn get_market_data(&self, symbol: &str) -> Result<MT5MarketData> {
        let url = format!("{}/market/{}", self.bridge_url, symbol);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await?;
        
        let result: BridgeResponse<MarketDataResponse> = response.json().await?;
        
        if result.success {
            if let Some(data) = result.data {
                Ok(MT5MarketData {
                    symbol: data.symbol,
                    bid: data.bid,
                    ask: data.ask,
                    last: data.last,
                    volume: data.volume,
                    time: data.time,
                    spread: data.spread,
                    digits: data.digits,
                })
            } else {
                Err(anyhow::anyhow!("No market data returned"))
            }
        } else {
            Err(anyhow::anyhow!(
                "Failed to get market data: {}",
                result.error.unwrap_or_default()
            ))
        }
    }
    
    /// Health check
    pub async fn health_check(&self) -> bool {
        self.is_connected().await
    }
    
    /// Map MT5 order type to action code
    fn map_order_type_to_action(&self, order_type: &str) -> Result<u32> {
        match order_type {
            "OP_BUY" => Ok(0),      // TRADE_ACTION_DEAL
            "OP_SELL" => Ok(1),     // TRADE_ACTION_DEAL
            "OP_BUYLIMIT" => Ok(2), // TRADE_ACTION_PENDING
            "OP_SELLLIMIT" => Ok(3),
            "OP_BUYSTOP" => Ok(4),
            "OP_SELLSTOP" => Ok(5),
            _ => Err(anyhow::anyhow!("Unknown order type: {}", order_type)),
        }
    }
    
    /// Convert position data to model
    fn position_data_to_model(&self, data: PositionData) -> MT5Position {
        MT5Position {
            ticket: data.ticket,
            symbol: data.symbol,
            position_type: if data.position_type == 0 {
                "OP_BUY".to_string()
            } else {
                "OP_SELL".to_string()
            },
            volume: data.volume,
            price_open: data.price_open,
            price_current: data.price_current,
            profit: data.profit,
            swap: data.swap,
            commission: data.commission,
            stop_loss: data.stop_loss,
            take_profit: data.take_profit,
            comment: data.comment,
            magic: data.magic,
            time_open: data.time_open,
        }
    }
}

