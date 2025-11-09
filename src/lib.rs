//! FKS Meta - MetaTrader 5 Plugin Library
//!
//! Provides MT5 integration as an execution plugin for fks_execution

pub mod api;
pub mod config;
pub mod models;
pub mod mt5;

pub use models::{MT5Order, MT5Position, MT5MarketData};
pub use mt5::{MT5Client, MT5Plugin};

/// Plugin name identifier
pub const PLUGIN_NAME: &str = "mt5";

/// Plugin version
pub const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");

