use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use argusx_common::config::DatabaseConfig;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::error::Result;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Type alias for DatabaseConfig from argusx-common
pub type DbConfig = DatabaseConfig;

fn database_url(config: &DatabaseConfig) -> String {
    format!("sqlite://{}", config.path)
}

pub async fn ensure_parent_directory(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    Ok(())
}

pub async fn connect(config: &DbConfig) -> Result<SqlitePool> {
    let db_path = PathBuf::from(&config.path);
    ensure_parent_directory(&db_path).await?;

    let options = SqliteConnectOptions::from_str(&database_url(config))?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_millis(config.busy_timeout_ms));

    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PragmaStatus {
    pub foreign_keys: i64,
    pub journal_mode: String,
    pub busy_timeout: i64,
}

pub async fn pragma_status(pool: &SqlitePool) -> Result<PragmaStatus> {
    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys;")
        .fetch_one(pool)
        .await?;
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode;")
        .fetch_one(pool)
        .await?;
    let busy_timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout;")
        .fetch_one(pool)
        .await?;

    Ok(PragmaStatus {
        foreign_keys,
        journal_mode,
        busy_timeout,
    })
}
