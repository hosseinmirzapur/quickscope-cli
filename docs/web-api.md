# QuickScope Web API

Base URL: `http://127.0.0.1:3000` (configurable via `--port` and `--host`)

All endpoints return JSON. Errors use the format `{ "error": "message" }` with appropriate HTTP status codes.

---

## Authentication

No authentication is required for local access. API keys (GMGN, Alph AI, LLM providers) are stored server-side in `~/.config/quickscope/.env` and are never exposed to the client.

---

## REST Endpoints

### Tokens

#### `GET /api/tokens/trending`

Returns trending tokens from GMGN.

```json
{
  "tokens": [
    {
      "address": "So11...",
      "symbol": "PEPE",
      "name": "Pepe",
      "price_usd": 0.0000123,
      "market_cap": 1500000.0,
      "liquidity_usd": 50000.0,
      "volume_5m": 25000.0,
      "volume_1h": 150000.0,
      "change_5m": 2.5,
      "change_1h": 8.1,
      "hot_level": 3,
      "smart_degen_count": 12,
      "holder_count": 450,
      "swaps_5m": 85,
      "swaps_1h": 420,
      "rug_ratio": 0.05,
      "dexscr_boost": false
    }
  ]
}
```

#### `GET /api/tokens/trenches`

Returns newly launched tokens (default: last 1h).

```json
{
  "tokens": [
    {
      "address": "So11...",
      "symbol": "NEW",
      "name": "NewToken",
      "price_usd": 0.000001,
      "market_cap": 5000.0,
      "liquidity_usd": 1000.0,
      "age_minutes": 5,
      "platform": "pump.fun",
      "holder_count": 3,
      "dev_hold_rate": 0.8,
      "bonding_curve": false
    }
  ]
}
```

#### `GET /api/tokens/watchlist`

Returns watchlisted token addresses.

```json
{
  "watchlist": [
    {
      "id": 1,
      "token_address": "So11abc...",
      "token_symbol": "PEPE",
      "added_at": "2026-07-07T10:00:00Z"
    }
  ]
}
```

#### `GET /api/tokens/analyze/{address}`

Returns full token detail and alpha analysis report.

```json
{
  "detail": {
    "symbol": "PEPE",
    "name": "Pepe",
    "address": "So11...",
    "price_usd": 0.0000123,
    "market_cap": 1500000.0,
    "security": { ... },
    "dev_info": { ... },
    "social_links": { ... }
  },
  "report": {
    "alpha_score": 78.5,
    "scores": {
      "momentum": 82.0,
      "safety": 65.0,
      "holder_quality": 70.0,
      "liquidity": 85.0,
      "dev_trust": 90.0,
      "social": 75.0
    },
    "mode": "ALPHA",
    "rug_report": {
      "severity": "Low",
      "flags": [],
      "verdict": "No significant rug indicators detected."
    }
  }
}
```

---

### Trading

#### `GET /api/positions`

Returns all open paper trading positions.

```json
{
  "positions": [
    {
      "id": "uuid-...",
      "token_address": "So11...",
      "token_symbol": "PEPE",
      "entry_price": 0.000012,
      "amount_sol": 0.5,
      "amount_tokens": 41666.67,
      "mode": "ALPHA",
      "tp_percent": 80.0,
      "sl_percent": 40.0,
      "status": "open",
      "opened_at": "2026-07-07T10:00:00Z",
      "pnl_sol": null,
      "alpha_score": 78.5
    }
  ]
}
```

#### `POST /api/trade/buy`

Execute a paper buy.

**Request:**
```json
{
  "token_address": "So11...",
  "amount_sol": 0.5,
  "mode": "ALPHA",
  "tp_percent": 80.0,
  "sl_percent": 40.0
}
```

**Response:**
```json
{
  "success": true,
  "tokens_received": 41666.67,
  "effective_price": 0.000012
}
```

#### `POST /api/trade/sell`

Execute a paper sell (partial or full).

**Request:**
```json
{
  "position_id": "uuid-...",
  "sell_percent": 100.0
}
```

**Response:**
```json
{
  "success": true,
  "pnl_sol": 0.05,
  "pnl_percent": 12.5
}
```

---

### Journal

#### `GET /api/journal`

Returns closed trade history.

```json
{
  "journal": [
    {
      "id": "uuid-...",
      "token_symbol": "PEPE",
      "entry_price": 0.000010,
      "exit_price": 0.000015,
      "pnl_sol": 0.25,
      "pnl_percent": 50.0,
      "mode": "EXPLODE",
      "status": "closed",
      "opened_at": "...",
      "closed_at": "..."
    }
  ]
}
```

---

### Strategy (Alpha Config)

#### `GET /api/strategy`

Returns current alpha filter configuration.

```json
{
  "strategy": {
    "w_momentum": 0.25,
    "w_safety": 0.15,
    "w_holder": 0.20,
    "w_liquidity": 0.18,
    "w_dev": 0.07,
    "w_social": 0.15,
    "hf_rug_ratio_max": 0.30,
    "hf_dev_hold_max": 0.15,
    "hf_wash_trading": true,
    "hf_renounced_mint": true,
    "hf_liquidity_min_usd": 5000.0
  }
}
```

#### `PUT /api/strategy`

Update strategy settings (partial update accepted).

**Request:**
```json
{
  "daily_loss_cap": 5.0,
  "per_trade_risk": 2.5
}
```

Currently only alpha config is persisted; risk settings are planned for a future update.

---

### Settings

#### `GET /api/settings`

Returns app settings (API key status, theme, log level).

```json
{
  "theme": "dark",
  "log_level": "info",
  "api_keys": {
    "alph_dex": "configured",
    "openai": "configured",
    "anthropic": "missing",
    "ollama": "missing"
  }
}
```

API key values are never returned — only `"configured"` or `"missing"` status.

#### `PUT /api/settings`

Update settings.

**Request:**
```json
{
  "theme": "dark",
  "log_level": "info"
}
```

Currently a no-op (returns `{ "success": true }`). Persistence will be added in a future update.

---

## WebSocket (`/ws`)

### Connection

Connect to `ws://127.0.0.1:3000/ws`. The server immediately responds:

```json
{ "type": "connected" }
```

### Message Format

All messages are JSON-serialised `DataEvent` values:

```json
{
  "TrendingUpdated": [{ "symbol": "PEPE", ... }]
}
```

### Events

| Event | Payload | Description |
|-------|---------|-------------|
| `TrendingUpdated` | `Vec<TrendingToken>` | New trending data (every 10s) |
| `SmartMoneyActivity` | `Vec<SmartMoneyTrade>` | Recent smart money trades |
| `SignalReceived` | `TokenSignal` | GMGN/Alpha AI signal |
| `ConnectionError` | `(String, String)` | (endpoint, message) — used for notifications |
| `PriceUpdated` | `(String, f64)` | Token price update |
| `TokenLoaded` | `TokenDetail` | Token detail loaded |
| `KlineUpdated` | `(String, Vec<KlineCandle>)` | Kline data loaded |
| `TrenchesUpdated` | `Vec<TrenchToken>` | New trenches loaded |

### Error Handling

If a client lags behind, the server drops older messages and sends a lag notification. The client should handle reconnection on disconnect.

---

## Security Notes

1. **API keys are server-side only.** The frontend never receives API key values.
2. **CORS is permissive** (allows all origins) for development. Restrict to specific origins in production.
3. **No authentication** is implemented for the web API. Run on localhost only.
4. **Rate limits** apply to backend API calls (GMGN, Alph AI, DEX Screener) — they are managed server-side and not exposed to the client.
