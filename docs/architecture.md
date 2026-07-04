# Architecture

> System architecture for QuickScope. See the master spec at
> `docs/superpowers/specs/2026-07-05-quickscope-design.md` for full detail.

---

## High-Level Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                              │
│   tokio runtime + crossterm event loop + ratatui render      │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                       AppState (shared)                       │
│   Arc<tokio::sync::Mutex<AppState>>                          │
│   - tab states  - data caches  - positions  - settings       │
└──────┬──────────┬──────────┬──────────┬──────────┬──────────┘
       │          │          │          │          │
       ▼          ▼          ▼          ▼          ▼
   ┌───────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌─────────┐
   │  ui/  │ │ data/  │ │ alpha/ │ │ trade/ │ │learning/│
   │ render│ │ fetch  │ │ score  │ │ paper  │ │  tune   │
   └───────┘ └───┬────┘ └───┬────┘ └───┬────┘ └────┬────┘
                │          │          │            │
                ▼          ▼          ▼            ▼
            ┌─────────────────────────────────────────┐
            │              storage/ (SQLite)            │
            │   positions · journal · config · cache   │
            └─────────────────────────────────────────┘
```

---

## Event Flow

```
User input (key/mouse)
        │
        ▼
  crossterm event ──▶ AppEvent::Key/Mouse
        │
        ▼
  input router ──▶ AppCommand (SwitchTab, BuyPaper, AnalyzeToken, ...)
        │
        ▼
  update() mutates AppState, spawns async tasks
        │
        ├──▶ DataOrchestrator.fetch*() ──▶ GMGN/Alph AI/DEX Screener
        │                                      │
        │                                      ▼ (via tokio::mpsc)
        │──▶ DataEvent::TrendingUpdated / KlineUpdated / ...
        │
        ├──▶ TradeEngine.paper_buy() ──▶ Position opened ──▶ storage
        │
        ├──▶ TP/SL monitor (bg task) ──▶ TradeEvent::PositionClosed
        │
        └──▶ view() re-renders all tabs from current AppState
```

---

## Module Responsibilities

### `app/` — Application Core
- `mod.rs`: `AppState` struct, `update()` and `view()` entry points.
- `tabs.rs`: Tab manager — which tab is focused, tab state per view.
- `input.rs`: Global keybinding router, mouse event dispatcher.

### `ui/` — Rendering (ratatui)
- `mod.rs`: Root layout (status bar, tab bar, content area, keybinding hints).
- `theme.rs`: `Theme` struct with semantic color tokens. Dark + Degen presets.
- `dashboard.rs` ... `settings.rs`: One file per tab. Each exports `render(frame, area, state)`.
- `widgets/`: Reusable components — `Sparkline`, `ProgressBar`, `Tag`, `Modal`, `ContextMenu`, `Table`, `SearchBar`, `Notification`, `Chart`.

**Rule:** UI code never does I/O. It only reads `AppState` and renders. Async work happens in `app/` or `data/`.

### `data/` — External Data
- `mod.rs`: `DataOrchestrator` (high-level facade), `DataCache` (LRU + TTL), `DataEvent` enum.
- `models.rs`: Shared domain types (`Token`, `Wallet`, `Trade`, `Kline`, `Signal`, etc.) independent of any specific API's response shape.
- `gmgn/`: GMGN REST client. One file per skill group (market, token, portfolio, track, quote). `types.rs` holds serde structs matching GMGN JSON.
- `alph_ai/`: Alph AI REST + WebSocket client. `twitter.rs` for X monitoring, `websocket.rs` for real-time feeds.
- `dex_screener/`: DEX Screener REST client.

**Rule:** API clients return `data::models` types, not raw API structs. Translation happens inside the client module. This isolates the rest of the app from API changes.

### `alpha/` — Alpha Filter Engine
- `mod.rs`: `AlphaFilter` orchestrator — takes a `Token` + fetched data, returns `AlphaReport { score, sub_scores, mode, rug_report }`.
- `scoring.rs`: Feature vector extraction + 6 category scores + composite Alpha Score.
- `rug_detect.rs`: Produces `RugReport` with severity flags.
- `narrative.rs`: Detects narrative/meta from token name, description, social links, Twitter mentions.
- `thresholds.rs`: Loads/saves mutable weights + hard filter thresholds from `alpha_config` table.
- `modes.rs`: Maps score profile → TradeMode (EXPLODE/ALPHA/SCALP/FALLBACK) + sizing bounds.

### `trade/` — Paper Trade Engine
- `mod.rs`: `TradeEngine` — orchestrates buy/sell flow.
- `simulator.rs`: Price simulation (slippage, liquidity impact).
- `position.rs`: `PaperPosition` struct + state machine (open/closed).
- `monitor.rs`: Background TP/SL monitor task.
- `risk.rs`: `RiskManager` — pre-trade checks, daily loss cap, kill-switch.

### `learning/` — Learning Engine
- `mod.rs`: `LearningEngine` — coordinates tuner + LLM.
- `tuner.rs`: Statistical auto-tuner. Runs after every N trades.
- `analyzer.rs`: Feature discrimination analysis (winners vs losers).
- `journal.rs`: Trade journal queries + formatting for tuner and LLM.
- `llm/mod.rs`: `LLMProvider` trait + factory.
- `llm/openai.rs`, `anthropic.rs`, `ollama.rs`: Provider implementations.
- `llm/prompts.rs`: Post-mortem prompt templates.

### `storage/` — SQLite Persistence
- `mod.rs`: `DbManager` singleton, connection pool.
- `migrations.rs`: Schema creation on first run.
- `positions.rs`, `journal.rs`, `config.rs`, `cache.rs`: CRUD per domain.
- `models.rs`: Row structs mirroring SQLite tables.

DB location: `~/.config/quickscope/data.db`

---

## Threading Model

| Thread / Task | Role |
|---|---|
| Main thread | crossterm event poll + ratatui render (~60fps) |
| tokio worker pool | HTTP requests, SQLite writes, LLM calls |
| Dedicated task: TP/SL monitor | Polls kline for open positions every 2s (or consumes Alph AI WS kline) |
| Dedicated task: Auto-tuner | Runs after every N trades |
| Dedicated task: Alph AI WebSocket | Long-lived connection, pushes events to mpsc channel |
| Dedicated task: Daily reset | Fires at midnight UTC |

All cross-task communication via `tokio::mpsc` channels routed into the main event loop.

---

## Key Design Principles

1. **UI is pure.** Rendering reads state, never mutates it or does I/O.
2. **API clients translate.** External API shapes never leak past `data/`.
3. **State is explicit.** `AppState` is the single source of truth; tabs hold only view-local state (scroll position, selected index).
4. **Async at the edges.** I/O is async; scoring, risk checks, and rendering are sync.
5. **Paper trading is unbreakable.** No code path in v1 touches real-money endpoints. The `trade/` module only simulates.
6. **Learning is observable.** Every auto-tune delta and LLM suggestion is logged with full context for audit.
