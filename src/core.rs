//! Shared core logic for both TUI and web server.
//!
//! `AppCore` encapsulates all business logic: data orchestration, alpha filtering,
//! paper trading, learning engine, and database access.

use std::sync::Arc;

use anyhow::Result;
use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::alpha;
use crate::data::models::*;
use crate::data::orchestrator::DataOrchestrator;
use crate::learning::LlmProvider;
use crate::storage::positions::PositionRow;
use crate::storage::journal::{WatchlistRow, TuningHistoryRow, PostMortemRow};
use crate::storage::DbManager;
use crate::trade::{TradeEngine, PaperBuyResult, PaperSellResult};

/// Application configuration loaded from environment.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub alph_dex_cookie: String,
    pub db_path: String,
    pub gmgn_api_key: String,
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub ollama_base_url: Option<String>,
    pub log_level: String,
}

impl AppConfig {
    /// Load from environment variables.
    pub fn from_env() -> Self {
        Self {
            alph_dex_cookie: std::env::var("ALPH_DEX_COOKIE").unwrap_or_default(),
            gmgn_api_key: std::env::var("GMGN_API_KEY").unwrap_or_default(),
            db_path: std::env::var("QUICKSCOPE_DB_PATH").unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.config/quickscope/data.db", home)
            }),
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            ollama_base_url: std::env::var("OLLAMA_BASE_URL").ok(),
            log_level: std::env::var("QUICKSCOPE_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        }
    }

    pub fn has_critical_keys(&self) -> bool {
        !self.alph_dex_cookie.is_empty()
    }

    pub fn llm_provider(&self) -> LlmProvider {
        if let Some(key) = &self.openai_api_key {
            if !key.is_empty() {
                return LlmProvider::OpenAi {
                    api_key: key.clone(),
                    model: "gpt-4o-mini".to_string(),
                };
            }
        }
        if let Some(key) = &self.anthropic_api_key {
            if !key.is_empty() {
                return LlmProvider::Anthropic {
                    api_key: key.clone(),
                    model: "claude-3-haiku-20240307".to_string(),
                };
            }
        }
        if let Some(url) = &self.ollama_base_url {
            if !url.is_empty() {
                return LlmProvider::Ollama {
                    base_url: url.clone(),
                    model: "llama3.2".to_string(),
                };
            }
        }
        LlmProvider::Stub {
            model: "stub".to_string(),
            response: "Set up an LLM API key (OPENAI_API_KEY, ANTHROPIC_API_KEY, or OLLAMA_BASE_URL) for real analysis.".to_string(),
        }
    }
}

/// Shared application core — all business logic lives here.
#[derive(Clone)]
pub struct AppCore {
    pub orchestrator: Arc<DataOrchestrator>,
    pub trade_engine: Arc<Mutex<TradeEngine>>,
    pub db: DbManager,
    pub pool: SqlitePool,
    pub config: AppConfig,
}

impl AppCore {
    /// Create a new AppCore instance.
    pub async fn new(config: AppConfig) -> Result<Self> {
        let orchestrator = Arc::new(DataOrchestrator::new(config.alph_dex_cookie.clone()));
        let db = DbManager::new(&config.db_path).await?;
        let pool = db.pool.clone();
        let trade_engine = Arc::new(Mutex::new(TradeEngine::new(db.clone())));

        Ok(Self {
            orchestrator,
            trade_engine,
            db,
            pool,
            config,
        })
    }

    // ── Data fetching ───────────────────────────────────────────────

    pub async fn fetch_trending(&self) -> Result<Vec<TrendingToken>> {
        self.orchestrator.fetch_trending().await
    }

    pub async fn fetch_trenches(&self, time_filter: &str) -> Result<Vec<TrenchToken>> {
        self.orchestrator.fetch_trenches(time_filter).await
    }

    pub async fn fetch_token_detail(&self, address: &str) -> Result<TokenDetail> {
        self.orchestrator.fetch_token_detail(address).await
    }

    pub async fn fetch_kline(
        &self,
        address: &str,
        resolution: &str,
        from: i64,
        to: i64,
    ) -> Result<Vec<KlineCandle>> {
        self.orchestrator.fetch_kline(address, resolution, from, to).await
    }

    pub async fn fetch_smart_money(&self, limit: u32) -> Result<Vec<SmartMoneyTrade>> {
        self.orchestrator.fetch_smart_money_trades(limit).await
    }

    pub async fn fetch_signals(&self) -> Result<Vec<TokenSignal>> {
        self.orchestrator.fetch_signals_gmgn().await
    }

