use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS cex_listings (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    exchange     TEXT NOT NULL,
    external_id  TEXT NOT NULL,
    token_name   TEXT NOT NULL,
    token_symbol TEXT NOT NULL DEFAULT '',
    listing_url  TEXT NOT NULL DEFAULT '',
    announced_at TEXT,
    detected_at  TEXT NOT NULL,
    UNIQUE(exchange, external_id)
);

CREATE TABLE IF NOT EXISTS poll_timestamps (
    service        TEXT    NOT NULL PRIMARY KEY,
    last_run_at    TEXT    NOT NULL,
    listings_found INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS sse_events (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    event      TEXT    NOT NULL,
    data       TEXT    NOT NULL DEFAULT '{}',
    created_at TEXT    NOT NULL
);
"#;

pub async fn setup_memory_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .expect("failed to create in-memory pool");

    sqlx::query(SCHEMA_SQL)
        .execute(&pool)
        .await
        .expect("schema creation failed");

    pool
}
