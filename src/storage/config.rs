use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;

use crate::data::models::AlphaConfig;

/// Load the alpha config singleton from the database.
pub async fn load_alpha_config(pool: &SqlitePool) -> Result<AlphaConfig> {
    let row: (f64, f64, f64, f64, f64, f64, f64, f64, i32, i32, f64) = sqlx::query_as(
        "SELECT w_momentum, w_safety, w_holder, w_liquidity, w_dev, w_social, \
         hf_rug_ratio_max, hf_dev_hold_max, hf_wash_trading, hf_renounced_mint, \
         hf_liquidity_min_usd FROM alpha_config WHERE id = 1"
    )
    .fetch_one(pool)
    .await?;

    Ok(AlphaConfig {
        w_momentum: row.0,
        w_safety: row.1,
        w_holder: row.2,
        w_liquidity: row.3,
        w_dev: row.4,
        w_social: row.5,
        hf_rug_ratio_max: row.6,
        hf_dev_hold_max: row.7,
        hf_wash_trading: row.8 != 0,
        hf_renounced_mint: row.9 != 0,
        hf_liquidity_min_usd: row.10,
    })
}

/// Save updated alpha config (overwrites the singleton row).
pub async fn save_alpha_config(pool: &SqlitePool, config: &AlphaConfig) -> Result<()> {
    let updated_at = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE alpha_config SET \
         w_momentum = ?, w_safety = ?, w_holder = ?, w_liquidity = ?, \
         w_dev = ?, w_social = ?, \
         hf_rug_ratio_max = ?, hf_dev_hold_max = ?, \
         hf_wash_trading = ?, hf_renounced_mint = ?, \
         hf_liquidity_min_usd = ?, updated_at = ? \
         WHERE id = 1"
    )
    .bind(config.w_momentum)
    .bind(config.w_safety)
    .bind(config.w_holder)
    .bind(config.w_liquidity)
    .bind(config.w_dev)
    .bind(config.w_social)
    .bind(config.hf_rug_ratio_max)
    .bind(config.hf_dev_hold_max)
    .bind(config.hf_wash_trading as i32)
    .bind(config.hf_renounced_mint as i32)
    .bind(config.hf_liquidity_min_usd)
    .bind(&updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Load a single setting by key.
pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM settings WHERE key = ?"
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

/// Upsert a setting key-value pair.
pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<()> {
    let updated_at = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at"
    )
    .bind(key)
    .bind(value)
    .bind(&updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete a setting by key.
pub async fn delete_setting(pool: &SqlitePool, key: &str) -> Result<()> {
    sqlx::query("DELETE FROM settings WHERE key = ?")
        .bind(key)
        .execute(pool)
        .await?;
    Ok(())
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
    async fn test_load_default_alpha_config() {
        let (db, _dir) = setup_db().await;
        let config = load_alpha_config(&db.pool).await.unwrap();
        assert_eq!(config.w_momentum, 0.25);
        assert_eq!(config.hf_rug_ratio_max, 0.30);
        assert!(config.hf_wash_trading);
    }

    #[tokio::test]
    async fn test_save_and_reload_alpha_config() {
        let (db, _dir) = setup_db().await;
        let mut config = load_alpha_config(&db.pool).await.unwrap();
        config.w_momentum = 0.30;
        config.hf_rug_ratio_max = 0.25;
        save_alpha_config(&db.pool, &config).await.unwrap();

        let reloaded = load_alpha_config(&db.pool).await.unwrap();
        assert_eq!(reloaded.w_momentum, 0.30);
        assert_eq!(reloaded.hf_rug_ratio_max, 0.25);
    }

    #[tokio::test]
    async fn test_settings_crud() {
        let (db, _dir) = setup_db().await;

        // Initially missing
        assert!(get_setting(&db.pool, "theme").await.unwrap().is_none());

        set_setting(&db.pool, "theme", "degen").await.unwrap();
        assert_eq!(
            get_setting(&db.pool, "theme").await.unwrap().unwrap(),
            "degen"
        );

        // Update
        set_setting(&db.pool, "theme", "dark").await.unwrap();
        assert_eq!(
            get_setting(&db.pool, "theme").await.unwrap().unwrap(),
            "dark"
        );

        // Delete
        delete_setting(&db.pool, "theme").await.unwrap();
        assert!(get_setting(&db.pool, "theme").await.unwrap().is_none());
    }
}