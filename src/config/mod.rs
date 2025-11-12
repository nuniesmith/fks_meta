//! Configuration management for FKS Meta

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub service_name: String,
    pub service_port: u16,
    
    // MT5 Configuration
    pub mt5_terminal_path: Option<String>,
    pub mt5_data_path: Option<String>,
    pub mt5_account_number: Option<u64>,
    pub mt5_password: Option<String>,
    pub mt5_server: Option<String>,
    pub mt5_symbol_prefix: String,
    
    // Connection Settings
    pub mt5_timeout_ms: u64,
    pub mt5_retry_attempts: u32,
    pub mt5_retry_delay_ms: u64,
    pub mt5_testnet: bool,
    
    // Bridge Service (if using HTTP bridge)
    pub mt5_bridge_url: Option<String>,
}

impl Settings {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            service_name: env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "fks_meta".to_string()),
            service_port: env::var("SERVICE_PORT")
                .unwrap_or_else(|_| "8005".to_string())
                .parse()
                .unwrap_or(8005),
            
            mt5_terminal_path: env::var("MT5_TERMINAL_PATH").ok(),
            mt5_data_path: env::var("MT5_DATA_PATH").ok(),
            mt5_account_number: env::var("MT5_ACCOUNT_NUMBER")
                .ok()
                .and_then(|s| s.parse().ok()),
            mt5_password: env::var("MT5_PASSWORD").ok(),
            mt5_server: env::var("MT5_SERVER").ok(),
            mt5_symbol_prefix: env::var("MT5_SYMBOL_PREFIX")
                .unwrap_or_else(|_| String::new()),
            
            mt5_timeout_ms: env::var("MT5_TIMEOUT_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            mt5_retry_attempts: env::var("MT5_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            mt5_retry_delay_ms: env::var("MT5_RETRY_DELAY_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            mt5_testnet: env::var("MT5_TESTNET")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            
            mt5_bridge_url: env::var("MT5_BRIDGE_URL").ok(),
        })
    }
}

