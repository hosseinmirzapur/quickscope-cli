# QuickScope — Design Specification

**Date:** 2026-07-05
**Status:** Approved
**Author:** brainstorming session (smartinex + ZCode)

---

## 1. Overview

**QuickScope** is a Rust-based terminal user interface (TUI) for memecoin alpha hunting and paper trading on Solana. It operationalizes the `memecoin-alpha-agent` skill — alpha-selection filtering, momentum-explosion detection, scalping, rug detection, risk discipline, and learning — into an interactive, OpenCode-style terminal application that feels nothing like a traditional terminal.

**Core promise:** Discover tokens, analyze them with a multi-dimensional Alpha Filter, paper-trade them with realistic simulation, and let the system learn from every outcome — continuously auto-tuning its thresholds and offering LLM-assisted post-mortems.

**Non-goals (v1):**
- No real-money trading. Paper trading only.
- No multi-chain. Solana only.
- No token creation (launchpad cooking).
- No social media posting.

---

## 2. Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Artifact type | TUI application | OpenCode-style, high interactivity, keyboard + mouse |
| Trading mode | Paper trading (simulated) | Learn without real-money risk; train on real market data |
| Learning system | Auto-tuner (statistical) + LLM post-mortem (on-demand) | Continuous deterministic improvement + smart reviewer |
| Chain | Solana only | Focused; where memecoin action is |
| Primary data source | GMGN API (REST, Ed25519 API key) | Best Solana memecoin data, proper auth, documented rate limits |
| Secondary data source | Alph AI API (REST + WebSocket, dex_cookie) | Twitter/X monitoring, real-time WS feeds, signal confidence levels — fills GMGN's gaps |
| Tertiary data source | DEX Screener API | Cross-reference trending/boosts |
| LLM provider | Pluggable (OpenAI + Anthropic + Ollama) | Future-proof, swap based on cost/quality |
| Storage | SQLite via `sqlx` | Embedded, single-file, async, relational queries |
| Architecture | Monolithic with feature modules (single crate) | Fast compile, zero cross-crate plumbing, easy iteration |

---

## 3. Core Architecture & Event Loop

### 3.1 App Lifecycle

```
[main.rs]
  └─→ tokio::runtime (async)
       └─→ AppState { ... }
            ├─→ TUI render loop (ratatui, ~60fps via crossterm)
            ├─→ Data poller (async tasks, channels)
            ├─→ User input handler (crossterm events → commands)
            └─→ Background workers (learning tuner, signal monitor, TP/SL monitor)
```

**Pattern:** Elm/TEA-inspired — `update()` processes events and returns commands, `view()` renders state.

- **State** (`AppState`): Holds all tab states, data caches, trade journal, settings, paper portfolio. Lives behind `tokio::sync::Mutex` so async tasks can safely read/write.
- **Event** (`AppEvent`): `Key(key)`, `Mouse(mouse)`, `Data(DataEvent)`, `Tick`, `Trade(TradeEvent)`, `LLM(LLMResponse)`.
- **Message** (`AppCommand`): `FetchTrending`, `AnalyzeToken(addr)`, `BuyPaper(token, amount)`, `RunPostMortem`, `SwitchTab(tab)`, etc.
- **Render** (ratatui `Frame`): Each tab has its own `render(frame, area, state)` function.

### 3.2 Threading Model

| Thread | Role |
|---|---|
| Main | Crossterm event polling + ratatui rendering (sync, ~60fps) |
| `tokio` worker pool | HTTP requests (GMGN, Alph AI, DEX Screener, LLM APIs), SQLite writes |
| Dedicated tasks | Auto-tuner (periodic), TP/SL monitor (2s poll), Alph AI WebSocket listener, daily reset (midnight UTC) |

Communication: `tokio::mpsc` channels route `DataEvent`s, `TradeEvent`s, and `LLMResponse`s back to the main render loop.

---

## 4. TUI Layout & Interactivity

