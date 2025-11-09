# FKS Meta - MetaTrader 5 Plugin

**Port**: 8005  
**Framework**: Rust + Actix-web/Axum  
**Role**: MetaTrader 5 execution plugin for fks_execution

## Overview

FKS Meta provides MetaTrader 5 integration as a plugin for the FKS Execution service. It implements the `ExecutionPlugin` trait to enable order execution through MT5 terminals.

## Features

- **MT5 Integration**: Connects to MetaTrader 5 via MQL5 API
- **Order Execution**: Market, limit, stop, and stop-limit orders
- **Position Management**: Real-time position tracking
- **Market Data**: Live bid/ask prices and market information
- **Health Monitoring**: Connection status and terminal health checks
- **Plugin Architecture**: Implements ExecutionPlugin trait for fks_execution

## Architecture

```
┌─────────────────────────────────────────┐
│         fks_execution                   │
│  ┌───────────────────────────────────┐  │
│  │   Plugin Registry                 │  │
│  │  ┌──────────┐  ┌──────────┐      │  │
│  │  │  CCXT    │  │   MT5    │      │  │
│  │  │  Plugin  │  │  Plugin  │      │  │
│  │  └──────────┘  └──────────┘      │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
              │
              │ HTTP/WebSocket
              ▼
┌─────────────────────────────────────────┐
│         fks_meta (this service)          │
│  ┌───────────────────────────────────┐  │
│  │   MT5 Bridge Service              │  │
│  │  - Order Execution                │  │
│  │  - Market Data                    │  │
│  │  - Position Tracking              │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
              │
              │ MQL5 API / Named Pipes
              ▼
┌─────────────────────────────────────────┐
│      MetaTrader 5 Terminal              │
└─────────────────────────────────────────┘
```

## Tech Stack

- **Language**: Rust (stable)
- **Web Framework**: Axum 0.8.x
- **Async Runtime**: Tokio
- **MT5 Integration**: MQL5 API via named pipes or DLL
- **Serialization**: serde, serde_json
- **Monitoring**: Prometheus metrics

## Configuration

### Environment Variables

```bash
# Service Configuration
SERVICE_NAME=fks_meta
SERVICE_PORT=8005

# MT5 Configuration
MT5_TERMINAL_PATH=/path/to/MetaTrader5
MT5_DATA_PATH=/path/to/MetaTrader5/MQL5
MT5_ACCOUNT_NUMBER=12345678
MT5_PASSWORD=your_password
MT5_SERVER=your_broker_server
MT5_SYMBOL_PREFIX=""  # Optional prefix for symbols

# Connection Settings
MT5_TIMEOUT_MS=5000
MT5_RETRY_ATTEMPTS=3
MT5_RETRY_DELAY_MS=1000
```

### Plugin Configuration (JSON)

```json
{
  "terminal_path": "/path/to/MetaTrader5",
  "account_number": 12345678,
  "password": "encrypted_password",
  "server": "broker-server.com",
  "symbol_prefix": "",
  "timeout_ms": 5000,
  "testnet": false
}
```

## API Endpoints

### Health & Status

- `GET /health` - Service health check
- `GET /metrics` - Prometheus metrics
- `GET /status` - MT5 connection status

### Orders

- `POST /orders` - Execute order via MT5
- `GET /orders/{order_id}` - Get order status
- `DELETE /orders/{order_id}` - Cancel order

### Positions

- `GET /positions` - Get all open positions
- `GET /positions/{symbol}` - Get position for symbol
- `DELETE /positions/{symbol}` - Close position

### Market Data

- `GET /market/{symbol}` - Get current market data
- `GET /market/{symbol}/history` - Get historical data

## Directory Structure

