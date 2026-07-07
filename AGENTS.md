# AGENTS.md — QuickScope

> Instructions for AI agents (and humans) working on the QuickScope codebase.
> Read this before making any changes.

---

## What is QuickScope?

QuickScope is a **Rust-based terminal user interface (TUI)** for memecoin alpha hunting and **paper trading** on Solana. It is built around the `memecoin-alpha-agent` skill — alpha-selection filtering, momentum-explosion detection, rug detection, risk discipline, and continuous learning.

- **Language:** Rust (edition 2021)
- **Architecture:** Monolithic single-crate with feature modules (see `docs/architecture.md`)
- **Trading mode:** Paper trading only (v1). No real-money transactions.
- **Chain:** Solana only.
- **Inspiration:** OpenCode TUI — multi-panel, tabbed, keyboard + mouse, themeable.

---

## Project Status

| Phase | Status |
|---|---|
| Brainstorming & design | ✅ Complete — `docs/superpowers/specs/2026-07-05-quickscope-design.md` |
| Implementation plan | ✅ Complete — `docs/superpowers/plans/2026-07-05-quickscope-implementation-plan.md` |
| Storage (SQLite, positions, config, cache) | ✅ Complete |
| Data — GMGN client (via gmgn-cli subprocess) | ✅ Complete |
| Data — Alph AI client (REST + WebSocket with auto-reconnect) | ✅ Complete |
| Data — DEX Screener client + DataOrchestrator merge | ✅ Complete |
| Alpha Filter engine (feature vector, 6-category scoring, rug, modes, narrative) | ✅ Complete |
| Paper Trade Engine (simulator, risk manager, TP/SL monitor) | ✅ Complete |
| Learning Engine (auto-tuner, discrimination analyzer, LLM providers) | ✅ Complete |
| TUI Core (AppState, responsive event loop, theme system, 15 shared widgets) | ✅ Complete |
| TUI Tabs (all 7 — Dashboard, Scanner, Analyzer, Trade, Journal, Strategy, Settings) | ✅ Complete |
| Modal redesign (sweet-alert style with dimmed backdrop) | ✅ Complete |
| Command palette (searchable, keyboard-navigable) | ✅ Complete |
| Scroll/layer bug fixes (clamped offsets, overlay clearing) | ✅ Complete |
| Scanner mode selector (Trending / Trenches / Watchlist / AI-Rec) | ✅ Complete |
|  by Address (contract lookup via command palette) | ✅ Complete |
| Theme system (Dark / Terminal / Degen / Cyberpunk) | ✅ Complete |
| Error handling (fatal error modals for missing API keys) | ✅ Complete |
| Alph AI WebSocket reconnect with exponential backoff | ✅ Complete |
| Integration, polish, docs | ✅ Complete — 105 tests, all docs updated |

---

## Critical Rules for Agents

### 1. Read the design spec first
Before any implementation work, read `docs/superpowers/specs/2026-07-05-quickscope-design.md`. Every module, type, and decision is documented there. Do not improvise architecture that contradicts the spec.

### 2. Paper trading only — never execute real trades
The `gmgn swap`, `gmgn cooking`, and Alph AI `order/create` endpoints are **off-limits** for v1. Only read endpoints and `order quote` (pricing) are used. Real trading is a future milestone gated behind explicit user opt-in.

### 3. Use the skills
- **`memecoin-alpha-agent`** — domain knowledge (alpha filter, risk, mindset). Embed this mindset in the trading logic.
- **`rust-engineer`** — Rust idioms, error handling, async patterns.
- **`brainstorming`** / **`writing-plans`** — for any new feature before implementing.
- **`systematic-debugging`** — when fixing bugs.

### 4. Never hardcode secrets
API keys (`GMGN_API_KEY`, `ALPH_DEX_COOKIE`, `OPENAI_API_KEY`, etc.) come from `~/.config/quickscope/.env` or environment variables. See `.env.example`. Never commit real credentials.

