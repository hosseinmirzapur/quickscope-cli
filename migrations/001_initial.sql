-- QuickScope initial schema (SQLite)
-- Run on first launch via storage::migrations.

-- Paper portfolio state (singleton row)
CREATE TABLE IF NOT EXISTS portfolios (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    balance_sol REAL NOT NULL DEFAULT 50.0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO portfolios (id, balance_sol) VALUES (1, 50.0);

-- Paper positions (open and closed)
CREATE TABLE IF NOT EXISTS positions (
    id              TEXT PRIMARY KEY,                       -- UUID
    token_address   TEXT NOT NULL,
    token_symbol    TEXT NOT NULL,
    side            TEXT NOT NULL CHECK (side IN ('buy', 'sell')),
    entry_price     REAL NOT NULL,
    amount_sol      REAL NOT NULL,
    amount_tokens   REAL NOT NULL,
    slippage        REAL NOT NULL,
    mode            TEXT NOT NULL CHECK (mode IN ('EXPLODE', 'ALPHA', 'SCALP', 'FALLBACK')),
    tp_percent      REAL,
    sl_percent      REAL,
    status          TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed')),
    opened_at       TEXT NOT NULL,
    closed_at       TEXT,
    exit_price      REAL,
    pnl_sol         REAL,
    pnl_percent     REAL,
    feature_vector  TEXT NOT NULL,                          -- JSON blob
    alpha_score     REAL NOT NULL,
    rug_report      TEXT NOT NULL                           -- JSON blob
);

CREATE INDEX IF NOT EXISTS idx_positions_status ON positions(status);
CREATE INDEX IF NOT EXISTS idx_positions_token ON positions(token_address);
CREATE INDEX IF NOT EXISTS idx_positions_opened_at ON positions(opened_at);

-- Daily risk tracking
CREATE TABLE IF NOT EXISTS daily_risk (
    date                TEXT PRIMARY KEY,                   -- YYYY-MM-DD
    starting_balance    REAL NOT NULL,
    daily_realized_pnl  REAL NOT NULL DEFAULT 0,
    trades_today        INTEGER NOT NULL DEFAULT 0,
    wins_today          INTEGER NOT NULL DEFAULT 0,
    losses_today        INTEGER NOT NULL DEFAULT 0,
    kill_switch_active  INTEGER NOT NULL DEFAULT 0,
    override_count      INTEGER NOT NULL DEFAULT 0,
    ended_at            TEXT
);

-- Alpha filter weights/thresholds (singleton row, mutable)
CREATE TABLE IF NOT EXISTS alpha_config (
    id          INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    w_momentum  REAL NOT NULL DEFAULT 0.25,
    w_safety    REAL NOT NULL DEFAULT 0.15,
    w_holder    REAL NOT NULL DEFAULT 0.20,
    w_liquidity REAL NOT NULL DEFAULT 0.18,
    w_dev       REAL NOT NULL DEFAULT 0.07,
    w_social    REAL NOT NULL DEFAULT 0.15,
    hf_rug_ratio_max       REAL NOT NULL DEFAULT 0.30,
    hf_dev_hold_max        REAL NOT NULL DEFAULT 0.15,
    hf_wash_trading        INTEGER NOT NULL DEFAULT 1,
    hf_renounced_mint      INTEGER NOT NULL DEFAULT 1,
    hf_liquidity_min_usd   REAL NOT NULL DEFAULT 5000.0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO alpha_config (id) VALUES (1);

-- Tuning history (every auto-tune delta)
CREATE TABLE IF NOT EXISTS tuning_history (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tuned_at        TEXT NOT NULL,
    sample_size     INTEGER NOT NULL,
    wins            INTEGER NOT NULL,
    losses          INTEGER NOT NULL,
    old_weights     TEXT NOT NULL,                          -- JSON
    new_weights     TEXT NOT NULL,                          -- JSON
    old_filters     TEXT NOT NULL,                          -- JSON
    new_filters     TEXT NOT NULL,                          -- JSON
    discrimination  TEXT NOT NULL                           -- JSON
);

-- LLM post-mortem history
CREATE TABLE IF NOT EXISTS post_mortems (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    run_at          TEXT NOT NULL,
    period_start    TEXT NOT NULL,
    period_end      TEXT NOT NULL,
    provider        TEXT NOT NULL,
    model           TEXT NOT NULL,
    prompt_summary  TEXT NOT NULL,
    response        TEXT NOT NULL,
    suggestions_applied INTEGER NOT NULL DEFAULT 0,
    suggestions_dismissed INTEGER NOT NULL DEFAULT 0
);

-- Settings (key-value)
CREATE TABLE IF NOT EXISTS settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Watchlist
CREATE TABLE IF NOT EXISTS watchlist (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    token_address TEXT NOT NULL UNIQUE,
    token_symbol TEXT NOT NULL,
    added_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- GMGN API response cache
CREATE TABLE IF NOT EXISTS gmgn_cache (
    endpoint    TEXT NOT NULL,
    params_hash TEXT NOT NULL,
    response    TEXT NOT NULL,
    fetched_at  TEXT NOT NULL,
    ttl_seconds INTEGER NOT NULL DEFAULT 30,
    PRIMARY KEY (endpoint, params_hash)
);

-- Alph AI API response cache
CREATE TABLE IF NOT EXISTS alphai_cache (
    endpoint    TEXT NOT NULL,
    params_hash TEXT NOT NULL,
    response    TEXT NOT NULL,
    fetched_at  TEXT NOT NULL,
    ttl_seconds INTEGER NOT NULL DEFAULT 30,
    PRIMARY KEY (endpoint, params_hash)
);