### 4.1 Overall Layout

```
┌──────────────────────────────────────────────────────┐
│  ⚡ QuickScope  Balance: 50.00 SOL  PnL: +0.00 ...  │  ← Status bar (top, full width)
├──┬───────────────────────────────────────────────────┤
│⬡ │                                                   │
│⌕ │                                                   │
│◎ │              Active Tab Content                    │
│⟠ │          (sidebar | content layout)                │
│☰ │                                                   │
│⚙ │                                                   │
│◆ │                                                   │
├──┴───────────────────────────────────────────────────┤
│  ↑↓:Navigate  r:Refresh  Ctrl+P:Commands  q:Quit    │  ← Keybinding hints (bottom)
└──────────────────────────────────────────────────────┘
```

### 4.2 Navigation Model

**Keyboard (OpenCode-style):**
- `↑` / `↓` — Move focus within lists/tables (no VIM-style `j`/`k`)
- `PageUp` / `PageDown` — Scroll lists faster
- `Tab` / `Shift+Tab` — Cycle tabs in sidebar
- `Enter` — Select item / drill into detail
- `Escape` — Back / close popup / dismiss modal
- `/` — Search/filter (live filtering of token lists)
- `Space` — Toggle watch/star
- `?` — Help overlay (context-sensitive per tab)
- `q` or `Ctrl+C` — Quit (with confirmation modal if trades active)
- `Ctrl+P` — Command palette (searchable overlay with 14 commands)
- `Ctrl+B` — Toggle sidebar collapsed/expanded
- `Ctrl+E` — Emergency exit all positions
- `r` — Refresh data

**Mouse (full support via crossterm):**
- Click sidebar icons to switch tabs
- Click rows to select/focus
- Scroll wheel for lists
- Right-click for context menu (copy address, open in browser, analyze)

### 4.3 Theme System

Two built-in themes:
- **Dark (default)** — Deep navy/charcoal background, green/red for PnL, amber for warnings, cyan accents.
- **Degen Mode** — Black background, neon green/magenta, CRT glow on highlights.

Theme struct uses semantic color tokens (`primary`, `success`, `danger`, `warning`, `muted`, `surface`, `text`, `highlight`).

### 4.4 Tabs

#### Tab 1: Dashboard
- Left: Paper portfolio overview — balance, PnL breakdown, active positions with live price updates.
- Right top: Trending tokens from GMGN (`market trending`), interval selector (1m/5m/1h/24h).
- Right bottom: Real-time smart money signals from GMGN (`track smartmoney`) and KOL activity.

#### Tab 2: Token Scanner
- Real-time feed from GMGN `market trending` + `market trenches`, plus Alph AI `snipe/list/aimost`.
- Interval toggle, platform filter (Pump.fun, letsbonk, all).
- Server-side filtering via GMGN (`--filter`, `--min-liquidity`, `--max-insider-rate`, etc.).
- Alpha Score column (0-100).
- Expand-on-select detail panel: security, holders, dev status.
- Action buttons: `[Analyze]`, `[Paper Buy]`, `[Watch ★]`.

#### Tab 3: Alpha Analyzer
- Deep-dive on a single token — triggered from Scanner or by entering a CA.
- Left: price info, sparkline K-line chart (GMGN `kline`), wallet breakdown bar chart.
- Right: full security audit (GMGN `token security`), holder concentration bars, dev profile.
- Bottom left: Smart money/KOL trade activity (GMGN `track smartmoney`, Alph AI smart wallets).
- Bottom right: Full Alpha Filter score breakdown with mode recommendation.
- Twitter panel: recent KOL tweets mentioning the token (Alph AI `x/search`).

#### Tab 4: Trade Terminal
- Paper buy/sell with real GMGN pricing (via `order quote` or live kline price).
- Slippage simulation, TP/SL planning (paper).
- Mode selector: EXPLODE / SCALP / FALLBACK.
- Quick action buttons for common amounts.
- Risk bar at bottom: daily loss tracker, current mode.
- Confirmation modal before every trade.

