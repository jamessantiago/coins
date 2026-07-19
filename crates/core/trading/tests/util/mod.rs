use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

pub async fn setup_pool() -> SqlitePool {
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

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS risk_settings (
    id                    INTEGER PRIMARY KEY,
    peak_value           REAL    NOT NULL DEFAULT 1000.0,
    real_peak_value      REAL    NOT NULL DEFAULT 0.0,
    real_portfolio_value REAL    NOT NULL DEFAULT 0.0,
    virtual_peak_value   REAL    NOT NULL DEFAULT 1000.0,
    virtual_portfolio_value REAL NOT NULL DEFAULT 1000.0,
    virtual_wallet_balance  REAL NOT NULL DEFAULT 1000.0,
    trading_mode         TEXT    NOT NULL DEFAULT 'virtual',
    max_drawdown_pct     REAL    NOT NULL DEFAULT 20.0,
    drawdown_reduce_pct  REAL    NOT NULL DEFAULT 5.0,
    drawdown_pause_pct   REAL    NOT NULL DEFAULT 10.0,
    max_positions        INTEGER NOT NULL DEFAULT 8,
    max_narrative_pct    REAL    NOT NULL DEFAULT 30.0,
    default_stop_pct     REAL    NOT NULL DEFAULT 20.0,
    default_position_pct REAL    NOT NULL DEFAULT 2.0,
    updated_at           TEXT    NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS trades (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    address                 TEXT    NOT NULL,
    symbol                  TEXT    NOT NULL DEFAULT '',
    name                    TEXT    NOT NULL DEFAULT '',
    status                  TEXT    NOT NULL DEFAULT 'watching',
    trade_type              TEXT    NOT NULL DEFAULT 'virtual',
    entry_price             REAL,
    entry_date              TEXT,
    position_size           REAL,
    exit_price              REAL,
    exit_date               TEXT,
    notes                   TEXT    NOT NULL DEFAULT '',
    stop_loss_pct           REAL,
    stop_price              REAL,
    trailing_stop           INTEGER NOT NULL DEFAULT 0,
    peak_price              REAL,
    stop_loss_enabled       INTEGER NOT NULL DEFAULT 1,
    take_profit_enabled     INTEGER NOT NULL DEFAULT 0,
    take_profit_multiplier  REAL,
    peak_decay_enabled      INTEGER NOT NULL DEFAULT 0,
    peak_decay_pct          REAL,
    volume_exhaustion_enabled INTEGER NOT NULL DEFAULT 0,
    volume_exhaustion_pct   REAL,
    peak_volume_24h         REAL,
    close_reason            TEXT,
    tx_hash                 TEXT    NOT NULL DEFAULT '',
    narrative               TEXT    NOT NULL DEFAULT '',
    pump_graduated          INTEGER NOT NULL DEFAULT 0,
    created_at              TEXT    NOT NULL,
    updated_at              TEXT    NOT NULL
);
"#;
