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
│   - tab states  - data caches  - positions  - settings      │
│   - list_mode (Trending/Trenches/Watchlist/AI-Rec)          │
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
  input router ──▶ AppCommand (SwitchTab, BuyPaper, AnalyzeToken, FetchTrenches, ...)
        │
        ▼
  update() mutates AppState, spawns async tasks
        │
        ├──▶ DataOrchestrator.fetch*() ──▶ GMGN/Alph AI/DEX Screener
        │                                      │
        │                                      ▼ (via tokio::mpsc)
        │──▶ DataEvent::TrendingUpdated / TrenchesUpdated / ...
        │
        ├──▶ TradeEngine.paper_buy() ──▶ Position opened ──▶ storage
        │
        ├──▶ TP/SL monitor (bg task) ──▶ TradeEvent::PositionClosed
        │
        └──▶ view() re-renders all tabs from current AppState

Address Search Flow:
        │
        ▼
  Ctrl+P → " by Address" → input_active=true, address_search_mode=true
        │
        ▼
  User types address, presses Enter
        │
        ▼
  AppCommand::FetchTokenDetail(addr)
        │
        ▼
  DataEvent::TokenLoaded → switch to Analyzer tab
```

---

## Module Responsibilities

### `app/` — Application Core
- `mod.rs`: `AppState` struct, `update()` and `view()` entry points. Handles all `DataEvent` variants including `TrenchesUpdated`, `AutoTuneHistoryLoaded`, `PostMortemHistoryLoaded`.
- `state.rs`: `AppState` struct, `TokenListMode` enum (Trending / Trenches / Watchlist / AiRec), `filtered_trending()`, `current_list()`, `list_mode_label()`, `check_api_keys()`, `show_fatal_error()`.
- `input.rs`: Global keybinding router, mouse event dispatcher. Context-sensitive `←`/`→` (tab switch vs mode switch), address search mode, mouse guard (only processes clicks on list tabs, ignores modals/palettes).

### `ui/` — Rendering (ratatui)
- `mod.rs`: Root module, re-exports `render_ui`, `Theme`, and format utilities.
- `layout.rs`: Root layout — top status bar, sidebar | content split, bottom keybinding bar, overlays (modal, toast, command palette). Dynamic toast area sizing, modal backdrop clearing.
- `sidebar.rs`: Persistent VS Code-style activity bar with 7 tab icons. Collapsible via `Ctrl+B`. Shows kill switch indicator.
- `format.rs`: `format_marketcap()`, `format_volume()`, `marketcap_color()`, `volume_color()` — abbreviated display + tier-based colors.
- `theme.rs`: `Theme` struct with 18 semantic color tokens. 4 presets: Dark, Terminal (Bloomberg), Degen (neon), Cyberpunk (pink/cyan).
- `dashboard.rs` ... `settings.rs`: One file per tab. Each exports `render(frame, area, state)`.
- `scanner.rs`: Mode selector bar at top (Trending │ Trenches │ Watchlist │ AI-Rec). Switches data source based on `state.list_mode`. Renders trenches separately with age, platform, smart money columns.
- `strategy.rs`: Shows auto-tune history with feature discrimination bars, LLM post-mortem results with acceptance rates.
- `settings.rs`: Live API key status from env vars (green/red indicators).
- `widgets/command_palette.rs`: Ctrl+P overlay with search/filter, 15+ commands, arrow-key navigation, dimmed backdrop.
- `widgets/modal.rs`: Sweet-alert style with centered panel, dimmed backdrop, colored border, action hints (Enter/ESC).
- `widgets/`: Reusable components — `Toast`, `Modal`, `Sparkline`, `ProgressBar`, `Tag`, `Table`, `SearchBar`, `Chart`, `ContextMenu`.

**Key UI patterns:**
- All overlays render a dark backdrop (solid black fill) to obscure content behind and prevent ghost clicks.
- Token lists use `current_list()` (respects `list_mode`) and `scroll_offset` + `list_cursor` for bounded scrolling.
- Marketcap is always abbreviated and color-coded (blue ≥$10M, green ≥$1M, yellow ≥$100K, red <$100K).
- Volume and marketcap are primary display columns; price is secondary/muted.
- Toast notifications auto-dismiss after 4 seconds with smooth decay.
- Mouse events only fire on Dashboard, Scanner, and Analyzer tabs. Other tabs and overlays ignore content clicks.

**Rule:** UI code never does I/O. It only reads `AppState` and renders. Async work happens in `app/` or `data/`.

### `data/` — External Data
- `mod.rs`: `DataOrchestrator` (high-level facade), `DataCache` (LRU + TTL), `DataEvent` enum.
- `models.rs`: Shared domain types (`Token`, `Wallet`, `Trade`, `Kline`, `Signal`, `TrenchToken`, etc.) independent of any specific API's response shape. Event/command enums (`AppEvent`, `AppCommand`, `DataEvent`). `TabIndex` with `next()`/`prev()`.
- `gmgn/`: GMGN client via `gmgn-cli` subprocess. `types.rs` holds serde structs matching GMGN JSON. Parser tries `market_cap` then `marketCap`.
- `alph_ai/`: Alph AI REST + WebSocket client. `websocket.rs` with exponential backoff reconnection (1s → 2s → 4s → ... → 60s max), ping/pong handling, channel-based subscription.
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
- `llm/mod.rs`: `LLMProvider` trait + factory (OpenAI, Anthropic, Ollama, Stub).
- `llm/prompts.rs`: Post-mortem prompt templates.

### `storage/` — SQLite Persistence
- `mod.rs`: `DbManager` singleton, connection pool.
- `migrations.rs`: Schema creation on first run.
- `positions.rs`, `journal.rs`, `config.rs`, `cache.rs`: CRUD per domain.
- `journal.rs` includes: daily risk, watchlist, portfolio, tuning history, post-mortems.
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
| Dedicated task: Alph AI WebSocket | Long-lived connection with exponential backoff reconnect, pushes events to mpsc channel |
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
7. **Mouse is guarded.** Content clicks only fire on tabs that display token lists (Dashboard, Scanner, Analyzer). Modals and palettes block all mouse events.
8. **Data sources are swappable.** The Scanner mode selector (`TokenListMode`) abstracts data source selection from rendering.
