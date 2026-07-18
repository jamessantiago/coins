use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS telegram_channels (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    username  TEXT NOT NULL UNIQUE,
    enabled   INTEGER NOT NULL DEFAULT 1,
    chat_id   INTEGER,
    added_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS telegram_messages (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    channel_id          INTEGER NOT NULL REFERENCES telegram_channels(id),
    message_id          INTEGER NOT NULL,
    text                TEXT NOT NULL DEFAULT '',
    extracted_addresses TEXT NOT NULL DEFAULT '',
    posted_at           TEXT NOT NULL,
    detected_at         TEXT NOT NULL,
    UNIQUE(channel_id, message_id)
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
