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
| Scaffolding (Cargo, dirs, stubs) | ✅ Complete |
| Storage (SQLite, positions, config, cache) | ✅ Complete |
| Data — GMGN client (via gmgn-cli subprocess) | ✅ Complete |
| Data — Alph AI client (REST + WebSocket skeleton) | ✅ Complete |
| Data — DEX Screener client + DataOrchestrator merge | ✅ Complete |
| Alpha Filter engine (feature vector, 6-category scoring, rug, modes, narrative) | ✅ Complete |
| Paper Trade Engine (simulator, risk manager, TP/SL monitor) | ✅ Complete |
| Learning Engine (auto-tuner, discrimination analyzer, LLM providers) | ✅ Complete |
| TUI Core (AppState, responsive event loop, theme system, 9 shared widgets) | ✅ Complete |
| TUI Tabs (all 7 — Dashboard, Scanner, Analyzer, Trade, Journal, Strategy, Settings) | ✅ Complete |
| UI Redesign (sidebar, command palette, marketcap/volume focus, toast, Clear modals) | ✅ Complete |
| Integration, polish, docs | ✅ Complete — 103 tests, all docs updated |

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
├── app/                 # AppState, tab manager, input router
├── ui/                  # ratatui rendering: sidebar + 7 tabs + reusable widgets + theme + format
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
| **GMGN** (primary) | Trending, trenches, token info/security, smart money, pricing | Ed25519 keypair (handled by `gmgn-cli` subprocess) | QuickScope calls `gmgn-cli` as subprocess, not raw HTTP. Ed25519 signing required. See `docs/api-reference/gmgn-endpoints.md`. |
| **Alph AI** (secondary) | Twitter/X monitoring, real-time WS, signals, smart wallets | `ALPH_DEX_COOKIE` (14-day browser cookie) | Fills GMGN's Twitter gap. REST + WebSocket. See `docs/api-reference/alph-ai-endpoints.md`. |
| **DEX Screener** (tertiary) | Trending/boosts cross-reference | None (free) | Boosts conviction multiplier. See `docs/api-reference/dex-screener.md`. |

---

## Development Commands

```bash
# Build
cargo build

# Run the TUI
cargo run

# Run tests
cargo test

# Check without building
cargo check

# Format
cargo fmt

# Lint
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

Copy `.env.example` and fill in values.

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
- **Overlays:** All modals use the `Modal` widget (with `Clear` backdrop). Command palette at `ui/widgets/command_palette.rs`.
- **Marketcap display:** Always use `format_marketcap()` + `marketcap_color()` from `ui/format.rs` for token values. Primary columns: marketcap, volume, change %. Price is secondary/muted.
- **Toast notifications:** Use `state.notify()` to queue toasts. They auto-dismiss after 4s. Rendered via `Toast` widget.
- **Key bindings:** Arrow keys (`↑`/`↓`) for navigation. No VIM-style `j`/`k`. `Ctrl+P` for command palette, `Ctrl+B` for sidebar toggle.
- **Error handling:** `anyhow::Result` for application code, `thiserror` for library-style error enums in `data/` and `alpha/`.
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