#### Tab 5: Trade Journal
- Full history of every paper trade with all features/scores at time of entry.
- Filter by win/loss/token/date.
- Expand to see full feature vector.
- Session stats with aggregate performance.
- "Avg score of winners vs losers" insight (feeds auto-tuner).
- `[Postmortem]` button triggers LLM session review.

#### Tab 6: Strategy & Learning
- Left: Current alpha filter weights/thresholds with auto-tuning history log.
- Right: LLM post-mortem panel — select period, provider, hit RUN.
- Each LLM suggestion has `[Apply]` / `[Dismiss]` buttons.
- Auto-tune trigger indicator (e.g., "runs every 20 trades, next: 6 away").

#### Tab 7: Settings
- Sectioned with bordered blocks.
- API connection status with live rate limit indicator.
- Paper trading config: balance, caps, defaults.
- Theme switcher, refresh rate, mouse/animation toggles.
- Risk profile: aggression slider, auto-tune frequency, kill-switch toggle.

### 4.5 Shared UI Components

| Widget | Description |
|---|---|
| `Modal` | Centered overlay with `Clear` backdrop (SweetAlert-style). Used for help, confirmations, emergency exit |
| `CommandPalette` | Ctrl+P searchable overlay with 14 commands — tab switching, actions, toggles |
| `Toast` | Top-right notification with 4s auto-dismiss. Info/Success/Warning/Error styles |
| `Sparkline` | Mini chart from K-line data (block chars) |
| `ProgressBar` | Horizontal bar with color coding |
| `Tag` | Colored chip (e.g., `[Pump.fun]`, `[CTO]`, `[RENOUNCED]`) |
| `ContextMenu` | Right-click popup menu |
| `Table` | Sortable, filterable, scrollable table with headers |
| `SearchBar` | `/` activated inline search with live filtering |
| `Chart` | Larger OHLCV candlestick + volume (unicode box drawing) |
| `Sidebar` | Persistent VS Code-style activity bar with 7 tab icons, collapsible via Ctrl+B |

---

## 5. Data Layer

### 5.1 GMGN Client (Primary)

Direct REST API calls from Rust using `reqwest` (no shelling out to `gmgn-cli`):
- Async non-blocking
- Typed responses (deserialize JSON into Rust structs)
- Full rate limit control (leaky bucket, rate=20, capacity=20)
- No Node.js dependency — pure Rust binary

**Module structure:**
```
src/data/gmgn/
├── mod.rs            # GmgnClient struct, auth, rate limiter
├── market.rs         # kline, trending, trenches, signal, hot-searches
├── token.rs          # info, security, pool, holders, traders
├── portfolio.rs      # info, holdings, activity, stats, token-balance, created-tokens
├── track.rs          # follow-tokens, follow-wallet, kol, smartmoney
├── quote.rs          # order quote (paper pricing)
└── types.rs          # All response structs
```

**Auth (v1):** Only `GMGN_API_KEY` (X-APIKEY header). No Ed25519 signing needed for read endpoints.

**Rate Limiter:** Each endpoint has a known weight (1-5). Token-based semaphore wrapper; on 429, read `reset_at`, sleep until reset, retry once; on repeated 429, surface error to UI.

**Endpoints used (read-only):**
| Skill | Endpoints |
|---|---|
| gmgn-market | `kline`, `trending`, `trenches`, `signal`, `hot-searches` |
| gmgn-token | `info`, `security`, `pool`, `holders`, `traders` |
| gmgn-portfolio | `info`, `holdings`, `activity`, `stats`, `token-balance`, `created-tokens` |
| gmgn-track | `follow-tokens`, `follow-wallet`, `kol`, `smartmoney` |
| gmgn-swap | `order quote` only (for paper pricing) |
| gmgn-cooking | `stats` only (launchpad trends) |

