# Data Sources

> How GMGN + Alph AI + DEX Screener combine to feed QuickScope.
> See `docs/api-reference/` for endpoint-by-endpoint detail.

---

## Source Roles

| Source | Role | Why |
|---|---|---|
| **GMGN** (primary) | Backbone — market data, token analytics, holders, portfolio | Best Solana memecoin data, proper Ed25519 API key, documented rate limits, finest trending granularity (1m/5m/1h/6h/24h). Also provides `market trenches` for newly launched tokens. |
| **Alph AI** (secondary) | Twitter/X monitoring + WebSocket real-time with auto-reconnect + signal confidence | Fills GMGN's gaps: Twitter is invisible to GMGN; WebSocket eliminates polling; Gold/Silver/Copper signals add a scoring dimension. WebSocket reconnects with exponential backoff (1s → 2s → ... → 60s max). |
| **DEX Screener** (tertiary) | Cross-reference trending/boosts | "Boosted on DEX Screener AND trending on GMGN" = stronger signal than either alone |

---

## What Each Source Provides

### GMGN (via `gmgn-cli` subprocess, read-only for v1)

| Data | Endpoints |
|---|---|
| Trending tokens | `market trending` (1m/5m/1h/6h/24h) — sorted by volume, swaps, marketcap, etc. |
| New token discovery | `market trenches` (Pump.fun, letsbonk, Moonshot, etc.) — 3 lifecycle stages: `new_creation`, `near_completion`, `completed` |
| Price charts | `market kline` (30s to 1d) |
| Real-time signals | `market signal` (price spikes, smart money buys, large buys, CTO) |
| Hot searches | `market hot-searches` — tokens people are searching for right now |
| Token info | `token info` (price, MC, liquidity, dev, social, launchpad) |
| Token security | `token security` (rug_ratio, honeypot, wash trade, renounced) |
| Token holders | `token holders` (with wallet tags: smart_degen, KOL, sniper, bundler) |
| Token traders | `token traders` (with PnL) |
| Wallet portfolio | `portfolio holdings/activity/stats/created-tokens` |
| Smart money/KOL trades | `track smartmoney`, `track kol`, `track follow-wallet` |
| Paper pricing | `swap order quote` (no private key needed) |

**Auth:** `GMGN_API_KEY` configured via `gmgn-cli config --apply <KEY>`. Ed25519 signing handled by the CLI.

**Rate limits:** Leaky bucket, rate=20, capacity=20. Endpoint weights 1-5. Documented in `docs/api-reference/gmgn-endpoints.md`.

### Alph AI (REST + WebSocket, read-only for v1)

| Data | Endpoints |
|---|---|
| Token detail (one-shot) | `GET /token/token-detail` (price, MC, liquidity, security, social, **AI description**) |
| Popular tokens | `GET /sherlock/popular_token/tokenPage` |
| New tokens (AI-recommended) | `POST /snipe/list/aimost/{chain}` — unique `aimost` signal |
| New tokens (latest) | `POST /snipe/list/new/{chain}` |
| Graduated tokens | `POST /snipe/list/graduated/{chain}` |
| Smart wallets | `GET /smart/smart-wallet`, `/smart/wallet`, `/smart/holding-tokens`, `/smart/wallet-activity`, `/smart/wallet-profit-loss` |
| Smart wallet hot tokens | `GET /smart/hot-tokens` (1h, smart money buys) |
| Signals | `GET /signal/rank-list` (Gold/Silver/Copper), `/signal/list/latest` |
| **Twitter/X monitoring** | `POST /tracker/x/follow`, `/tracker/x/config`, `/tracker/x/monitorList`, `/x/search`, `/x/tweets`, `/x/detail` |
| **Tweet → token CA** | `GET /token/twitter-search` — extract contract addresses from a tweet URL |
| **WebSocket** | kline, ticker, smart_trade, kol_call, new_token, signal — real-time push. **Auto-reconnects** with exponential backoff. |

**Auth:** `dex_cookie` (browser cookie from alph.ai, 14-day expiry). Stored in `~/.config/quickscope/.env`.

**WebSocket Reconnect Behavior:**
- On disconnect: waits 1 second, retries
- Each failure doubles the wait: 2s → 4s → 8s → ... → 60s max
- Resets to 1s on successful connection
- Sends ping/pong responses to keep-alive
- Subscriptions are restored automatically on reconnect

### DEX Screener (REST, free)

