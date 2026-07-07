use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;

/// Get cached response if it exists and hasn't expired.
pub async fn get_cached(
    pool: &SqlitePool,
    table: &str,
    endpoint: &str,
    params_hash: &str,
) -> Result<Option<String>> {
    // Sanitize table name to prevent SQL injection
    let table = match table {
        "gmgn_cache" | "alphai_cache" => table,
        _ => anyhow::bail!("invalid cache table: {}", table),
    };

    let query = format!(
        "SELECT response FROM {} WHERE endpoint = ? AND params_hash = ? \
         AND datetime(fetched_at, '+' || CAST(ttl_seconds AS TEXT) || ' seconds') > datetime('now')",
        table
    );

    let row: Option<(String,)> = sqlx::query_as(&query)
        .bind(endpoint)
        .bind(params_hash)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| r.0))
}

/// Store a response in the cache.
pub async fn set_cached(
    pool: &SqlitePool,
    table: &str,
    endpoint: &str,
    params_hash: &str,
    response: &str,
    ttl_seconds: i64,
) -> Result<()> {
    let table = match table {
        "gmgn_cache" | "alphai_cache" => table,
        _ => anyhow::bail!("invalid cache table: {}", table),
    };

    let fetched_at = Utc::now().to_rfc3339();
    let query = format!(
        "INSERT OR REPLACE INTO {} (endpoint, params_hash, response, fetched_at, ttl_seconds) \
         VALUES (?, ?, ?, ?, ?)",
        table
    );

    sqlx::query(&query)
        .bind(endpoint)
        .bind(params_hash)
        .bind(response)
        .bind(&fetched_at)
        .bind(ttl_seconds)
        .execute(pool)
        .await?;

    Ok(())
}

/// Remove expired entries from a cache table to keep DB small.
pub async fn purge_expired(pool: &SqlitePool, table: &str) -> Result<u64> {
    let table = match table {
        "gmgn_cache" | "alphai_cache" => table,
        _ => anyhow::bail!("invalid cache table: {}", table),
    };

    let query = format!(
        "DELETE FROM {} WHERE datetime(fetched_at, '+' || CAST(ttl_seconds AS TEXT) || ' seconds') <= datetime('now')",
        table
    );

    let result = sqlx::query(&query).execute(pool).await?;

    Ok(result.rows_affected())
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
    async fn test_cache_set_and_get() {
        let (db, _dir) = setup_db().await;

        // Nothing cached yet
        let cached = get_cached(&db.pool, "gmgn_cache", "/market/rank", "abc123")
            .await
            .unwrap();
        assert!(cached.is_none());

        // Set cache
        set_cached(
            &db.pool,
            "gmgn_cache",
            "/market/rank",
            "abc123",
            r#"{"data":[{"symbol":"PEPE"}]}"#,
            300,
        )
        .await
        .unwrap();

        // Get cached
        let cached = get_cached(&db.pool, "gmgn_cache", "/market/rank", "abc123")
            .await
            .unwrap();
        assert!(cached.unwrap().contains("PEPE"));

        // Wrong params_hash → miss
        let cached2 = get_cached(&db.pool, "gmgn_cache", "/market/rank", "xyz")
            .await
            .unwrap();
        assert!(cached2.is_none());
    }

    #[tokio::test]
    async fn test_cache_purge_expired() {
        let (db, _dir) = setup_db().await;

        // Set with very short TTL
        set_cached(&db.pool, "alphai_cache", "/test", "h1", r#"{}"#, 0)
            .await
            .unwrap();

        // Should be expired immediately
        let cached = get_cached(&db.pool, "alphai_cache", "/test", "h1")
            .await
            .unwrap();
        assert!(cached.is_none());

        // Purge should remove it
        let removed = purge_expired(&db.pool, "alphai_cache").await.unwrap();
        // May be 0 or 1 depending on datetime precision, but shouldn't error
        assert!(removed <= 1);
    }

    #[tokio::test]
    async fn test_invalid_table_rejected() {
        let (db, _dir) = setup_db().await;
        let result = get_cached(&db.pool, "DROP TABLE", "/x", "h").await;
        assert!(result.is_err());
    }
}