### 5.2 Alph AI Client (Secondary)

REST + WebSocket using `reqwest` + `tokio-tungstenite`. Fills GMGN's gaps with Twitter/X monitoring and real-time WebSocket feeds.

**Module structure:**
```
src/data/alph_ai/
├── mod.rs           # AlphAIClient, dex_cookie auth
├── market.rs        # token-detail, popular, snipe/aimost/new/graduated
├── smart.rs         # smart wallets, trackers, signals (Gold/Silver/Copper)
├── twitter.rs       # KOL monitoring, tweet scraping, CA extraction
├── websocket.rs     # Real-time: kline, ticker, smart_trade, kol_call, new_token, signal
└── types.rs
```

**Auth:** `dex_cookie` (browser cookie, 14-day expiry). Stored in `~/.config/quickscope/.env`. QuickScope warns 2 days before expiry.

**Why Alph AI (complementary to GMGN):**
- **Twitter/X monitoring** — GMGN has zero; Alph AI has full KOL tweet tracking + CA extraction from tweets
- **WebSocket real-time** — eliminates polling for kline, smart trades, new tokens, signals
- **AI-recommended new tokens** (`aimost`) — unique signal
- **Signal confidence levels** (Gold/Silver/Copper) — extra Alpha Score dimension
- **Smart wallet PnL analytics** — richer than GMGN's wallet data

**WebSocket subscription types used:**
| Type | Purpose |
|---|---|
| `kline` | Real-time price updates push to TUI |
| `smart_trade` | Live smart money buys/sells in Dashboard |
| `new_token` | New Pump.fun launches appear in Scanner instantly |
| `signal` | Gold/Silver/Copper signals as toast notifications |
| `kol_call` | KOL mentions push to Analyzer Twitter panel |

### 5.3 DEX Screener Client (Tertiary)

Cross-references GMGN trending/boosts. "Token boosted on DEX Screener AND trending on GMGN" = stronger signal than either alone.

```
src/data/dex_screener/
├── mod.rs            # DexScreenerClient
├── search.rs         # /latest/dex/search
├── trending.rs       # /token-boosts/latest, /trending-new-pairs
└── types.rs
```

### 5.4 Data Orchestration

```
src/data/mod.rs
├── DataOrchestrator   # High-level facade: "give me trending tokens"
│                       #   internally calls GMGN + Alph AI + DEX Screener, merges
├── DataCache           # In-memory LRU cache (ttl-based) to avoid hammering APIs
│                       #   trending cached 30s, kline 10s
└── DataEvent           # Enum of all async data events sent back to UI thread
    ├── TrendingUpdated(tokens)
    ├── TokenLoaded(token, detail)
    ├── KlineUpdated(token, candles)
    ├── SmartMoneyActivity(trades)
    ├── SignalReceived(signal)
    ├── TwitterMention(token, tweet)
    ├── RateLimitHit(endpoint, reset_at)
    └── ConnectionError(endpoint, error)
```

---

## 6. Alpha Filter & Scoring Engine

The brain of QuickScope — from the memecoin-alpha-agent skill's Alpha Filter Framework, adapted for real-time TUI operation.

### 6.1 Feature Vector (30+ dimensions per token)

| Category | Feature | Source |
|---|---|---|
| **MARKET MOMENTUM** | volume_1m, volume_5m, volume_1h, swaps_1h, price_change_1m, price_change_1h, hot_level | trending |
| **LIQUIDITY** | liquidity_usd, market_cap, pool_exchange, is_on_curve | token |
| **SECURITY / RUG** | rug_ratio, is_wash_trading, open_source, renounced_mint, renounced_freeze | security |
| **HOLDER** | holder_count, top_10_holder_rate, dev_team_hold_rate, creator_hold_rate, suspected_insider_hold, fresh_wallet_rate | token/stat |
| **WALLET SIGNALS** | smart_degen_count, renowned_count, sniper_count, bundler_rate, rat_trader_rate | stat |
| **DEV PROFILE** | creator_status, creator_prev_tokens, creator_ath_mc, cto_flag, dexscr_ad, dexscr_boost | dev |
| **SOCIAL / META** | has_social_links, dexscr_trending_bar, launchpad_platform | link/dev |
| **TWITTER (Alph AI)** | twitter_mentions_1h, twitter_sentiment, twitter_follower_count, twitter_ca_extracted, signal_confidence, smart_wallet_pnl | alph_ai |

