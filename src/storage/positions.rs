use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Insert a new paper position. Returns the new position ID.
#[allow(clippy::too_many_arguments)]
pub async fn insert_position(
    pool: &SqlitePool,
    token_address: &str,
    token_symbol: &str,
    side: &str,
    entry_price: f64,
    amount_sol: f64,
    amount_tokens: f64,
    slippage: f64,
    mode: &str,
    tp_percent: Option<f64>,
    sl_percent: Option<f64>,
    feature_vector_json: &str,
    alpha_score: f64,
    rug_report_json: &str,
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let opened_at = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO positions (id, token_address, token_symbol, side, entry_price, \
         amount_sol, amount_tokens, slippage, mode, tp_percent, sl_percent, status, \
         opened_at, feature_vector, alpha_score, rug_report) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'open', ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(token_address)
    .bind(token_symbol)
    .bind(side)
    .bind(entry_price)
    .bind(amount_sol)
    .bind(amount_tokens)
    .bind(slippage)
    .bind(mode)
    .bind(tp_percent)
    .bind(sl_percent)
    .bind(&opened_at)
    .bind(feature_vector_json)
    .bind(alpha_score)
    .bind(rug_report_json)
    .execute(pool)
    .await?;

    Ok(id)
}

/// A flat row returned from the positions table.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct PositionRow {
    pub id: String,
    pub token_address: String,
    pub token_symbol: String,
    pub side: String,
    pub entry_price: f64,
    pub amount_sol: f64,
    pub amount_tokens: f64,
    pub slippage: f64,
    pub mode: String,
    pub tp_percent: Option<f64>,
    pub sl_percent: Option<f64>,
    pub status: String,
    pub opened_at: String,
    pub closed_at: Option<String>,
    pub exit_price: Option<f64>,
    pub pnl_sol: Option<f64>,
    pub pnl_percent: Option<f64>,
    pub feature_vector: Option<String>,
    pub alpha_score: f64,
    pub rug_report: Option<String>,
}

/// Get all currently open positions, newest first.
pub async fn get_open_positions(pool: &SqlitePool) -> Result<Vec<PositionRow>> {
    let rows = sqlx::query_as::<_, PositionRow>(
        "SELECT * FROM positions WHERE status = 'open' ORDER BY opened_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get closed positions within a date range (optional bounds).
pub async fn get_closed_positions(
    pool: &SqlitePool,
    since: Option<&str>,
    until: Option<&str>,
) -> Result<Vec<PositionRow>> {
    let mut query = "SELECT * FROM positions WHERE status = 'closed'".to_string();
    if since.is_some() {
        query.push_str(" AND closed_at >= ?");
    }
    if until.is_some() {
        query.push_str(" AND closed_at <= ?");
    }
    query.push_str(" ORDER BY closed_at DESC");

    let mut q = sqlx::query_as::<_, PositionRow>(&query);
    if let Some(s) = since {
        q = q.bind(s);
    }
    if let Some(u) = until {
        q = q.bind(u);
    }
    let rows = q.fetch_all(pool).await?;
    Ok(rows)
}

/// Count open positions.
pub async fn count_open_positions(pool: &SqlitePool) -> Result<i64> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM positions WHERE status = 'open'")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

/// Count open positions for a specific token.
pub async fn count_open_for_token(pool: &SqlitePool, token_address: &str) -> Result<i64> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM positions WHERE status = 'open' AND token_address = ?",
    )
    .bind(token_address)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// Close a position with exit price and PnL.
pub async fn close_position(
    pool: &SqlitePool,
    id: &str,
    exit_price: f64,
    pnl_sol: f64,
    pnl_percent: f64,
) -> Result<()> {
    let closed_at = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE positions SET status = 'closed', closed_at = ?, exit_price = ?, \
         pnl_sol = ?, pnl_percent = ? WHERE id = ?",
    )
    .bind(&closed_at)
    .bind(exit_price)
    .bind(pnl_sol)
    .bind(pnl_percent)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a single position by ID.
pub async fn get_position_by_id(pool: &SqlitePool, id: &str) -> Result<Option<PositionRow>> {
    let row = sqlx::query_as::<_, PositionRow>("SELECT * FROM positions WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

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
    async fn test_insert_and_get_open() {
        let (db, _dir) = setup_db().await;
        let id = insert_position(
            &db.pool,
            "So11abc",
            "PEPE",
            "buy",
            0.000018,
            0.5,
            27777.0,
            0.03,
            "EXPLODE",
            Some(100.0),
            Some(60.0),
            "{}",
            87.0,
            "{}",
        )
        .await
        .unwrap();
        assert!(!id.is_empty());

        let open = get_open_positions(&db.pool).await.unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].token_symbol, "PEPE");
    }

    #[tokio::test]
    async fn test_close_position() {
        let (db, _dir) = setup_db().await;
        let id = insert_position(
            &db.pool,
            "So11xyz",
            "WIF",
            "buy",
            0.5,
            1.0,
            2.0,
            0.02,
            "ALPHA",
            Some(80.0),
            Some(40.0),
            "{}",
            72.0,
            "{}",
        )
        .await
        .unwrap();

        close_position(&db.pool, &id, 0.75, 0.5, 50.0)
            .await
            .unwrap();

        let open = get_open_positions(&db.pool).await.unwrap();
        assert_eq!(open.len(), 0);

        let closed = get_closed_positions(&db.pool, None, None).await.unwrap();
        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].pnl_sol, Some(0.5));
    }

    #[tokio::test]
    async fn test_count_open_positions() {
        let (db, _dir) = setup_db().await;
        assert_eq!(count_open_positions(&db.pool).await.unwrap(), 0);

        insert_position(
            &db.pool, "A", "A", "buy", 1.0, 1.0, 1.0, 0.01, "SCALP", None, None, "{}", 50.0, "{}",
        )
        .await
        .unwrap();
        insert_position(
            &db.pool, "B", "B", "buy", 1.0, 1.0, 1.0, 0.01, "FALLBACK", None, None, "{}", 40.0,
            "{}",
        )
        .await
        .unwrap();

        assert_eq!(count_open_positions(&db.pool).await.unwrap(), 2);
        assert_eq!(count_open_for_token(&db.pool, "A").await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_get_position_by_id() {
        let (db, _dir) = setup_db().await;
        let id = insert_position(
            &db.pool,
            "So11abc",
            "BONK",
            "buy",
            0.000001,
            0.1,
            100000.0,
            0.01,
            "EXPLODE",
            Some(200.0),
            Some(50.0),
            "{\"hot_level\":5}",
            92.0,
            "{}",
        )
        .await
        .unwrap();

        let pos = get_position_by_id(&db.pool, &id).await.unwrap().unwrap();
        assert_eq!(pos.alpha_score, 92.0);
        assert_eq!(pos.token_symbol, "BONK");
        assert!(pos.feature_vector.unwrap().contains("hot_level"));

        let none = get_position_by_id(&db.pool, "nonexistent").await.unwrap();
        assert!(none.is_none());
    }
}
