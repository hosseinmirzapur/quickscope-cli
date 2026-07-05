use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;

use super::migrations::SCHEMA;

/// Manages the SQLite connection pool and schema initialization.
pub struct DbManager {
    pub pool: SqlitePool,
}

impl DbManager {
    /// Open a SQLite database, creating it and all tables if they don't exist.
    pub async fn new(db_path: &str) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating db directory {:?}", parent))?;
        }

        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true)
            .synchronous(SqliteSynchronous::Normal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("connecting to SQLite")?;

        // Apply schema: split into individual statements and execute each
        let statements: Vec<&str> = SCHEMA
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for stmt in statements {
            sqlx::query(stmt)
                .execute(&pool)
                .await
                .with_context(|| format!("running schema statement: {}...", &stmt[..stmt.len().min(60)]))?;
        }

        tracing::info!(db_path = %db_path, "Database initialized");

        Ok(Self { pool })
    }

    /// Run a health check query.
    pub async fn health_check(&self) -> Result<()> {
        let _: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}