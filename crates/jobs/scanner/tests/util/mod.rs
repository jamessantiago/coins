use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

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

CREATE TABLE IF NOT EXISTS tokens (
    address    TEXT NOT NULL PRIMARY KEY,
    symbol     TEXT NOT NULL,
    name       TEXT NOT NULL,
    chain_id   TEXT NOT NULL DEFAULT 'solana',
    first_seen TEXT NOT NULL,
    last_seen  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cluster_counts (
    cluster TEXT NOT NULL,
    bucket  TEXT NOT NULL,
    count   INTEGER NOT NULL DEFAULT 0,
    UNIQUE(cluster, bucket)
);

CREATE INDEX IF NOT EXISTS idx_cluster_counts_lookup
    ON cluster_counts(cluster, bucket);

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

CREATE TABLE IF NOT EXISTS research_entries (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    address       TEXT NOT NULL,
    symbol        TEXT NOT NULL DEFAULT '',
    name          TEXT NOT NULL DEFAULT '',
    notes         TEXT NOT NULL DEFAULT '',
    conviction    INTEGER NOT NULL DEFAULT 3,
    safety_score  REAL,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

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

CREATE TABLE IF NOT EXISTS known_pools (
    pool_address TEXT NOT NULL PRIMARY KEY,
    base_mint    TEXT NOT NULL,
    quote_mint   TEXT NOT NULL,
    symbol       TEXT NOT NULL DEFAULT '',
    name         TEXT NOT NULL DEFAULT '',
    first_seen   TEXT NOT NULL,
    last_seen    TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_known_pools_base_mint
    ON known_pools(base_mint);

CREATE TABLE IF NOT EXISTS pump_bonding_curves (
    bonding_curve TEXT NOT NULL PRIMARY KEY,
    mint          TEXT NOT NULL,
    first_seen    TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pump_bonding_curves_mint
    ON pump_bonding_curves(mint);

CREATE TABLE IF NOT EXISTS distilled_tokens (
    address              TEXT    NOT NULL PRIMARY KEY,
    symbol               TEXT    NOT NULL DEFAULT '',
    name                 TEXT    NOT NULL DEFAULT '',
    first_seen           TEXT    NOT NULL,
    last_seen            TEXT    NOT NULL,
    sources              TEXT    NOT NULL DEFAULT '',
    safety_score         REAL,
    liquidity_usd        REAL,
    volume_24h           REAL,
    fdv                  REAL,
    narrative_clusters   TEXT    NOT NULL DEFAULT '',
    telegram_mentions    INTEGER NOT NULL DEFAULT 0,
    cex_listed           INTEGER NOT NULL DEFAULT 0,
    research_conviction  INTEGER,
    dexscreener_url      TEXT    NOT NULL DEFAULT '',
    ranking_score        REAL    NOT NULL DEFAULT 0.0,
    price_change_24h     REAL,
    price_change_1h      REAL,
    vol_liq_ratio        REAL,
    buy_sell_ratio       REAL,
    updated_at           TEXT    NOT NULL
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

pub fn pubkey_bytes(seed: u8) -> [u8; 32] {
    let mut b = [0u8; 32];
    b[0] = seed;
    b
}

pub fn pubkey_str(seed: u8) -> String {
    bs58::encode(pubkey_bytes(seed)).into_string()
}
