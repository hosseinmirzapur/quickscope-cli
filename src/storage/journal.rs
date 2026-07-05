use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;

/// Upsert daily risk tracking for today.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_daily_risk(
    pool: &SqlitePool,
    date: &str,
    starting_balance: f64,
    realized_pnl: f64,
    trades: i64,
    wins: i64,
    losses: i64,
    kill_switch: bool,
    overrides: i64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO daily_risk (date, starting_balance, daily_realized_pnl, trades_today, \
         wins_today, losses_today, kill_switch_active, override_count) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(date) DO UPDATE SET \
         daily_realized_pnl = excluded.daily_realized_pnl, \
         trades_today = excluded.trades_today, \
         wins_today = excluded.wins_today, \
         losses_today = excluded.losses_today, \
         kill_switch_active = excluded.kill_switch_active, \
         override_count = excluded.override_count"
    )
    .bind(date)
    .bind(starting_balance)
    .bind(realized_pnl)
    .bind(trades)
    .bind(wins)
    .bind(losses)
    .bind(kill_switch as i32)
    .bind(overrides)
    .execute(pool)
    .await?;
    Ok(())
}

/// End today's session.
pub async fn end_day(pool: &SqlitePool, date: &str) -> Result<()> {
    let ended_at = Utc::now().to_rfc3339();
    sqlx::query("UPDATE daily_risk SET ended_at = ? WHERE date = ?")
        .bind(&ended_at)
        .bind(date)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get today's risk row (or None).
pub async fn get_today_risk(pool: &SqlitePool, date: &str) -> Result<Option<DailyRiskRow>> {
    let row = sqlx::query_as::<_, DailyRiskRow>(
        "SELECT * FROM daily_risk WHERE date = ?"
    )
    .bind(date)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DailyRiskRow {
    pub date: String,
    pub starting_balance: f64,
    pub daily_realized_pnl: f64,
    pub trades_today: i64,
    pub wins_today: i64,
    pub losses_today: i64,
    pub kill_switch_active: i32,
    pub override_count: i64,
    pub ended_at: Option<String>,
}

impl DailyRiskRow {
    pub fn kill_switch_active_bool(&self) -> bool {
        self.kill_switch_active != 0
    }

    pub fn pnl_string(&self) -> String {
        if self.daily_realized_pnl >= 0.0 {
            format!("+{:.2} SOL", self.daily_realized_pnl)
        } else {
            format!("{:.2} SOL", self.daily_realized_pnl)
        }
    }

    pub fn win_rate(&self) -> f64 {
        let total = self.wins_today + self.losses_today;
        if total == 0 {
            0.0
        } else {
            self.wins_today as f64 / total as f64
        }
    }
}

// ── Watchlist ─────────────────────────────────────────────────

/// Add a token to the watchlist.
pub async fn add_to_watchlist(
    pool: &SqlitePool,
    token_address: &str,
    token_symbol: &str,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR IGNORE INTO watchlist (token_address, token_symbol, added_at) VALUES (?, ?, ?)"
    )
    .bind(token_address)
    .bind(token_symbol)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove a token from the watchlist.
pub async fn remove_from_watchlist(pool: &SqlitePool, token_address: &str) -> Result<()> {
    sqlx::query("DELETE FROM watchlist WHERE token_address = ?")
        .bind(token_address)
        .execute(pool)
        .await?;
    Ok(())
}

/// Check if a token is on the watchlist.
pub async fn is_watchlisted(pool: &SqlitePool, token_address: &str) -> Result<bool> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM watchlist WHERE token_address = ?"
    )
    .bind(token_address)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

/// Get all watchlisted tokens.
pub async fn get_watchlist(pool: &SqlitePool) -> Result<Vec<WatchlistRow>> {
    let rows = sqlx::query_as::<_, WatchlistRow>(
        "SELECT * FROM watchlist ORDER BY added_at DESC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WatchlistRow {
    pub id: i64,
    pub token_address: String,
    pub token_symbol: String,
    pub added_at: String,
}

// ── Tuning History ──────────────────────────────────────────────

/// Log an auto-tune run.
#[allow(clippy::too_many_arguments)]
pub async fn log_tuning_run(
    pool: &SqlitePool,
    sample_size: i64,
    wins: i64,
    losses: i64,
    old_weights_json: &str,
    new_weights_json: &str,
    old_filters_json: &str,
    new_filters_json: &str,
    discrimination_json: &str,
) -> Result<()> {
    let tuned_at = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO tuning_history (tuned_at, sample_size, wins, losses, \
         old_weights, new_weights, old_filters, new_filters, discrimination) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&tuned_at)
    .bind(sample_size)
    .bind(wins)
    .bind(losses)
    .bind(old_weights_json)
    .bind(new_weights_json)
    .bind(old_filters_json)
    .bind(new_filters_json)
    .bind(discrimination_json)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get the most recent N tuning runs.
pub async fn get_recent_tuning_runs(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<TuningHistoryRow>> {
    let rows = sqlx::query_as::<_, TuningHistoryRow>(
        "SELECT * FROM tuning_history ORDER BY tuned_at DESC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TuningHistoryRow {
    pub id: i64,
    pub tuned_at: String,
    pub sample_size: i64,
    pub wins: i64,
    pub losses: i64,
    pub old_weights: String,
    pub new_weights: String,
    pub old_filters: String,
    pub new_filters: String,
    pub discrimination: String,
}

// ── Post-mortems ────────────────────────────────────────────────

/// Save an LLM post-mortem.
pub async fn log_post_mortem(
    pool: &SqlitePool,
    period_start: &str,
    period_end: &str,
    provider: &str,
    model: &str,
    prompt_summary: &str,
    response: &str,
) -> Result<i64> {
    let run_at = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO post_mortems (run_at, period_start, period_end, provider, model, \
         prompt_summary, response) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&run_at)
    .bind(period_start)
    .bind(period_end)
    .bind(provider)
    .bind(model)
    .bind(prompt_summary)
    .bind(response)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Get the most recent N post-mortems.
pub async fn get_recent_post_mortems(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<PostMortemRow>> {
    let rows = sqlx::query_as::<_, PostMortemRow>(
        "SELECT * FROM post_mortems ORDER BY run_at DESC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PostMortemRow {
    pub id: i64,
    pub run_at: String,
    pub period_start: String,
    pub period_end: String,
    pub provider: String,
    pub model: String,
    pub prompt_summary: String,
    pub response: String,
    pub suggestions_applied: i64,
    pub suggestions_dismissed: i64,
}

// ── Portfolio ────────────────────────────────────────────────────

/// Get the portfolio singleton row.
pub async fn get_portfolio(pool: &SqlitePool) -> Result<(f64, String)> {
    let row: (f64, String) = sqlx::query_as(
        "SELECT balance_sol, updated_at FROM portfolios WHERE id = 1"
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Update portfolio balance and timestamp.
pub async fn update_portfolio_balance(
    pool: &SqlitePool,
    balance_sol: f64,
) -> Result<()> {
    let updated_at = Utc::now().to_rfc3339();
    sqlx::query("UPDATE portfolios SET balance_sol = ?, updated_at = ? WHERE id = 1")
        .bind(balance_sol)
        .bind(&updated_at)
        .execute(pool)
        .await?;
    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::DbManager;

    async fn setup_db() -> (DbManager, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = DbManager::new(db_path.to_str().unwrap()).await.unwrap();
        (db, dir)
    }

    #[tokio::test]
    async fn test_daily_risk_upsert() {
        let (db, _dir) = setup_db().await;
        let today = Utc::now().format("%Y-%m-%d").to_string();

        upsert_daily_risk(&db.pool, &today, 50.0, 2.5, 3, 2, 1, false, 0)
            .await.unwrap();

        let risk = get_today_risk(&db.pool, &today).await.unwrap().unwrap();
        assert_eq!(risk.daily_realized_pnl, 2.5);
        assert_eq!(risk.trades_today, 3);
        assert!(!risk.kill_switch_active_bool());
        assert_eq!(risk.win_rate(), 2.0 / 3.0);

        // Upsert with updated values
        upsert_daily_risk(&db.pool, &today, 50.0, -1.0, 5, 2, 3, true, 0)
            .await.unwrap();
        let risk2 = get_today_risk(&db.pool, &today).await.unwrap().unwrap();
        assert_eq!(risk2.daily_realized_pnl, -1.0);
        assert!(risk2.kill_switch_active_bool());
    }

    #[tokio::test]
    async fn test_watchlist_crud() {
        let (db, _dir) = setup_db().await;

        add_to_watchlist(&db.pool, "So11abc", "PEPE").await.unwrap();
        add_to_watchlist(&db.pool, "So11xyz", "WIF").await.unwrap();

        assert!(is_watchlisted(&db.pool, "So11abc").await.unwrap());
        assert!(!is_watchlisted(&db.pool, "none").await.unwrap());

        let list = get_watchlist(&db.pool).await.unwrap();
        assert_eq!(list.len(), 2);

        remove_from_watchlist(&db.pool, "So11abc").await.unwrap();
        let list2 = get_watchlist(&db.pool).await.unwrap();
        assert_eq!(list2.len(), 1);
        assert_eq!(list2[0].token_symbol, "WIF");
    }

    #[tokio::test]
    async fn test_tuning_history_log_and_get() {
        let (db, _dir) = setup_db().await;

        log_tuning_run(
            &db.pool, 20, 12, 8,
            r#"{"w_momentum":0.25}"#, r#"{"w_momentum":0.27}"#,
            r#"{"hf_rug_ratio_max":0.30}"#, r#"{"hf_rug_ratio_max":0.28}"#,
            r#"{"momentum":0.45}"#,
        ).await.unwrap();

        let runs = get_recent_tuning_runs(&db.pool, 5).await.unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].wins, 12);
        assert!(runs[0].new_weights.contains("0.27"));
    }

    #[tokio::test]
    async fn test_post_mortem_log_and_get() {
        let (db, _dir) = setup_db().await;

        let id = log_post_mortem(
            &db.pool, "2026-07-01", "2026-07-05",
            "openai", "gpt-4o",
            "Review trades", "Great job, but tighten SL",
        ).await.unwrap();
        assert!(id > 0);

        let mortems = get_recent_post_mortems(&db.pool, 5).await.unwrap();
        assert_eq!(mortems.len(), 1);
        assert_eq!(mortems[0].model, "gpt-4o");
        assert!(mortems[0].response.contains("tighten SL"));
    }

    #[tokio::test]
    async fn test_portfolio_get_and_update() {
        let (db, _dir) = setup_db().await;

        let (balance, _) = get_portfolio(&db.pool).await.unwrap();
        assert_eq!(balance, 50.0);

        update_portfolio_balance(&db.pool, 55.0).await.unwrap();
        let (balance2, _) = get_portfolio(&db.pool).await.unwrap();
        assert_eq!(balance2, 55.0);
    }
}