### 5. Respect rate limits
- **GMGN:** Leaky bucket, rate=20, capacity=20. Each endpoint has a known weight (1-5). See `docs/api-reference/gmgn-endpoints.md`.
- **Alph AI:** Cookie-based, undocumented limits — be conservative, cache aggressively.
- **DEX Screener:** Free tier, be polite.
- On 429: read `reset_at`, sleep, retry once. Never hammer.

### 6. IPv6 warning
GMGN does NOT support IPv6. If `gmgn` calls fail with 401/403, check IPv6 first. This applies to our `reqwest` calls too — consider forcing IPv4 in the HTTP client config.

---

## Module Map

```
src/
├── main.rs              # Entry point, TUI event loop
├── app/                 # AppState, TokenListMode, input router
├── ui/                  # ratatui rendering: sidebar + 7 tabs + 15 widgets + theme + format
├── data/                # DataOrchestrator + GMGN + Alph AI + DEX Screener clients
├── alpha/               # Alpha Filter: scoring, rug detection, narrative, modes
├── trade/               # Paper trade engine: simulator, position, monitor, risk
├── learning/            # Auto-tuner + LLM post-mortem (OpenAI/Anthropic/Ollama)
└── storage/             # SQLite via sqlx: positions, journal, config, cache
```

Detailed responsibilities: see `docs/architecture.md`.

---

## Data Sources

| Source | Role | Auth | Notes |
|---|---|---|---|---|
| **GMGN** (primary) | Trending, trenches, token info/security, smart money, pricing, signals | Ed25519 keypair (handled by `gmgn-cli` subprocess) | QuickScope calls `gmgn-cli` as subprocess, not raw HTTP. Ed25519 signing required. See `docs/api-reference/gmgn-endpoints.md`. |
| **Alph AI** (secondary) | Twitter/X monitoring, real-time WS with auto-reconnect, signals, smart wallets | `ALPH_DEX_COOKIE` (14-day browser cookie) | Fills GMGN's Twitter gap. REST + WebSocket with exponential backoff reconnection. See `docs/api-reference/alph-ai-endpoints.md`. |
| **DEX Screener** (tertiary) | Trending/boosts cross-reference | None (free) | Boosts conviction multiplier. See `docs/api-reference/dex-screener.md`. |

---

## Scanner Mode Selector

The Scanner tab has a **mode selector bar** at the top with four data sources:
- **Trending** — GMGN's trending tokens (default)
- **Trenches** — Newly launched tokens from `gmgn-cli market trenches`
- **Watchlist** — Filtered view showing only watchlisted tokens
- **AI-Rec** — Tokens with Gold/Silver confidence signals from Alph AI

Switch modes with `←`/`→` arrow keys when in the Scanner tab. Press `r` to refresh the current mode's data.

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `↑`/`↓` | Navigate lists |
| `←`/`→` | Switch tabs (or switch Scanner mode when in Scanner tab) |
| `Enter` | Select / View token detail |
| `Tab` / `Shift+Tab` | Next/Previous tab |
| `r` | Refresh data (trenches if in Trenches mode) |
| `/` | Search tokens |
| `f` | Filter modal |
| `Space` | Watchlist toggle |
| `Esc` | Close modal / Back |
| `?` | Help screen |
| `Ctrl+P` | Command palette |
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+E` | Emergency exit all |
| `Ctrl+C` or `q` | Quit |

**No VIM-style bindings** — all navigation uses arrow keys and explicit shortcuts.

---

## Modal System

All modals use a **dimmed backdrop** (solid black fill) that obscures the content behind them, preventing ghost clicks and layering issues:
- **Error modals:** Centered panel with dark background, colored border, action hints at bottom
- **Command palette:** Centered overlay with search bar, filtered command list, keyboard navigation
- **Confirm dialogs:** Enter/ESC action hints displayed at bottom

Modals intentionally ignore mouse events to prevent accidental clicks on underlying content.

---

## Theme System

Four themes are available, cycled via `Ctrl+P` → Cycle Theme:

| Theme | Description |
|-------|-------------|
| **Dark** | OpenCode-inspired, blue accents on dark gray |
| **Terminal** | Bloomberg-style, amber/green on deep black |
| **Degen** | Neon green on dark purple |
| **Cyberpunk** | Cyan/pink on dark purple |

---

## Development Commands

```bash
# Build
cargo build

