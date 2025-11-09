//! MetaTrader 5 integration module

pub mod bridge;
pub mod client;
pub mod plugin;

pub use bridge::MT5BridgeClient;
pub use client::MT5Client;
pub use plugin::MT5Plugin;