### 6.2 Scoring Pipeline

```
Raw GMGN + Alph AI Data
     │
     ▼
Feature Extraction → Category Scores (6 sub-scores) → Alpha Score (0-100)
                          │                                       │
                          ▼                                       ▼
                   Hard Filters (reject/OK)           Mode Select + Sizing
```

### 6.3 Category Scores (Weighted Sub-scores)

```
Momentum Score (weight: w_momentum)
  = normalize(volume_1h) * 0.3
  + normalize(swaps_1h) * 0.2
  + normalize(hot_level) * 0.2
  + clamp(price_change_1h, -100, +500) / 500 * 0.3

Safety Score (weight: w_safety)
  = (1 - rug_ratio) * 0.35
  + (is_wash_trading ? 0 : 1) * 0.15
  + (renounced_mint ? 1 : 0) * 0.15
  + (renounced_freeze ? 1 : 0) * 0.15
  + (1 - top_10_holder_rate) * 0.20

Holder Quality Score (weight: w_holder)
  = normalize(smart_degen_count) * 0.35
  + normalize(renowned_count) * 0.20
  + (1 - dev_team_hold_rate) * 0.20
  + (1 - suspected_insider_hold_rate) * 0.15
  + (1 - fresh_wallet_rate) * 0.10

Liquidity Score (weight: w_liquidity)
  = sigmoid(log10(liquidity_usd), midpoint=$50k) * 0.5
  + (1 - is_on_curve) * 0.3
  + sigmoid(log10(market_cap), midpoint=$10k) * 0.20

Dev Trust Score (weight: w_dev)
  = (creator_status == "creator_close" ? 0 : 0.5) * 0.30
  + normalize(creator_ath_mc, max=$1M) * 0.25
  + (cto_flag ? 0.8 : 0.3) * 0.20
  + (dexscr_boost ? 0.7 : 0.3) * 0.15
  + clamp(creator_prev_tokens, 0, 10) / 10 * 0.10

Social Score (weight: w_social) [Alph AI enhanced]
  = normalize(twitter_mentions_1h) * 0.30
  + twitter_sentiment_score * 0.20
  + normalize(twitter_follower_count) * 0.20
  + signal_confidence_score(Gold=1,Silver=0.6,Copper=0.3) * 0.30
```

**Final Alpha Score:**
```
alpha = w_momentum * Momentum
      + w_safety     * Safety
      + w_holder     * HolderQuality
      + w_liquidity  * Liquidity
      + w_dev        * DevTrust
      + w_social     * Social
```

All weights `w_*` start at defaults but are mutable — the auto-tuner adjusts them.

### 6.4 Hard Filters (Reject Instantly)

| Filter | Threshold | Rationale |
|---|---|---|
| `rug_ratio` | > 0.30 | High rug pull likelihood |
| `dev_team_hold_rate` | > 0.15 | Dev holds too much, can dump |
| `is_wash_trading` | true | Artificial volume |
| `renounced_mint` | false (on SOL) | Can mint more tokens |
| `creator_status` | `creator_hold` + `dev_team_hold_rate > 0.10` | Dev still holding AND large allocation |
| `liquidity_usd` | < $5,000 | Can't exit without massive slippage |

Hard filters are configurable (user can relax in Settings).

### 6.5 Mode Selection