# Run the TUI
cargo run

# Run all tests (105 total)
cargo test

# Check without building
cargo check

# Format
cargo fmt

# Lint (zero warnings required)
cargo clippy -- -D warnings

# Run a specific test
cargo test scoring::tests
```

---

## Configuration

Config lives in `~/.config/quickscope/.env`:

```
GMGN_API_KEY=gmgn_xxx
ALPH_DEX_COOKIE=xxx
OPENAI_API_KEY=sk-xxx
ANTHROPIC_API_KEY=sk-ant-xxx
OLLAMA_BASE_URL=http://localhost:11434
QUICKSCOPE_DB_PATH=~/.config/quickscope/data.db
QUICKSCOPE_LOG_LEVEL=info
```

Copy `.env.example` and fill in values. Missing API keys will trigger a fatal error modal on startup.

---

## Doc Directory Index

| Doc | Purpose |
|---|---|
| `docs/superpowers/specs/2026-07-05-quickscope-design.md` | **The master design spec.** Read first. |
| `docs/architecture.md` | System architecture, module responsibilities, data flow |
| `docs/data-sources.md` | How GMGN + Alph AI + DEX Screener combine |
| `docs/alpha-filter.md` | Scoring engine, feature vector, modes, rug detection |
| `docs/learning-engine.md` | Auto-tuner algorithm + LLM post-mortem flow |
| `docs/api-reference/gmgn-endpoints.md` | Every GMGN endpoint used, with weights |
| `docs/api-reference/alph-ai-endpoints.md` | Every Alph AI endpoint used |
| `docs/api-reference/dex-screener.md` | DEX Screener endpoints |
| `docs/plans/` | Implementation plans (one per phase) |

---

## Conventions

- **UI Layout:** `ui/layout.rs` uses sidebar | content split (not a top tab bar). Sidebar in `ui/sidebar.rs`.
- **Overlays:** All modals use the `Modal` widget (with `Clear` + dark backdrop). Command palette at `ui/widgets/command_palette.rs`.
- **Marketcap display:** Always use `format_marketcap()` + `marketcap_color()` from `ui/format.rs` for token values. Primary columns: marketcap, volume, change %. Price is secondary/muted.
- **Market cap parsing:** GMGN returns `market_cap` in some responses and `marketCap` in others. The parser tries both (`parse_f64(v, "market_cap").or_else(|| parse_f64(v, "marketCap"))`).
- **Toast notifications:** Use `state.notify()` to queue toasts. They auto-dismiss after 4s. Rendered via `Toast` widget.
- **Key bindings:** Arrow keys (`↑`/`↓`) for navigation. **No VIM-style `j`/`k`.** `Ctrl+P` for command palette, `Ctrl+B` for sidebar toggle. `←`/`→` switch tabs globally, or switch Scanner mode when in Scanner tab.
- **Mouse handling:** Content area clicks only fire on Dashboard, Scanner, and Analyzer tabs. Clicks on Trade, Journal, Strategy, Settings tabs are ignored. Modals and palettes block all mouse events.
- **Error handling:** `anyhow::Result` for application code, `thiserror` for library-style error enums in `data/` and `alpha/`. Fatal errors (missing API keys, DB failures) show a centered modal.
- **Async:** `tokio` runtime. All I/O (HTTP, WebSocket, SQLite) is async.
- **State sharing:** `Arc<tokio::sync::Mutex<AppState>>` for shared mutable state across tasks.
- **Logging:** `tracing` (never `println!` in libraries). TUI rendering must not log to stdout (it corrupts the terminal).
- **Types:** Strongly typed API responses via `serde`. Never parse JSON dynamically if a struct can model it.
- **Tests:** Unit tests in `#[cfg(test)] mod tests` per module. Integration tests in `tests/`. Use `mockall` to mock HTTP clients.
- **Commits:** Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`).

---

## When in doubt

1. Re-read the design spec.
2. Check the relevant doc in `docs/`.
3. Follow the `memecoin-alpha-agent` mindset: **preservation > multiplication**.
4. Ask the user before deviating from the spec.