| Data | Endpoints |
|---|---|
| Token/pair search | `GET /latest/dex/search` |
| Token boosts | `GET /token-boosts/latest`, `/token-boosts/top` |
| Trending pairs | `GET /latest/dex/search?q=...` |

**Auth:** None. Free tier.

---

## Data Orchestration

`DataOrchestrator` (`src/data/mod.rs`) is the single facade the rest of the app uses. It:

1. **Receives a high-level request** (e.g., "give me trending tokens with alpha scores").
2. **Fan-outs to the right sources** (GMGN trending + Alph AI signals + DEX Screener boosts).
3. **Merges results** into unified `data::models` types.
4. **Caches** responses (TTL-based LRU) to avoid hammering APIs.
5. **Emits `DataEvent`s** back to the UI thread via `tokio::mpsc`.

### Cache TTLs

| Data | TTL | Reason |
|---|---|---|
| Trending tokens | 30s | Changes fast but not instantly |
| Trenches (new tokens) | 30s | Fresh launches are time-sensitive |
| Token info/security | 60s | Stable for a minute |
| K-line (historical) | 10s | Live trading needs freshness |
| Wallet portfolio | 120s | Doesn't change fast |
| Twitter mentions | 30s | Fresh signal matters |
| Signals | 15s | Time-sensitive |
| Watchlist | 60s | User-managed list |

### WebSocket vs Polling

| Data | Transport | Why |
|---|---|---|
| K-line for open positions | **Alph AI WebSocket** | Real-time TP/SL monitoring needs instant updates |
| Smart money trades | **Alph AI WebSocket** | Live Dashboard feed |
| New token launches | **Alph AI WebSocket** | Instant Scanner updates |
| Signals | **Alph AI WebSocket** | Toast notifications on Gold/Silver |
| Trending lists | REST poll (GMGN) | Bulk data, changes less often |
| Trenches lists | REST poll (GMGN) | Bulk data, changes less often |
| Token deep-analysis | REST on-demand (GMGN) | User-triggered |

### Scanner Mode Selector

The Scanner tab uses a **mode selector** to pick the data source:

| Mode | Source | Command | Description |
|---|---|---|---|
| **Trending** | GMGN | `market trending` | Volume/swap-sorted trending tokens |
| **Trenches** | GMGN | `market trenches` | Newly launched tokens (new_creation stage) |
| **Watchlist** | Filtered | Filtered `trending` | Trending tokens that are on your watchlist |
| **AI-Rec** | Alph AI | `signal/rank-list` | Tokens with Gold/Silver confidence signals |

Switch modes with `←`/`→` arrow keys when in the Scanner tab. Press `r` to refresh the current mode's data.

### Address Search

A ` by Address` command palette action lets users look up any Solana token by contract address:
1. Open command palette (`Ctrl+P`)
2. Select " by Address"
3. Paste or type the contract address
4. Press Enter → fetches token detail from GMGN
5. Automatically switches to Analyzer tab with results

---

## Combining the Three Sources

### Alpha Score Integration

The Alpha Filter's feature vector pulls from all three:

```
GMGN-driven features (majority):
  volume, swaps, liquidity, market_cap, rug_ratio, holder stats,
  wallet tags, dev profile, launchpad, kline-derived momentum

Alph AI-driven features (social + signal):
  twitter_mentions_1h, twitter_sentiment, twitter_follower_count,
  twitter_ca_extracted, signal_confidence (Gold/Silver/Copper),
  smart_wallet_pnl

DEX Screener-driven features (validation):
  dexscr_boost (already in GMGN dev profile, cross-referenced),
  dexscr_trending_bar
```

### Cross-Source Validation

A token that is:
- Trending on GMGN (1h, top 20 by volume) **AND**
- Has a Gold signal on Alph AI **AND**
- Is boosted on DEX Screener

...is a much stronger candidate than one hitting only a single source. The `DataOrchestrator` surfaces these overlaps as a "conviction multiplier" in the Alpha Score.

---

## Auth & Security Notes

- **GMGN API key:** Configured via `gmgn-cli config --apply <KEY>`. Read-only for v1 — no private key needed.
- **Alph AI cookie:** Log into `alph.ai`, copy `dex_cookie` from browser DevTools. 14-day expiry. Read-only for v1.
- **Never commit credentials.** `.env` is gitignored. `.env.example` documents the keys without values.
- **Missing API keys trigger a fatal error modal** on startup with a clear message pointing to the `.env` file.
- **IPv6:** GMGN does not support IPv6. Force IPv4 in `reqwest` client config if 401/403 errors appear.