```
Mode: EXPLODE
  Trigger: alpha >= 75 AND momentum >= 80 AND safety >= 70
  Sizing:  0.5-1.0 SOL (paper)
  Exit:    TP +100-300%, SL -60%, trailing after +50%

Mode: ALPHA
  Trigger: alpha >= 55 AND safety >= 65
  Sizing:  0.2-0.5 SOL (paper)
  Exit:    TP +50-150%, SL -40%, trailing after +30%

Mode: SCALP
  Trigger: momentum >= 70 AND (alpha < 55 OR safety < 65)
  Sizing:  0.1-0.2 SOL (paper)
  Exit:    TP +10-30%, SL -15%, tight stops, minutes only

Mode: FALLBACK (Heuristics)
  Trigger: alpha < 55 OR any hard filter is borderline
  Sizing:  0.05-0.1 SOL (paper) — sniff test only
  Exit:    Any profit, cut fast
```

### 6.6 Rug Detection Module

Separate from scoring — `rug_detect.rs` produces a **Rug Report**:

```
RugReport {
    severity: LOW | MEDIUM | HIGH | CRITICAL,
    flags: Vec<RugFlag>,
    verdict: String,
}

RugFlag {
    name: "high_dev_allocation",
    severity: HIGH,
    detail: "Dev holds 8.5% of supply (threshold: 5%)",
    value: 0.085,
    threshold: 0.05,
}
```

CRITICAL severity blocks paper trades unless user explicitly overrides (scary confirmation modal).

---

## 7. Paper Trade Engine & Risk Management

### 7.1 Position State Machine

```
                  Paper Buy
                     │
                     ▼
                ┌──────────┐
                │  OPEN    │─── TP/SL hit (simulated via kline) ───┐
                │  Live PnL│─── User Paper Sell (partial/full) ────┤
                │  updates │─── Timeout (24h) ─────────────────────┤
                └──────────┘                                         │
                                                                     ▼
                                                          ┌──────────────┐
                                                          │   CLOSED     │
                                                          │  Log result  │
                                                          │  → Journal   │
                                                          │  → Tuner     │
                                                          └──────────────┘
```

### 7.2 Position Model

```rust
struct PaperPosition {
    id:              Uuid,
    token_address:   String,
    token_symbol:    String,
    side:            Side,
    entry_price:     f64,
    amount_sol:      f64,
    amount_tokens:   f64,
    slippage:        f64,
    mode:            TradeMode,
    tp_percent:      Option<f64>,
    sl_percent:      Option<f64>,
    trailing_tp:     Option<TrailingConfig>,
    trailing_sl:     Option<TrailingConfig>,
    status:          PositionStatus,
    opened_at:       DateTime<Utc>,
    closed_at:       Option<DateTime<Utc>>,
    exit_price:      Option<f64>,
    pnl_sol:         Option<f64>,
    pnl_percent:     Option<f64>,
    feature_vector:  FeatureVector,  // Full alpha filter snapshot at entry
    alpha_score:     f64,
    rug_report:      RugReport,
}
```

### 7.3 Price Simulation

**Paper Buy:**
1. User enters amount (e.g., 0.5 SOL)
2. Fetch current price from GMGN kline (latest candle close)
3. Apply slippage: `effective_entry = price + (price * slippage%)`
4. `tokens_received = amount_sol / effective_entry`
5. Calculate liquidity impact: `impact = amount_sol / liquidity_usd`; warn if > 5%
6. Record position as OPEN

**Paper Sell:**
1. User selects position + sell %
2. Fetch current price from GMGN kline
3. Apply slippage: `effective_exit = price - (price * slippage%)`
4. Calculate PnL: `pnl = (effective_exit - entry) / entry * 100`
5. Partial: reduce position, record realized PnL for slice
6. Full: close position

### 7.4 TP/SL Monitor (Background Task)

Runs as a `tokio` task, polls kline for open positions every 2 seconds (or uses Alph AI WebSocket kline feed):