    /// Analyze a token with the alpha filter.
    pub async fn analyze_token(
        &self,
        address: &str,
        config: &AlphaConfig,
    ) -> Result<(TokenDetail, AlphaReport)> {
        let detail = self.fetch_token_detail(address).await?;
        let report = alpha::analyze_token(&detail, config);
        Ok((detail, report))
    }

    pub async fn get_alpha_config(&self) -> Result<AlphaConfig> {
        crate::storage::config::load_alpha_config(&self.pool).await
    }

    pub async fn save_alpha_config(&self, config: &AlphaConfig) -> Result<()> {
        crate::storage::config::save_alpha_config(&self.pool, config).await?;
        Ok(())
    }

    // ── Paper trading ──────────────────────────────────────────────

    pub async fn paper_buy(
        &self,
        address: &str,
        amount_sol: f64,
        _mode: TradeMode,
        _tp_percent: Option<f64>,
        _sl_percent: Option<f64>,
    ) -> Result<PaperBuyResult> {
        let detail = self.fetch_token_detail(address).await?;
        let config = self.get_alpha_config().await?;
        let _report = alpha::analyze_token(&detail, &config);
        let sol_price_usd = 150.0; // TODO: fetch actual SOL price

        let mut engine = self.trade_engine.lock().await;
        engine.paper_buy(&detail, &config, amount_sol, detail.token.price_usd, sol_price_usd).await
    }

    pub async fn paper_sell(
        &self,
        position_id: &str,
        current_price_usd: f64,
        sell_percent: f64,
    ) -> Result<PaperSellResult> {
        let mut engine = self.trade_engine.lock().await;
        engine.paper_sell(position_id, current_price_usd, sell_percent).await
    }

    pub async fn get_open_positions(&self) -> Result<Vec<PositionRow>> {
        crate::storage::positions::get_open_positions(&self.pool).await
    }

    pub async fn get_position(&self, id: &str) -> Result<Option<PositionRow>> {
        crate::storage::positions::get_position_by_id(&self.pool, id).await
    }

    pub async fn emergency_exit_all(&self) -> Result<usize> {
        let positions = self.get_open_positions().await?;
        let mut count = 0;
        let mut engine = self.trade_engine.lock().await;
        for pos in &positions {
            if engine.paper_sell(&pos.id, 0.0, 100.0).await.is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    // ── Watchlist ──────────────────────────────────────────────────

    pub async fn add_to_watchlist(&self, address: &str, symbol: &str) -> Result<()> {
        crate::storage::journal::add_to_watchlist(&self.pool, address, symbol).await
    }

    pub async fn remove_from_watchlist(&self, address: &str) -> Result<()> {
        crate::storage::journal::remove_from_watchlist(&self.pool, address).await
    }

    pub async fn get_watchlist(&self) -> Result<Vec<WatchlistRow>> {
        crate::storage::journal::get_watchlist(&self.pool).await
    }

    // ── Portfolio ──────────────────────────────────────────────────

    pub async fn get_portfolio(&self) -> Result<(f64, String)> {
        crate::storage::journal::get_portfolio(&self.pool).await
    }

    pub async fn get_closed_positions(
        &self,
        since: Option<&str>,
        until: Option<&str>,
    ) -> Result<Vec<PositionRow>> {
        crate::storage::positions::get_closed_positions(&self.pool, since, until).await
    }

    // ── Tuning & post-mortems ──────────────────────────────────────

    pub async fn get_tuning_history(&self, limit: i64) -> Result<Vec<TuningHistoryRow>> {
        crate::storage::journal::get_recent_tuning_runs(&self.pool, limit).await
    }

    pub async fn get_post_mortems(&self, limit: i64) -> Result<Vec<PostMortemRow>> {
        crate::storage::journal::get_recent_post_mortems(&self.pool, limit).await
    }

    // ── Learning ───────────────────────────────────────────────────

    pub async fn run_auto_tune(&self) -> Result<Option<crate::learning::tuner::TuneResult>> {
        crate::learning::tuner::run_auto_tune(&self.pool).await
    }

    pub async fn run_post_mortem(&self, start_date: &str, end_date: &str) -> Result<String> {
        let provider = self.config.llm_provider();
        crate::learning::journal::run_post_mortem(&self.pool, &provider, start_date, end_date).await
    }

    // ── Risk ───────────────────────────────────────────────────────

    pub async fn toggle_kill_switch(&self) -> Result<bool> {
        let active = crate::storage::journal::get_today_risk(&self.pool, "today")
            .await
            .ok()
            .flatten()
            .map(|r| !r.kill_switch_active_bool())
            .unwrap_or(true);
        // TODO: persist the new state
        Ok(active)
    }
}
