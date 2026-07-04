# GMGN API Reference (QuickScope Usage)

> Every GMGN endpoint QuickScope uses, with rate-limit weights.
> Source: [GMGN Agent Skills](https://github.com/GMGNAI/gmgn-skills)
> QuickScope calls these via `reqwest` (REST), NOT via `gmgn-cli`.

---

## Auth

- Header: `X-APIKEY: <GMGN_API_KEY>`
- v1 (paper trading): read-only endpoints only. **No Ed25519 signing required.**
- Demo key for testing: `gmgn_solbscbaseethmonadtron` (testing only — get a personal key from `gmgn.ai/ai`).

## Rate Limiting

Leaky bucket: `rate=20`, `capacity=20`.
- Sustained throughput ≈ `20 ÷ weight` requests/second.
- Max burst ≈ `floor(20 ÷ weight)`.
- On 429: read `X-RateLimit-Reset` header or `reset_at` in body, sleep until reset, retry once. Never spam.

## Chain

All endpoints: `--chain sol` (QuickScope is Solana-only).

---

## market (gmgn-market)

| Endpoint | Route | Weight | QuickScope Use |
|---|---|---|---|
| `market kline` | `GET /v1/market/token_kline` | 2 | Price charts (Analyzer, Journal), TP/SL monitor |
| `market trending` | `GET /v1/market/rank` | 1 | Dashboard pulse, Scanner feed |
| `market trenches` | `POST /v1/trenches` | 3 | New token discovery (Scanner) |
| `market signal` | `POST /v1/market/token_signal` | 3 | Real-time signals (Dashboard) |
| `market hot-searches` | `POST /v1/market/hot_searches` | 3 | Hot token cross-reference |

### Key params

- `market trending`: `--interval` (1m/5m/1h/6h/24h), `--order-by` (volume/marketcap/liquidity/smart_degen_count/...), `--filter` (SOL defaults: `renounced frozen`), `--platform` (Pump.fun, letsbonk, ...), `--min-*`/`--max-*` range filters.
- `market kline`: `--address`, `--resolution` (30s/1m/5m/15m/1h/4h/1d), `--from`/`--to` (Unix seconds).
- `market trenches`: `--type` (new_creation/near_completion/completed).

### Critical field notes

- `volume` = USD value; `amount` = token units (naming is counterintuitive).
- `rug_ratio`: 0-1, >0.30 high risk.
- `smart_degen_count` / `renowned_count`: smart money / KOL wallet counts.
- `renounced_mint` / `renounced_freeze_account`: SOL-specific safety baseline (both should be true).
- `is_honeypot`: EVM-only, always empty on SOL.

---

## token (gmgn-token)

| Endpoint | Route | Weight | QuickScope Use |
|---|---|---|---|
| `token info` | `GET /v1/token/info` | 1 | Alpha Analyzer: price, MC, liquidity, dev, social |
| `token security` | `GET /v1/token/security` | 1 | Rug detection, hard filters |
| `token pool` | `GET /v1/token/pool_info` | 1 | Pool details |
| `token holders` | `GET /v1/market/token_top_holders` | 5 | Holder breakdown by wallet tag |
| `token traders` | `GET /v1/market/token_top_traders` | 5 | Top traders with PnL |

### Key params

- `token holders`/`traders`: `--tag` (smart_degen/renowned/fresh_wallet/dev/sniper/rat_trader/bundler/dex_bot/bluechip_owner), `--order-by` (amount_percentage/profit/unrealized_profit/buy_volume_cur/sell_volume_cur).

### Response objects

- `token info`: nested `pool`, `dev`, `link`, `stat`, `wallet_tags_stat`, `price`, `fee_distribution`.
- `dev` object includes: `creator_address`, `creator_token_status` (hold/close), `top_10_holder_rate`, `cto_flag`, `dexscr_ad`/`dexscr_boost`/`dexscr_trending_bar`, `creator_open_count`, `ath_token_info`.
- `wallet_tags_stat`: `smart_wallets`, `renowned_wallets`, `sniper_wallets`, `rat_trader_wallets`, `bundler_wallets`, `whale_wallets`, `fresh_wallets`.

---

## portfolio (gmgn-portfolio)

| Endpoint | Route | Weight | Auth | QuickScope Use |
|---|---|---|---|---|
| `portfolio info` | `GET /v1/user/info` | 1 | exist | Wallet balances (future) |
| `portfolio holdings` | `GET /v1/user/wallet_holdings` | 5 | **critical** | Wallet holdings (future) |
| `portfolio activity` | `GET /v1/user/wallet_activity` | 3 | exist | Tx history (future) |
| `portfolio stats` | `GET /v1/user/wallet_stats` | 3 | exist | Win rate, PnL (future) |
| `portfolio token-balance` | `GET /v1/user/wallet_token_balance` | 1 | exist | Token balance (future) |
| `portfolio created-tokens` | `GET /v1/user/created_tokens` | 2 | exist | Dev profiling |

**Note:** `exist` auth = API key only. `critical` auth = API key + Ed25519 signed (not needed for v1 read-only, but `holdings` requires it — skip for v1 or use Alph AI equivalent).

---

## track (gmgn-track)

| Endpoint | Route | Weight | QuickScope Use |
|---|---|---|---|
| `track follow-tokens` | `GET /v1/user/follow_tokens` | 3 | Watchlist sync |
| `track follow-wallet` | `GET /v1/trade/follow_wallet` | 3 | Followed wallet trades (needs signed auth) |
| `track kol` | `GET /v1/user/kol` | 1 | KOL trade feed (Dashboard) |
| `track smartmoney` | `GET /v1/user/smartmoney` | 1 | Smart money trade feed (Dashboard, Analyzer) |

### Key fields (kol/smartmoney)

- `transaction_hash`, `maker`, `side` (buy/sell), `base_address`, `amount_usd`, `price_usd`, `price_change` (ratio since trade), `is_open_or_close` (0=open/add, 1=close/reduce), `timestamp`, `maker_info.tags`.

### Signal strength framework

| Level | Criteria |
|---|---|
| Weak | 1 KOL buys |
| Medium | 2-3 smart money same direction, OR 1 full position open |
| Strong | ≥3 smart money same direction within 30 min (cluster) |
| Very Strong | Cluster + full position opens + KOL joining |

---

## swap (gmgn-swap) — paper pricing only

| Endpoint | Route | Weight | QuickScope Use |
|---|---|---|---|
| `order quote` | `GET /v1/trade/quote` | 2 | **Paper trade pricing** (no private key needed) |

**All other swap endpoints OFF-LIMITS for v1** (real-money execution):
- `swap`, `multi-swap`, `order strategy create/cancel/list`, `order get`.

`order quote` needs only `GMGN_API_KEY` (exist auth) — safe for paper pricing.

---

## cooking (gmgn-cooking) — stats only

| Endpoint | Route | Weight | QuickScope Use |
|---|---|---|---|
| `cooking stats` | (TBD) | 1 | Launchpad token creation trends |

**`cooking create` OFF-LIMITS for v1** (real token deployment).

---

## Currency Addresses (SOL)

| Token | Address |
|---|---|
| SOL (native) | `So11111111111111111111111111111111111111112` |
| USDC | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` |

**Never guess these.** A wrong address causes silent failures.