```
for each OPEN position:
  fetch latest price
  if TP set and price >= entry * (1 + tp/100):
    if trailing: update peak, check drawdown
    else: simulate full sell at TP price
  if SL set and price <= entry * (1 - sl/100):
    if trailing: update trough, check rally
    else: simulate full sell at SL price
  update unrealized PnL
  send PnLUpdate event to UI thread
```

### 7.5 Risk Management

```rust
struct RiskManager {
    daily_loss_cap_sol:   f64,      // e.g., 5.00 SOL
    per_trade_max_sol:    f64,      // e.g., 2.50 SOL
    per_trade_max_pct:    f64,      // e.g., 5%
    daily_realized_pnl:   f64,
    trades_today:         u32,
    wins_today:           u32,
    losses_today:         u32,
    kill_switch_active:   bool,
    max_open_positions:   u8,       // e.g., 5
    max_same_token:       u8,       // e.g., 2
}
```

**Pre-trade checks (run BEFORE every paper buy):**
1. Kill switch active? → REJECTED
2. Daily loss exceeded? → REJECTED
3. Amount > per_trade_max? → REJECTED
4. Max positions reached? → REJECTED
5. Mode sizing bounds exceeded? → WARN
6. 2 daily wins reached? → WARN ("Greed kills")

**Daily reset:** Background task at midnight UTC logs day's summary, resets counters. Summary feeds LLM post-mortem.

**Emergency actions:**
- **Kill Switch:** Daily loss hits cap → modal, buy buttons grayed out.
- **Override:** User can override from Settings (scary confirmation, logged as discipline violation).
- **Emergency Exit All:** `Ctrl+E` sells all open positions at market (confirmation required).

---

## 8. Learning Engine

Two-pronged: Auto-Tuner (always-on, statistical) + LLM Post-Mortem (on-demand).

### 8.1 Auto-Tuner

**Not ML** — pure statistics: "what do winners have in common that losers don't?"

Every paper trade logs: feature_vector, alpha_score, mode, pnl_percent, outcome, duration.

**After every N trades (default 20):**

```
1. Separate into Winners (pnl > 0) and Losers (pnl < 0)
2. For each feature: winner_mean, loser_mean, winner_std
3. Discrimination power = |winner_mean - loser_mean| / (winner_std + epsilon)
4. For each category weight:
   - High discrimination → increase weight slightly
   - Low discrimination → decrease weight slightly
5. For each hard filter threshold:
   - If tightening would catch losers → tighten
   - If winners borderline → relax slightly
6. Compute "tuning delta", clamp to ±5% per run, apply, log
```

### 8.2 Tuner Guard Rails

| Guard | Rule |
|---|---|
| Max delta per run | ±5% total weight shift |
| Min sample size | 10+ wins AND 10+ losses before first tune |
| Weight bounds | Each weight stays within [0.05, 0.40] |
| Threshold bounds | Hard filters can tighten but never relax below safety floor |
| Revert capability | `Reset to Default` button in Settings |
| Tuning history | Every delta stored with timestamp + sample size |

### 8.3 LLM Post-Mortem

User clicks `[Run Post-Mortem]` in Strategy tab:

```
1. Collect trade journal for selected period
2. Compute summary statistics (win rate, avg win/loss, feature discrimination)
3. Build prompt: system (trading mindset) + GMGN workflow docs + journal data + current weights
4. Send to LLM provider (OpenAI / Anthropic / Ollama)
5. Parse response, display in Strategy tab
6. User applies/dismisses each suggestion (goes through ±5% guard rail)
```

