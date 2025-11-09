# MT5 Integration Guide

This guide explains how to complete the MT5 integration for fks_meta.

## Integration Options

There are several ways to integrate with MetaTrader 5:

### Option 1: HTTP Bridge Service (Recommended)

**Pattern**: Similar to how CCXT plugin works - create a Python/Node.js bridge service that communicates with MT5 via MQL5, then fks_meta calls the bridge via HTTP.

**Pros**:
- Clean separation of concerns
- Can use existing MQL5 libraries
- Easier to debug and test
- Works across platforms

**Cons**:
- Additional service to maintain
- Slight latency overhead

**Implementation**:
1. Create `mt5_bridge` service (Python/Node.js)
2. Use MQL5 Python library or MT5 Manager API
3. fks_meta calls bridge via HTTP
4. Bridge translates requests to MT5 API calls

### Option 2: MQL5 DLL via FFI

**Pattern**: Use Rust FFI to call MQL5 DLL functions directly.

**Pros**:
- Direct integration, low latency
- No intermediate service

**Cons**:
- Complex FFI setup
- Requires MQL5 DLL compilation
- Platform-specific (Windows primarily)
- Difficult to debug

**Implementation**:
1. Compile MQL5 DLL with required functions
2. Use `libloading` or `dlopen` in Rust
3. Call DLL functions via FFI
4. Handle memory management carefully

### Option 3: Named Pipes (Windows)

**Pattern**: Use Windows named pipes to communicate with MT5 Expert Advisor.

**Pros**:
- Native Windows IPC
- Low latency
- Direct communication

**Cons**:
- Windows-only
- Requires MQL5 EA running in MT5
- Complex error handling

**Implementation**:
1. Create MQL5 Expert Advisor that listens on named pipe
2. Use `tokio::net::windows::named_pipe` in Rust
3. Send/receive JSON messages
4. EA translates to MT5 API calls

### Option 4: MT5 Manager API

**Pattern**: Use MetaQuotes Manager API (if available).

**Pros**:
- Official API
- Well-documented

**Cons**:
- May require special licensing
- Limited availability
- May not support all operations

## Recommended Approach: HTTP Bridge Service

Following the CCXT plugin pattern, we recommend creating an HTTP bridge service.

### Architecture

```
fks_execution
    │
    └─> fks_meta (Rust)
            │
            └─> HTTP ──> mt5_bridge (Python)
                            │
                            └─> MQL5 API / MT5 Manager
                                    │
                                    └─> MetaTrader 5 Terminal
```

### Implementation Steps

#### Step 1: Create MT5 Bridge Service

Create `repo/meta/bridge/` directory with Python service:

```python
# bridge/main.py
from fastapi import FastAPI
from mql5 import MT5Client  # Use MetaTrader5 Python library

app = FastAPI()

mt5_client = MT5Client()

@app.post("/orders")
async def create_order(order: dict):
    """Create order in MT5"""
    result = mt5_client.order_send(
        symbol=order["symbol"],
        action=order["action"],  # TRADE_ACTION_DEAL, etc.
        volume=order["volume"],
        price=order.get("price", 0),
        sl=order.get("stop_loss"),
        tp=order.get("take_profit"),
    )
    return {"ticket": result.order, "success": result.retcode == 10009}

@app.get("/positions")
async def get_positions():
    """Get all open positions"""
    positions = mt5_client.positions_get()
    return [pos._asdict() for pos in positions]

@app.get("/market/{symbol}")
async def get_market_data(symbol: str):
    """Get market data for symbol"""
    tick = mt5_client.symbol_info_tick(symbol)
    return {
        "bid": tick.bid,
        "ask": tick.ask,
        "last": tick.last,
        "volume": tick.volume,
        "time": tick.time,
    }
```

#### Step 2: Update MT5 Client in fks_meta

Modify `src/mt5/client.rs` to call the bridge service:

```rust
pub struct MT5Client {
    settings: Arc<Settings>,
    connected: Arc<RwLock<bool>>,
    bridge_url: String,
    http_client: Client,
}

impl MT5Client {
    pub async fn new(settings: Arc<Settings>) -> Result<Self> {
        let bridge_url = settings.mt5_bridge_url
            .clone()
            .unwrap_or_else(|| "http://localhost:8006".to_string());
        
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;
        
        let mt5_client = Self {
            settings,
            connected: Arc::new(RwLock::new(false)),
            bridge_url,
            http_client: client,
        };
        
        // Test connection
        mt5_client.connect().await?;
        Ok(mt5_client)
    }
    
    async fn connect(&self) -> Result<()> {
        let health_url = format!("{}/health", self.bridge_url);
        let response = self.http_client.get(&health_url).send().await?;
        
        if response.status().is_success() {
            *self.connected.write().await = true;
            info!("Connected to MT5 bridge service");
        } else {
            return Err(anyhow::anyhow!("MT5 bridge service not available"));
        }
        Ok(())
    }
    
    pub async fn execute_order(&self, order: &MT5Order) -> Result<u64> {
        let url = format!("{}/orders", self.bridge_url);
        
        let payload = serde_json::json!({
            "symbol": order.symbol,
            "action": self.map_order_type(&order.order_type),
            "volume": order.volume,
            "price": order.price,
            "stop_loss": order.stop_loss,
            "take_profit": order.take_profit,
        });
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        let result: serde_json::Value = response.json().await?;
        
        if result["success"].as_bool().unwrap_or(false) {
            Ok(result["ticket"].as_u64().unwrap_or(0))
        } else {
            Err(anyhow::anyhow!("Order execution failed"))
        }
    }
    
    fn map_order_type(&self, order_type: &str) -> u32 {
        match order_type {
            "OP_BUY" => 0,  // TRADE_ACTION_DEAL
            "OP_SELL" => 1,
            "OP_BUYLIMIT" => 2,
            "OP_SELLLIMIT" => 3,
            "OP_BUYSTOP" => 4,
            "OP_SELLSTOP" => 5,
            _ => 0,
        }
    }
}
```

#### Step 3: Install MT5 Python Library

In the bridge service:

```bash
pip install MetaTrader5
```

#### Step 4: Configure Bridge Service

Add to `docker-compose.yml`:

```yaml
services:
  mt5_bridge:
    build:
      context: ./bridge
      dockerfile: Dockerfile
    ports:
      - "8006:8006"
    environment:
      - MT5_ACCOUNT=${MT5_ACCOUNT}
      - MT5_PASSWORD=${MT5_PASSWORD}
      - MT5_SERVER=${MT5_SERVER}
    volumes:
      - /path/to/MT5:/mt5:ro  # Mount MT5 if needed
```

## Alternative: Direct MQL5 DLL Integration

If you prefer direct integration, see `docs/MT5_DLL_INTEGRATION.md` for FFI implementation details.

## Testing

1. Start MT5 terminal
2. Start bridge service: `cd bridge && python main.py`
3. Test fks_meta: `cargo test`
4. Test integration: `curl http://localhost:8005/health`

## Next Steps

1. Choose integration method (recommended: HTTP bridge)
2. Implement bridge service or DLL integration
3. Update `src/mt5/client.rs` with actual implementation
4. Add error handling and retry logic
5. Test with real MT5 terminal
6. Add comprehensive tests