```
repo/meta/
├── src/
│   ├── main.rs              # Service entry point
│   ├── lib.rs               # Library root
│   ├── api/                 # HTTP endpoints
│   │   ├── mod.rs
│   │   ├── orders.rs
│   │   ├── positions.rs
│   │   ├── market.rs
│   │   └── health.rs
│   ├── mt5/                 # MT5 integration
│   │   ├── mod.rs
│   │   ├── client.rs        # MT5 API client
│   │   ├── orders.rs        # Order execution
│   │   ├── positions.rs     # Position management
│   │   └── market_data.rs   # Market data fetching
│   ├── models/              # Data models
│   │   ├── mod.rs
│   │   ├── order.rs
│   │   ├── position.rs
│   │   └── market.rs
│   └── config/              # Configuration
│       ├── mod.rs
│       └── settings.rs
├── tests/
│   ├── integration/
│   │   └── test_mt5_plugin.rs
│   └── unit/
│       └── test_models.rs
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── entrypoint.sh
└── README.md
```

## Development

### Prerequisites

- Rust 1.70+
- MetaTrader 5 terminal installed
- MQL5 API access

### Building

```bash
cargo build --release
```

### Running

```bash
cargo run
```

### Testing

```bash
cargo test
```

## Integration with fks_execution

The MT5 plugin is registered in fks_execution's plugin registry:

```rust
use fks_meta::MT5Plugin;
use fks_execution::plugins::registry::PluginRegistry;

let mut registry = PluginRegistry::new();
let mt5_plugin = Arc::new(MT5Plugin::new("mt5"));
registry.register("mt5".to_string(), mt5_plugin).await;
```

## 🐳 Docker

### Build

```bash
docker build -t nuniesmith/fks:meta-latest .
```

### Run

```bash
docker run -p 8005:8005 \
  -e MT5_TERMINAL_PATH=/path/to/MetaTrader5 \
  -e MT5_ACCOUNT_NUMBER=12345678 \
  -e MT5_PASSWORD=your_password \
  -e MT5_SERVER=your_broker_server \
  nuniesmith/fks:meta-latest
```

## ☸️ Kubernetes

### Deployment

```bash
# Deploy using Helm
cd repo/main/k8s/charts/fks-platform
helm install fks-platform . -n fks-trading

# Or using the unified start script
cd /home/jordan/Documents/code/fks
./start.sh --type k8s
```

### Health Checks

Kubernetes probes:
- **Liveness**: `GET /live`
- **Readiness**: `GET /ready` (checks MT5 connection)

### Configuration

Ensure MT5 configuration is set in Kubernetes secrets:
```bash
kubectl create secret generic mt5-config -n fks-trading \
  --from-literal=mt5-account-number=12345678 \
  --from-literal=mt5-password=your-password \
  --from-literal=mt5-server=your-broker-server
```

## 📚 Documentation

- [API Documentation](docs/API.md) - Complete API reference
- [MT5 Integration Guide](docs/MT5_INTEGRATION.md) - MetaTrader 5 setup
- [Deployment Guide](docs/DEPLOYMENT.md) - Deployment instructions

## 🔗 Integration

### Dependencies

- **MetaTrader 5**: MT5 terminal must be installed and running
- **fks_execution**: Plugin registry integration

### Consumers

- **fks_execution**: Uses MT5 plugin for order execution

## 📊 Monitoring

### Health Check Endpoints

- `GET /health` - Service health
- `GET /status` - MT5 connection status
- `GET /metrics` - Prometheus metrics

### Metrics

- MT5 connection status
- Order execution latency
- Position tracking accuracy
- Market data update frequency

### Logging

- MT5 API interactions
- Order execution events
- Connection status changes
- Error tracking and retries

## 🐛 Troubleshooting

### MT5 Connection Fails

- Verify MT5 terminal is running
- Check account credentials
- Ensure server name is correct
- Verify network connectivity

### Order Execution Fails

- Check account permissions
- Verify symbol is tradable
- Check account balance
- Review MT5 terminal logs

---

**Repository**: [nuniesmith/fks_meta](https://github.com/nuniesmith/fks_meta)  
**Docker Image**: `nuniesmith/fks:meta-latest`  
**Status**: Active Development