### 8.4 LLM Provider Trait

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, prompt: LLMRequest) -> Result<LLMResponse>;
    fn name(&self) -> &str;
}
```

Implementations: `openai.rs`, `anthropic.rs`, `ollama.rs`.

---

## 9. Storage Layer (SQLite)

### 9.1 Tables

- `portfolios` — Paper balance state
- `positions` — Open and closed paper positions (with feature_vector JSON, alpha_score, rug_report)
- `daily_risk` — Per-day risk tracking (pnl, trade count, kill-switch state)
- `alpha_config` — Singleton row of current weights + hard filter thresholds
- `tuning_history` — Every auto-tune delta (old/new weights, discrimination analysis)
- `post_mortems` — LLM post-mortem history (prompt, response, applied/dismissed counts)
- `settings` — Key-value config store
- `watchlist` — Watched tokens
- `gmgn_cache` — GMGN API response cache (endpoint, params_hash, response, ttl)
- `alphai_cache` — Alph AI API response cache

### 9.2 Storage Module

```
src/storage/
├── mod.rs            # DbManager (singleton, connection pool via sqlx)
├── migrations.rs     # Schema creation on first run
├── positions.rs
├── journal.rs
├── config.rs
├── cache.rs
└── models.rs
```

DB location: `~/.config/quickscope/data.db`

---

## 10. Tech Stack

### Core Dependencies

| Category | Crate |
|---|---|
| TUI Framework | `ratatui` |
| Terminal Backend | `crossterm` |
| Async Runtime | `tokio` |
| HTTP Client | `reqwest` |
| WebSocket | `tokio-tungstenite` |
| Database | `sqlx` (SQLite) |
| Serialization | `serde` + `serde_json` |
| LLM | `async-openai` |
| CLI Args | `clap` |
| Logging | `tracing` + `tracing-subscriber` |
| Error Handling | `anyhow` + `thiserror` |
| UUID | `uuid` |
| Time | `chrono` |
| Crypto | `ed25519-dalek` (future-proof for real trading auth) |
| Configuration | `config` + `dotenvy` |

### Dev Dependencies

`mockall`, `tempfile`, `rstest`, `criterion`

---

## 11. Project Structure

```
quickscope/
├── Cargo.toml
├── .env.example
├── AGENTS.md
├── docs/
│   ├── superpowers/specs/2026-07-05-quickscope-design.md
│   ├── architecture.md
│   ├── data-sources.md
│   ├── alpha-filter.md
│   ├── learning-engine.md
│   └── api-reference/
│       ├── gmgn-endpoints.md
│       ├── alph-ai-endpoints.md
│       └── dex-screener.md
├── migrations/
│   └── 001_initial.sql
└── src/
    ├── main.rs
    ├── app/            (mod, tabs, input)
    ├── ui/             (mod, theme, 7 tab files, widgets/)
    ├── data/           (mod, models, gmgn/, alph_ai/, dex_screener/)
    ├── alpha/          (mod, scoring, rug_detect, narrative, thresholds, modes)
    ├── trade/          (mod, simulator, position, monitor, risk)
    ├── learning/       (mod, tuner, analyzer, journal, llm/)
    └── storage/        (mod, migrations, positions, journal, config, cache, models)
```

---

## 12. Configuration

Environment variables (stored in `~/.config/quickscope/.env`):

```
GMGN_API_KEY=gmgn_xxx
ALPH_DEX_COOKIE=xxx
OPENAI_API_KEY=sk-xxx
ANTHROPIC_API_KEY=sk-ant-xxx
OLLAMA_BASE_URL=http://localhost:11434
DEXSCREENER_BASE_URL=https://api.dexscreener.com
QUICKSCOPE_DB_PATH=~/.config/quickscope/data.db
QUICKSCOPE_LOG_LEVEL=info
```

---

## 13. Success Criteria

- TUI launches and renders all 7 tabs smoothly at 60fps
- GMGN trending tokens load in < 2 seconds
- Alph AI WebSocket delivers real-time price/signal updates without polling
- Paper trades execute with realistic slippage simulation
- TP/SL monitor correctly triggers simulated exits
- Auto-tuner runs after 20 trades and produces auditable weight changes
- LLM post-mortem produces actionable, specific suggestions
- Kill switch activates on daily loss cap
- Theme switching works without restart
- Mouse + keyboard both fully functional across all tabs
