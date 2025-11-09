//! MT5 Client for connecting to MetaTrader 5 terminal
//!
//! This module provides a unified interface for MT5 integration.
//! It can use either:
//! - HTTP Bridge Client (recommended) - see bridge.rs
//! - Direct DLL integration (future)
//! - Named pipes (future)

use crate::config::Settings;
use crate::models::{MT5MarketData, MT5Order, MT5Position};
use crate::mt5::bridge::MT5BridgeClient;
use anyhow::Result;
use std::sync::Arc;

/// MT5 Client - Unified interface for MT5 integration
///
/// Currently uses HTTP bridge client. Can be extended to support
/// direct DLL integration or named pipes.
pub struct MT5Client {
    bridge: MT5BridgeClient,
}

impl MT5Client {
    /// Create new MT5 client
    ///
    /// Uses HTTP bridge by default. Set MT5_BRIDGE_URL environment variable
    /// to specify bridge service URL (default: http://localhost:8006)
    pub async fn new(settings: Arc<Settings>) -> Result<Self> {
        let bridge = MT5BridgeClient::new(settings).await?;
        Ok(Self { bridge })
    }
    
    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.bridge.is_connected().await
    }
    
    /// Execute order
    pub async fn execute_order(&self, order: &MT5Order) -> Result<u64> {
        self.bridge.execute_order(order).await
    }
    
    /// Get order status
    pub async fn get_order(&self, ticket: u64) -> Result<MT5Order> {
        self.bridge.get_order(ticket).await
    }
    
    /// Cancel order
    pub async fn cancel_order(&self, ticket: u64) -> Result<()> {
        self.bridge.cancel_order(ticket).await
    }
    
    /// Get all positions
    pub async fn get_positions(&self) -> Result<Vec<MT5Position>> {
        self.bridge.get_positions().await
    }
    
    /// Get position for symbol
    pub async fn get_position(&self, symbol: &str) -> Result<Option<MT5Position>> {
        self.bridge.get_position(symbol).await
    }
    
    /// Close position
    pub async fn close_position(&self, ticket: u64) -> Result<()> {
        self.bridge.close_position(ticket).await
    }
    
    /// Get market data
    pub async fn get_market_data(&self, symbol: &str) -> Result<MT5MarketData> {
        self.bridge.get_market_data(symbol).await
    }
    
    /// Health check
    pub async fn health_check(&self) -> bool {
        self.bridge.health_check().await
    }
}

