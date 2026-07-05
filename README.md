<div align="center">

# ⚡ QuickScope

**Solana Memecoin Alpha Hunting TUI — Paper Trade, Analyze, Learn**

[![Rust](https://img.shields.io/badge/Rust-2021-edition?logo=rust&style=flat-square)]()
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)]()
[![Tests](https://img.shields.io/badge/tests-104%20passing-success?style=flat-square)]()
[![Status](https://img.shields.io/badge/status-alpha-yellow?style=flat-square)]()

<img src="https://img.shields.io/badge/TUI-ratatui%20%7C%20crossterm-ff6b6b?style=flat-square" />
<img src="https://img.shields.io/badge/Solana-memecoin-9945FF?logo=solana&style=flat-square" />
<img src="https://img.shields.io/badge/GMGN%20%7C%20Alph%20AI%20%7C%20DEX%20Screener-powered-38d9a9?style=flat-square" />

*A terminal-based application for discovering, analyzing, and paper trading memecoins on Solana. Real-time data, intelligent filtering, risk discipline, and a learning engine — all running in your terminal.*

</div>

---

## 📑 Table of Contents

- [✨ Features](#-features)
- [🚀 Quick Start](#-quick-start)
- [🎮 Usage](#-usage)
- [🏗️ Architecture](#️-architecture)
- [📡 Data Sources](#-data-sources)
- [🧠 Alpha Filter Engine](#-alpha-filter-engine)
- [📊 Paper Trading](#-paper-trading)
- [🎯 Learning Engine](#-learning-engine)
- [🎨 Themes](#-themes)
- [⚙️ Configuration](#️-configuration)
- [📁 Project Structure](#-project-structure)
- [🧪 Testing](#-testing)
- [🛣️ Roadmap](#️-roadmap)
- [📚 Documentation](#-documentation)
- [🤝 Contributing](#-contributing)
- [📄 License](#-license)

---

## ✨ Features

<table>
<tr>
<td width="50%">

### 🔭 Alpha Discovery
- Real-time trending from **GMGN** + **Alph AI** + **DEX Screener**
- 30+ dimension **feature vector** per token
- **6 category sub-scores**: Momentum, Safety, Holder Quality, Liquidity, Dev Trust, Social
- Composite **Alpha Score** (0–100) with weighted formula
- **Narrative detection** (AI Agent, Dog, Cat, Frog, Political, Gaming, DeFi…)
- **Rug detection** with severity levels (Low → Critical)

</td>
<td width="50%">

### 📊 Paper Trading
- Simulated buy/sell with realistic **slippage**
- **4 trade modes**: EXPLODE, ALPHA, SCALP, FALLBACK
- Auto-sizing based on mode confidence
- **TP/SL monitoring** background task
- **Risk management**: daily loss cap, kill switch, max positions, win cap
- **Emergency exit all** (`Ctrl+E`)

</td>
</tr>
<tr>
<td width="50%">

### 🧠 Learning System
- **Auto-tuner**: statistical discrimination analysis on every closed trade
- Nudges weights ±5% per run (guard-railed: 0.05–0.40 per weight)
- Min 10 wins + 10 losses before first tune
- **LLM post-mortem** (pluggable: OpenAI, Anthropic, Ollama)
- Full tuning history logged to SQLite

</td>
<td width="50%">

### 🎨 Terminal UI
- **7 tabs**: Dashboard, Scanner, Analyzer, Trade Terminal, Journal, Strategy & Learning, Settings
- **2 themes**: Dark (OpenCode-inspired) & Degen Mode (neon)
- Vim-like navigation (`j`/`k`, number keys for tabs)
- Mouse support, modal dialogs, notification toasts
- Full keyboard shortcut system (`?` for help)
- Portfolio overview, real-time PnL, status bar

</td>
</tr>
</table>

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.75+ (edition 2021)
- A **GMGN API key** (free tier available)
- A **dex_cookie** from [alph.ai](https://alph.ai) (optional, for enhanced data)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/quickscope
cd quickscope

# Configure your API keys
cp .env.example .env
# Edit .env with your keys
$EDITOR .env

# Run
cargo run --release
```

### Configuration

```env
# Required
GMGN_API_KEY=gmgn_solbscbaseethmonadtron

# Optional — enhances data with Twitter/X monitoring + WebSocket
ALPH_DEX_COOKIE=your_dex_cookie_from_alph_ai

# Optional — LLM post-mortem (pick one)
OPENAI_API_KEY=sk-...
# ANTHROPIC_API_KEY=sk-ant-...
# OLLAMA_BASE_URL=http://localhost:11434

# Optional — override defaults
QUICKSCOPE_DB_PATH=~/.config/quickscope/data.db
QUICKSCOPE_LOG_LEVEL=info
```

---

## 🎮 Usage

### Keyboard Shortcuts

| Key | Action | Key | Action |
|---|---|---|---|
| `1`–`7` | Switch tab | `j` / `↓` | Move cursor down |
| `Tab` | Next tab | `k` / `↑` | Move cursor up |
| `Shift+Tab` | Previous tab | `Enter` | Select / View detail |
| `?` | Toggle help modal | `Esc` | Close modal / Back |
| `r` | Refresh data | `q` | Quit |
| `b` | Paper buy | `s` | Paper sell |
| `w` | Toggle watchlist | `Ctrl+E` | Emergency exit all |

### Tab Overview

| Tab | Key | Purpose |
|---|---|---|
| **Dashboard** | `1` | Portfolio snapshot + live trending list |
| **Scanner** | `2` | Browse trending tokens, filter, select for analysis |
| **Analyzer** | `3` | Deep-dive: kline, security, holders, smart money, Alpha Score |
| **Trade** | `4` | Open/close paper positions, TP/SL, quick actions |
| **Journal** | `5` | Trade history, session stats, win rate |
| **Strategy** | `6` | Auto-tune weights, LLM post-mortem |
| **Settings** | `7` | API keys, risk profile, display preferences |

---

## 🏗️ Architecture

```
                    ┌──────────────────────────────────────────────────────────────┐
                    │                    QuickScope TUI (ratatui)                   │
                    │  ┌─────────┐ ┌─────────┐ ┌────────┐ ┌──────┐ ┌──────────┐   │
                    │  │Dashboard│ │Scanner  │ │Analyzer│ │Trade │ │Strategy …│   │
                    │  └────┬────┘ └────┬────┘ └───┬────┘ └──┬───┘ └────┬─────┘   │
                    │       └───────────┴──────────┴─────────┴──────────┘          │
                    └───────────────────────┬──────────────────────────────────────┘
                                            │
                            ┌───────────────┴───────────────┐
                            │         AppState + update()    │
                            │       (Elm/TEA event loop)     │
                            └───────────────┬───────────────┘
                                            │
              ┌─────────────────────────────┼─────────────────────────────┐
              │                             │                             │
              ▼                             ▼                             ▼
    ┌─────────────────┐          ┌───────────────────┐         ┌──────────────────┐
    │  DataOrchestrator│          │   Alpha Filter     │         │  Trade Engine    │
    │  (GMGN + Alph AI│◄────────►│   (Feature Vector  │◄───────►│  (Simulator +    │
    │   + DEX Screen) │          │   → Scoring → Mode)│         │   Risk Manager)  │
    └─────────────────┘          └───────────────────┘         └────────┬─────────┘
              │                                                         │
              ▼                                                         ▼
    ┌─────────────────────┐                                 ┌──────────────────────┐
    │  Learning Engine     │                                 │  SQLite Storage      │
    │  (Auto-Tuner + LLM)  │◄───────────────────────────────│  (positions, journal, │
    └─────────────────────┘                                 │   config, cache)      │
                                                             └──────────────────────┘
```

### Threading Model

```
┌─────────────────────────────────────────────────────────┐
│                     tokio Runtime                         │
├───────────────────┬───────────────────┬──────────────────┤
│   Main Thread     │  Worker Pool      │  Background       │
│  (crossterm +     │  (HTTP + WS +     │  (TP/SL Monitor,  │
│   ratatui render) │   SQLite I/O)     │   Auto-Tuner,     │
│                   │                   │   Daily Reset)    │
└───────────────────┴───────────────────┴──────────────────┘
```

---

## 📡 Data Sources

QuickScope combines three data sources for maximum signal:

| Source | Role | Auth | Rate Limits | Key Strength |
|---|---|---|---|---|
| **[GMGN](https://gmgn.ai)** | **Primary** | `X-APIKEY` header | 20 req/s, weights 1–5 | Best Solana memecoin data: trending, kline, security, holders, smart money |
| **[Alph AI](https://alph.ai)** | **Secondary** | `Cookie: dex_cookie` | Unlisted (be reasonable) | **Twitter/X monitoring**, WebSocket real-time, AI signal confidence (Gold/Silver/Copper) |
| **[DEX Screener](https://dexscreener.com)** | **Tertiary** | None | Unlisted | Cross-reference trending/boosts |

### Why Three Sources?

GMGN provides the best fundamental data but has **zero Twitter/X visibility** — a critical gap for narrative-driven memecoin trading. Alph AI fills this completely with KOL tweet tracking, CA extraction from tweets, sentiment analysis, and real-time WebSocket feeds. DEX Screener adds boost/trending cross-reference for conviction multiplier.

### Cache Strategy

| Data Type | TTL | Source |
|---|---|---|
| Trending list | 30s | GMGN |
| Kline (price candles) | 10s | GMGN |
| Token detail | 60s | GMGN + Alph AI |
| Smart money trades | 15s | GMGN |
| Signals | 30s | GMGN + Alph AI |
| Twitter feed | 60s | Alph AI |

---

## 🧠 Alpha Filter Engine

The brain of QuickScope. Every token scored across **30+ dimensions** in **6 categories**:

### Category Scores

```
Momentum Score = vol_score × 0.30 + swap_score × 0.20 + hot_score × 0.20 + change_score × 0.30

Safety Score   = (1 - rug_ratio) × 0.35 + wash_trading × 0.15 + renounced_mint × 0.15
               + renounced_freeze × 0.15 + (1 - top10_rate) × 0.20

Holder Quality = smart_degen × 0.35 + renowned × 0.20 + (1 - dev_hold) × 0.20
               + (1 - insider) × 0.15 + (1 - fresh_rate) × 0.10

Liquidity Score = sigmoid(liquidity) × 0.50 + on_curve × 0.30 + sigmoid(market_cap) × 0.20

Dev Trust Score = creator_status × 0.30 + normalize(ath_mc) × 0.25 + cto_flag × 0.20
               + dexscr_boost × 0.15 + normalize(prev_tokens) × 0.10

Social Score   = twitter_mentions × 0.30 + sentiment × 0.20 + followers × 0.20
               + signal_confidence × 0.30
```

### Composite Alpha Score

```
Alpha = w_momentum × M + w_safety × S + w_holder × H
      + w_liquidity × L + w_dev × D + w_social × S
      × 100  →  [0, 100]
```

### Trade Mode Selection

| Mode | Trigger | Sizing | TP/SL |
|---|---|---|---|
| 🚀 **EXPLODE** | `alpha ≥ 75 AND momentum ≥ 80 AND safety ≥ 70` | 0.5–1.0 SOL | TP +200%, SL -60% |
| ⚡ **ALPHA** | `alpha ≥ 55 AND safety ≥ 65` | 0.2–0.5 SOL | TP +100%, SL -40% |
| 🎯 **SCALP** | `momentum ≥ 70 AND (alpha < 55 OR safety < 65)` | 0.1–0.2 SOL | TP +20%, SL -15% |
| 🐢 **FALLBACK** | Everything else | 0.05–0.1 SOL | TP +10%, SL -20% |

### Hard Filters (Instant Reject)

| Filter | Threshold |
|---|---|
| `rug_ratio` | > 0.30 |
| `dev_team_hold_rate` | > 0.15 |
| `is_wash_trading` | `true` |
| `renounced_mint` | `false` |
| `liquidity_usd` | < $5,000 |

---

## 📊 Paper Trading

All trading is **simulated** — no real money, no real execution.

### Order Flow

```
Paper Buy:
  1. User enters amount (e.g., 0.5 SOL)
  2. Fetch live price from GMGN kline
  3. Apply slippage → effective entry price
  4. Calculate tokens received
  5. Liquidity impact check (>5% → warning)
  6. Risk checks (5 checks + optional warnings)
  7. Record position as OPEN

Paper Sell:
  1. User selects position + sell percentage
  2. Fetch live price
  3. Apply slippage → effective exit price
  4. Calculate PnL
  5. Partial or full close
```

### Risk Management

- **Daily loss cap**: 5 SOL (default, configurable)
- **Per-trade max**: 2.5 SOL
- **Max open positions**: 5
- **Max same token**: 2
- **Kill switch**: auto-activated when daily cap hit
- **2-win warning**: "Greed kills" after 2 daily wins
- **Override**: scary confirmation modal required

---

## 🎯 Learning Engine

### Auto-Tuner (Always-On)

After every N closed trades (default 20), runs statistical discrimination analysis:

```
1. Split trades into Winners (PnL > 0) and Losers (PnL < 0)
2. For each of 14 features: compute winner_mean, loser_mean, discrimination_power
3. Map feature discrimination → category weight nudges
4. Apply nudges clamped to ±5% per run
5. Each weight stays within [0.05, 0.40]
6. Log every delta to tuning_history table
```

### LLM Post-Mortem (On-Demand)

User clicks "Run Post-Mortem" in Strategy tab → sends session data to LLM:

```python
Visit: [Strategy Tab] → [Run Post-Mortem]
Result: AI analyst reviews your trades, finds patterns,
        suggests weight adjustments, flags behavioral issues
```

Supported providers: OpenAI (GPT-4o), Anthropic (Claude 3), Ollama (local models).

---

## 🎨 Themes

### Dark (Default)
```
Background:  #0d1117 (GitHub Dark)
Accent:      #58a6ff (Blue)
Success:     #3fb950 (Green)
Danger:      #f85149 (Red)
Muted:       #8b9498 (Gray)
```

### Degen Mode
```
Background:  #0a0514 (Deep Purple)
Accent:      #00ff88 (Neon Green)
Success:     #00ff64 (Green)
Danger:      #ff3264 (Hot Pink)
Muted:       #7864a0 (Lavender)
```

Switch with: `Settings Tab → Theme Degen`

---

## ⚙️ Configuration

QuickScope reads configuration from:
1. CLI arguments (`--config path/to/config.toml`)
2. Environment variables
3. `.env` file in project root

### Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `GMGN_API_KEY` | ✅ | — | GMGN API key |
| `ALPH_DEX_COOKIE` | ❌ | — | Alph AI cookie for Twitter/WS |
| `OPENAI_API_KEY` | ❌ | — | OpenAI key for post-mortem |
| `ANTHROPIC_API_KEY` | ❌ | — | Anthropic key for post-mortem |
| `OLLAMA_BASE_URL` | ❌ | `http://localhost:11434` | Ollama endpoint |
| `QUICKSCOPE_DB_PATH` | ❌ | `~/.config/quickscope/data.db` | SQLite database path |
| `QUICKSCOPE_LOG_LEVEL` | ❌ | `info` | Tracing log level |

---

## 📁 Project Structure

```
quickscope/
├── Cargo.toml              # Rust project with all dependencies
├── .env.example            # Configuration template
├── migrations/             # SQL migration reference
│   └── 001_initial.sql
├── docs/
│   ├── AGENTS.md           # Agentic worker instructions
│   ├── architecture.md     # System architecture documentation
│   ├── alpha-filter.md     # Scoring formulas in detail
│   ├── data-sources.md     # API comparison and integration logic
│   ├── learning-engine.md  # Auto-tuner algorithm in depth
│   ├── api-reference/      # Endpoint reference per data source
│   │   ├── gmgn-endpoints.md
│   │   ├── alph-ai-endpoints.md
│   │   └── dex-screener.md
│   ├── plans/              # Implementation plan
│   └── superpowers/specs/  # Design specification
└── src/
    ├── main.rs             # Entry point (TUI event loop)
    ├── lib.rs              # Module declarations
    ├── app/                # Elm/TEA architecture
    │   ├── mod.rs          # update() function
    │   ├── state.rs        # AppState
    │   └── input.rs        # Key/mouse dispatch
    ├── ui/                 # Terminal UI
    │   ├── mod.rs
    │   ├── theme.rs        # Dark + Degen themes
    │   ├── layout.rs       # Root layout + tab dispatch
    │   └── widgets/        # 7 tab implementations
    │       ├── dashboard.rs
    │       ├── scanner.rs
    │       ├── analyzer.rs
    │       ├── trade_terminal.rs
    │       ├── journal.rs
    │       ├── strategy.rs
    │       └── settings.rs
    ├── data/               # Data sources
    │   ├── models.rs       # All domain types
    │   ├── orchestrator.rs # 3-source merge facade
    │   ├── gmgn/           # GMGN client + rate limiter + parsers
    │   ├── alph_ai/        # Alph AI REST + WebSocket client
    │   └── dex_screener/   # DEX Screener client
    ├── alpha/              # Alpha Filter Engine
    │   ├── mod.rs          # Pipeline orchestrator
    │   ├── feature_vector  # 30+ dimension extraction
    │   ├── scoring.rs      # 6 category formulas
    │   ├── hard_filters.rs # Instant reject checks
    │   ├── rug_detect.rs   # Rug pull analysis
    │   ├── modes.rs        # Trade mode selection
    │   └── narrative.rs    # Narrative detection (Dog, AI, etc.)
    ├── trade/              # Paper Trade Engine
    │   ├── mod.rs          # Trade orchestrator
    │   ├── simulator.rs    # Buy/sell simulation
    │   ├── risk.rs         # 6 pre-trade checks + kill switch
    │   └── monitor.rs      # TP/SL background task
    ├── learning/           # Learning Engine
    │   ├── mod.rs
    │   ├── analyzer.rs     # Statistical discrimination
    │   ├── tuner.rs        # Weight auto-tuning
    │   ├── journal.rs      # Post-mortem flow
    │   └── llm/            # AI providers
    │       ├── mod.rs      # Enum-based provider (OpenAI/Anthropic/Ollama)
    │       └── prompts.rs  # Prompt templates
    └── storage/            # SQLite persistence
        ├── mod.rs
        ├── db.rs           # Connection pool + init
        ├── migrations.rs   # Schema SQL
        ├── positions.rs    # Position CRUD
        ├── journal.rs      # Daily risk, watchlist, portfolio
        ├── config.rs       # Alpha config + settings
        └── cache.rs        # API response cache
```

---

## 🧪 Testing

```bash
# Run all library tests
cargo test --lib

# Run a specific module
cargo test --lib storage::positions

# Run with output
cargo test --lib -- --nocapture

# Full test suite (104+ tests)
```

### Test Coverage

| Module | Tests | Status |
|---|---|---|
| Domain models (`data::models`) | 14 | ✅ |
| GMGN client + parsers | 13 | ✅ |
| Alph AI client + types | 8 | ✅ |
| DEX Screener | 1 | ✅ |
| DataOrchestrator | 1 | ✅ |
| Alpha Filter (scoring, filters, rug, modes, narrative) | 33 | ✅ |
| Trade Engine (simulator, risk) | 15 | ✅ |
| Learning Engine (analyzer, tuner, LLM) | 8 | ✅ |
| Storage (DB, positions, journal, config, cache) | 15 | ✅ |
| App (state, tab switching) | 3 | ✅ |
| **Total** | **104** | ✅ |

---

## 🛣️ Roadmap

### v0.2 — Near Term
- [ ] Full WebSocket integration (live kline → TP/SL auto-trigger)
- [ ] Real GMGN/Alph AI API integration tests
- [ ] Tab polish: Scanner (filters), Analyzer (kline chart), Trade Terminal (order books)
- [ ] Notification system (toast → sound on signal)
- [ ] Settings persistence + theme switching in Settings tab

### v0.3 — Medium Term
- [ ] Watchlist with price alerts
- [ ] Multi-period backtesting (replay historical data through alpha filter)
- [ ] Advanced charts (sparkline, volume bars, order flow)
- [ ] Session export (CSV/JSON)
- [ ] Configurable risk profiles

### v1.0 — Long Term
- [ ] User-defined custom filter rules (DSL)
- [ ] Portfolio tracking (connect/sync real wallet, read-only)
- [ ] Plugin system for data sources
- [ ] Community signal sharing

---

## 📚 Documentation

All design documentation lives in the [`docs/`](./docs/) directory:

| Document | Description |
|---|---|
| [`docs/AGENTS.md`](./docs/AGENTS.md) | Instructions for AI agents working on this codebase |
| [`docs/architecture.md`](./docs/architecture.md) | System architecture, event flow, module responsibilities |
| [`docs/data-sources.md`](./docs/data-sources.md) | GMGN vs Alph AI vs DEX Screener comparison, auth, cache strategy |
| [`docs/alpha-filter.md`](./docs/alpha-filter.md) | Full scoring formulas, feature vector spec, rug detection |
| [`docs/learning-engine.md`](./docs/learning-engine.md) | Auto-tuner algorithm, guard rails, LLM prompt templates |
| [`docs/api-reference/gmgn-endpoints.md`](./docs/api-reference/gmgn-endpoints.md) | All GMGN endpoints with weights and critical notes |
| [`docs/api-reference/alph-ai-endpoints.md`](./docs/api-reference/alph-ai-endpoints.md) | All Alph AI endpoints, WebSocket subscription format |
| [`docs/superpowers/specs/2026-07-05-quickscope-design.md`](./docs/superpowers/specs/2026-07-05-quickscope-design.md) | Complete design specification (single source of truth) |
| [`docs/superpowers/plans/2026-07-05-quickscope-implementation-plan.md`](./docs/superpowers/plans/2026-07-05-quickscope-implementation-plan.md) | 11-phase implementation plan |

---

## 🤝 Contributing

1. Read [`docs/AGENTS.md`](./docs/AGENTS.md) first (critical rules for this project)
2. Fork / branch from `main`
3. Use conventional commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`)
4. Tests must pass before merge
5. Run `cargo clippy` and `cargo fmt` before committing

### Code Style
- 4-space indentation
- Prefer `anyhow::Result` over panics
- `tracing` for logging (no `println!` in library code)
- UI is pure: rendering reads state, never mutates or does I/O
- Paper trading only — no real-money execution paths

---

## 📄 License

MIT License — see [`LICENSE`](./LICENSE) (if applicable).

---

<div align="center">

**Built with** 🦀 **Rust** · **ratatui** · **tokio**

*Not financial advice. Trade responsibly.*

</div>
