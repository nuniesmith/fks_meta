//! Unit tests for models

use fks_meta::models::{MT5Order, MT5Position, MT5MarketData};

#[test]
fn test_mt5_order_serialization() {
    let order = MT5Order {
        ticket: 12345,
        symbol: "EURUSD".to_string(),
        order_type: "OP_BUY".to_string(),
        volume: 0.1,
        price: 1.0850,
        stop_loss: Some(1.0800),
        take_profit: Some(1.0900),
        comment: Some("Test order".to_string()),
        magic: 123456,
        expiration: None,
    };
    
    let json = serde_json::to_string(&order).unwrap();
    let deserialized: MT5Order = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.symbol, "EURUSD");
    assert_eq!(deserialized.volume, 0.1);
}

#[test]
fn test_mt5_position_serialization() {
    let position = MT5Position {
        ticket: 12345,
        symbol: "EURUSD".to_string(),
        position_type: "OP_BUY".to_string(),
        volume: 0.1,
        price_open: 1.0850,
        price_current: 1.0860,
        profit: 10.0,
        swap: 0.0,
        commission: -0.5,
        stop_loss: Some(1.0800),
        take_profit: Some(1.0900),
        comment: Some("Test position".to_string()),
        magic: 123456,
        time_open: 1699113600,
    };
    
    let json = serde_json::to_string(&position).unwrap();
    let deserialized: MT5Position = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.symbol, "EURUSD");
    assert_eq!(deserialized.profit, 10.0);
}

