//! MT5 Plugin implementation for fks_execution
//!
//! This module provides the ExecutionPlugin trait implementation.
//! When used as a library, it can be integrated into fks_execution.
//! When used standalone, it provides HTTP API endpoints.

use crate::mt5::MT5Client;
use crate::config::Settings;
use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

// Types matching fks_execution plugin interface
// These should match the ExecutionPlugin trait from fks_execution

/// Execution result matching fks_execution interface
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub order_id: Option<String>,
    pub filled_quantity: f64,
    pub average_price: f64,
    pub error: Option<String>,
    pub timestamp: i64,
}

/// Order side matching fks_execution interface
#[derive(Debug, Clone, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order type matching fks_execution interface
#[derive(Debug, Clone, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
    TakeProfit,
    StopLoss,
}

/// Order structure matching fks_execution interface
#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub confidence: f64,
}

/// Market data matching fks_execution interface
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: f64,
    pub timestamp: i64,
    pub extra: serde_json::Value,
}

/// ExecutionPlugin trait (matches fks_execution interface)
#[async_trait]
pub trait ExecutionPlugin: Send + Sync {
    async fn init(&mut self, config: serde_json::Value) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn execute_order(&self, order: Order) -> Result<ExecutionResult, Box<dyn Error + Send + Sync>>;
    async fn fetch_data(&self, symbol: &str) -> Result<MarketData, Box<dyn Error + Send + Sync>>;
    fn name(&self) -> &str;
    async fn health_check(&self) -> Result<bool, Box<dyn Error + Send + Sync>>;
}

/// MT5 Plugin for fks_execution
///
/// Implements ExecutionPlugin trait to integrate MT5 with fks_execution
pub struct MT5Plugin {
    name: String,
    client: Arc<RwLock<Option<Arc<MT5Client>>>>,
    settings: Arc<RwLock<Option<Arc<Settings>>>>,
}

impl MT5Plugin {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            client: Arc::new(RwLock::new(None)),
            settings: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait]
impl ExecutionPlugin for MT5Plugin {
    async fn init(&mut self, config: serde_json::Value) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!(plugin = %self.name, "Initializing MT5 plugin");
        
        // Parse configuration
        let settings = Arc::new(Settings::from_env()
            .map_err(|e| format!("Failed to load settings: {}", e))?);
        
        // Override with config JSON if provided
        if let Some(terminal_path) = config.get("terminal_path").and_then(|v| v.as_str()) {
            // Update settings
        }
        
        // Initialize MT5 client
        let client = Arc::new(MT5Client::new(settings.clone()).await
            .map_err(|e| format!("Failed to initialize MT5 client: {}", e))?);
        
        *self.settings.write().await = Some(settings);
        *self.client.write().await = Some(client);
        
        info!(plugin = %self.name, "MT5 plugin initialized successfully");
        Ok(())
    }
    
    async fn execute_order(
        &self,
        order: Order,
    ) -> Result<ExecutionResult, Box<dyn Error + Send + Sync>> {
        let client = self.client.read().await;
        let client = client.as_ref()
            .ok_or("Plugin not initialized")?;
        
        // Convert FKS Order to MT5 Order format
        let mt5_order_type = match (order.side, order.order_type) {
            (OrderSide::Buy, OrderType::Market) => "OP_BUY".to_string(),
            (OrderSide::Sell, OrderType::Market) => "OP_SELL".to_string(),
            (OrderSide::Buy, OrderType::Limit) => "OP_BUYLIMIT".to_string(),
            (OrderSide::Sell, OrderType::Limit) => "OP_SELLLIMIT".to_string(),
            (OrderSide::Buy, OrderType::Stop) => "OP_BUYSTOP".to_string(),
            (OrderSide::Sell, OrderType::Stop) => "OP_SELLSTOP".to_string(),
            _ => return Err("Unsupported order type".into()),
        };
        
        let mt5_order = crate::models::MT5Order {
            ticket: 0, // Will be assigned by MT5
            symbol: order.symbol.clone(),
            order_type: mt5_order_type,
            volume: order.quantity,
            price: order.price.unwrap_or(0.0),
            stop_loss: order.stop_loss,
            take_profit: order.take_profit,
            comment: Some(format!("FKS order (confidence: {})", order.confidence)),
            magic: 123456, // FKS magic number
            expiration: None,
        };
        
        info!(
            plugin = %self.name,
            symbol = %order.symbol,
            side = ?order.side,
            quantity = %order.quantity,
            "Executing order via MT5"
        );
        
        match client.execute_order(&mt5_order).await {
            Ok(ticket) => {
                Ok(ExecutionResult {
                    success: true,
                    order_id: Some(ticket.to_string()),
                    filled_quantity: order.quantity,
                    average_price: order.price.unwrap_or(0.0),
                    error: None,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                })
            }
            Err(e) => {
                error!(plugin = %self.name, error = %e, "Order execution failed");
                Ok(ExecutionResult {
                    success: false,
                    order_id: None,
                    filled_quantity: 0.0,
                    average_price: 0.0,
                    error: Some(e.to_string()),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                })
            }
        }
    }
    
    async fn fetch_data(&self, symbol: &str) -> Result<MarketData, Box<dyn Error + Send + Sync>> {
        let client = self.client.read().await;
        let client = client.as_ref()
            .ok_or("Plugin not initialized")?;
        
        let mt5_data = client.get_market_data(symbol).await?;
        
        Ok(MarketData {
            symbol: mt5_data.symbol,
            bid: mt5_data.bid,
            ask: mt5_data.ask,
            last: mt5_data.last,
            volume: mt5_data.volume,
            timestamp: mt5_data.time,
            extra: serde_json::json!({
                "spread": mt5_data.spread,
                "digits": mt5_data.digits,
            }),
        })
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn health_check(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let client = self.client.read().await;
        if let Some(client) = client.as_ref() {
            Ok(client.health_check().await)
        } else {
            Ok(false)
        }
    }
}

