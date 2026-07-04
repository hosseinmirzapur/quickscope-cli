# Alph AI API Reference (QuickScope Usage)

> Every Alph AI endpoint QuickScope uses.
> Source: [Alph-ai-agent/Skills](https://github.com/Alph-ai-agent/Skills)
> QuickScope calls these via `reqwest` (REST) + `tokio-tungstenite` (WebSocket).

---

## Auth

- Header: `Cookie: dex_cookie=<value>`
- Obtain: log into [alph.ai](https://alph.ai), open DevTools → Application → Cookies → copy `dex_cookie`.
- Validity: 14 days. QuickScope tracks expiry and warns 2 days before.
- v1 (paper trading): read-only. **No `order/create` (real trading).**

Base URL: `https://b.alph.ai/smart-web-gateway`

---

## market (alphai-market)

| Endpoint | Method | QuickScope Use |
|---|---|---|
| `/token/token-detail` | GET | **One-shot token detail** (price, MC, liquidity, security, social, **AI description**) — recommended primary detail endpoint |
| `/ticker/currentPrice` | GET | Real-time price |
| `/ticker/24h` | GET | 24h stats |
| `/kline/new/history` | GET | K-line (query params: chain, token, type) |
| `/sherlock/popular_token/tokenPage` | GET | Popular tokens |
| `/snipe/platform/{chain}` | GET | Available platforms (required before snipe lists) |
| `/snipe/list/new/{chain}` | POST | New tokens (latest launches) |
| `/snipe/list/aimost/{chain}` | POST | **AI-recommended new tokens** (unique signal) |
| `/snipe/list/graduated/{chain}` | POST | Graduated tokens (bonding curve → DEX) |
| `/snipe/homepage` | GET | Token homepage (community, twitter, dev) |

### Critical: `platform` param required for snipe lists

- SOL chain: `platform` = `"All"`
- BSC chain: `platform` = platform ID combo (e.g., `"3,13"`)

### Snipe filter params

`minAge`/`maxAge` (minutes), `minMarketCap`/`maxMarketCap`, `minLiquidityUsdt`/`maxLiquidityUsdt`, `minHoldings`/`maxHoldings`, `minKolCalls`/`maxKolCalls`, `label` (array, e.g., `["MEME"]`), `hasTwitter`, `hasWebsite`, `bondingCurve` (1=on curve).

### Token-detail key fields

- `tokenPriceUsdt` (USDT price — recommended, intuitive)
- `marketCap` (USD)
- Security info (open source, locked, honeypot, tax rates)
- Social media (twitter, telegram, website)
- **AI-generated description and narrative** (unique to Alph AI)

---

## smart (alphai-smart)

| Endpoint | Method | QuickScope Use |
|---|---|---|
| `/smart/smart-wallet` | GET | Smart wallet list (by chain/tag) |
| `/smart/wallet` | GET | Single wallet detail |
| `/smart/search` | GET | Search smart wallet by address |
| `/smart/holding-tokens` | GET | Wallet's held tokens |
| `/smart/wallet-activity` | GET | Wallet trade history |
| `/smart/wallet-profit-loss` | GET | **Wallet PnL breakdown** (richer than GMGN) |
| `/smart/hot-tokens` | GET | 1h hot tokens (smart money buys) |
| `/smart/tags` | GET | Wallet tag list |
| `/signal/rank-list` | GET | **24h signal rank (Gold/Silver/Copper)** |
| `/signal/list/latest` | GET | Latest signals (top 5) |
| `/signal/list` | GET | Signal paginated list |
| `/signal/list-by-token` | GET | Signals for a specific token |
| `/signal/time-axis` | GET | Signal timeline |

### Signal confidence levels

| Level | Meaning |
|---|---|
| Gold | Highest confidence signal |
| Silver | Medium confidence |
| Copper | Lower confidence |

Each signal includes: `tokenInfo`, `smartInfo` (smart wallet count + trades), `kolInfo` (KOL mentions + followers), `multiple` (gain since first push), `pushType` (`[SMART, KOL_CALL]`).

---

## twitter (alphai-twitter) — THE key differentiator

| Endpoint | Method | QuickScope Use |
|---|---|---|
| `/tracker/x/follow` | POST | Follow/unfollow X account (operationType: 1=follow, 0=unfollow) |
| `/tracker/x/config` | POST | Configure monitoring (send/retweeted/replied_to/quoted/follow/profile/icon/nick/banner) |
| `/tracker/x/configList` | GET | Current monitoring config |
| `/tracker/x/monitorList` | POST | Monitored tweets list (paginated) |
| `/tracker/x/myList` | POST | My monitored KOL list |
| `/tracker/x/hotList` | POST | Hot monitoring list |
| `/tracker/x/transTexts` | POST | Translate tweet text |
| **`/token/twitter-search`** | GET | **Extract token CA from a tweet URL** — massive alpha signal |
| `/x/detail` | POST | X user detail (followers, bio) |
| **`/x/search`** | POST | **Tweet keyword search** (sentiment, mention tracking) |
| `/x/tweets` | POST | User's tweet list (by user ID) |

### Tweet data fields

| Field | Meaning |
|---|---|
| `type` | send/retweeted/replied_to/quoted/follow/profile/icon/nick/banner |
| `id` | Tweet ID |
| `created_at` | ISO 8601 |
| `text` | Tweet body |
| `tweets.text` | Original content (for replies/quotes) |
| `referenced.text` | Referenced tweet body |
| `public_metrics` | likes, retweets, replies, impressions |

### twitter-search response

```json
{
  "data": [
    { "chain": "sol", "tokenAddress": "0x...", "poolLiquidityUsdt": 16377.57 }
  ]
}
```

---

## WebSocket (alphai-websocket) — real-time feeds

### Connection

1. `POST /ws/listenkey` (with dex_cookie) → returns `listenKey` (1h expiry)
2. Connect `wss://ws.alph.ai/stream/ws?listenKey=<key>`
3. Send subscribe messages
4. Auto-renew listenKey before expiry
5. Respond to server ping with pong (carrying timestamp)

### Subscribe format

```json
{
  "id": "req-001",
  "event": "SUBSCRIBE",
  "params": [{"chain": "sol", "token": "0x...", "type": "kline", "scale": "1m"}]
}
```

### Subscription types QuickScope uses

| Type | Needs token | Needs listenKey | QuickScope Use |
|---|---|---|---|
| `kline` | yes | no | Real-time price for open positions (TP/SL monitor) |
| `smart_trade` | yes | no | Live smart money trades (Dashboard) |
| `kol_call` | yes | no | KOL mentions push (Analyzer) |
| `new_token` | no | no | New token launches (Scanner) |
| `signal` | no | no | Gold/Silver/Copper signals (toasts) |

**Private types (need listenKey) — NOT used in v1:** `order`, `position`, `swap`, `pending_order`, `user_tracker_x`.

### kline push format

```json
{"e": "kline", "C": "sol", "t": "0x...", "s": "1m",
 "d": [{"i": 123, "o": "0.001", "c": "0.0012", "h": "0.0013", "l": "0.0009",
        "v": "10000", "ba": "6000", "sa": "4000", "bc": 50, "sc": 30}]}
```

- `o/c/h/l` — open/close/high/low
- `v` — volume
- `ba/sa` — buy/sell amount
- `bc/sc` — buy/sell count

---

## Rate Limits

Not formally documented. Cookie-based auth. **Be conservative:**
- Cache REST responses aggressively (TTL 30-120s depending on data).
- Prefer WebSocket over polling for real-time data.
- On repeated failures, back off exponentially.

---

## OFF-LIMITS for v1 (real trading / account management)

- `POST /order/create` — real trading
- Any `/user/*` account management (registration, wallet creation)
- `/ws` private subscription types (order, position, swap, pending_order)

QuickScope is paper-trading only. These endpoints are documented for completeness but must not be implemented in v1.
