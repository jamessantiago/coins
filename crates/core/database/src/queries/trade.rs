use anyhow::Result;
use sqlx::QueryBuilder;
use sqlx::SqlitePool;

use crate::models::trade::{Trade, TradeStatus};

pub async fn create(pool: &SqlitePool, trade: &Trade) -> Result<Trade> {
    let id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO trades (
            address, symbol, name, status, trade_type,
            entry_price, entry_date, position_size,
            exit_price, exit_date, notes,
            stop_loss_pct, stop_price, trailing_stop,
            peak_price, stop_loss_enabled, take_profit_enabled,
            take_profit_multiplier, peak_decay_enabled, peak_decay_pct,
            volume_exhaustion_enabled, volume_exhaustion_pct, peak_volume_24h,
            close_reason, tx_hash, narrative, pump_graduated,
            created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8,
            $9, $10, $11,
            $12, $13, $14,
            $15, $16, $17,
            $18, $19, $20,
            $21, $22, $23,
            $24, $25, $26, $27,
            $28, $29
        )
        RETURNING id
        "#,
    )
    .bind(&trade.address)
    .bind(&trade.symbol)
    .bind(&trade.name)
    .bind(&trade.status)
    .bind(&trade.trade_type)
    .bind(trade.entry_price)
    .bind(trade.entry_date)
    .bind(trade.position_size)
    .bind(trade.exit_price)
    .bind(trade.exit_date)
    .bind(&trade.notes)
    .bind(trade.stop_loss_pct)
    .bind(trade.stop_price)
    .bind(trade.trailing_stop)
    .bind(trade.peak_price)
    .bind(trade.stop_loss_enabled)
    .bind(trade.take_profit_enabled)
    .bind(trade.take_profit_multiplier)
    .bind(trade.peak_decay_enabled)
    .bind(trade.peak_decay_pct)
    .bind(trade.volume_exhaustion_enabled)
    .bind(trade.volume_exhaustion_pct)
    .bind(trade.peak_volume_24h)
    .bind(&trade.close_reason)
    .bind(&trade.tx_hash)
    .bind(&trade.narrative)
    .bind(trade.pump_graduated)
    .bind(trade.created_at)
    .bind(trade.updated_at)
    .fetch_one(pool)
    .await?;

    Ok(Trade {
        id,
        ..trade.clone()
    })
}

pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Trade>> {
    let row = sqlx::query_as::<_, Trade>("SELECT * FROM trades WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn update(pool: &SqlitePool, trade: &Trade) -> Result<bool> {
    let affected = sqlx::query(
        r#"
        UPDATE trades SET
            address = $1, symbol = $2, name = $3, status = $4, trade_type = $5,
            entry_price = $6, entry_date = $7, position_size = $8,
            exit_price = $9, exit_date = $10, notes = $11,
            stop_loss_pct = $12, stop_price = $13, trailing_stop = $14,
            peak_price = $15, stop_loss_enabled = $16, take_profit_enabled = $17,
            take_profit_multiplier = $18, peak_decay_enabled = $19, peak_decay_pct = $20,
            volume_exhaustion_enabled = $21, volume_exhaustion_pct = $22, peak_volume_24h = $23,
            close_reason = $24, tx_hash = $25, narrative = $26, pump_graduated = $27,
            updated_at = $28
        WHERE id = $29
        "#,
    )
    .bind(&trade.address)
    .bind(&trade.symbol)
    .bind(&trade.name)
    .bind(&trade.status)
    .bind(&trade.trade_type)
    .bind(trade.entry_price)
    .bind(trade.entry_date)
    .bind(trade.position_size)
    .bind(trade.exit_price)
    .bind(trade.exit_date)
    .bind(&trade.notes)
    .bind(trade.stop_loss_pct)
    .bind(trade.stop_price)
    .bind(trade.trailing_stop)
    .bind(trade.peak_price)
    .bind(trade.stop_loss_enabled)
    .bind(trade.take_profit_enabled)
    .bind(trade.take_profit_multiplier)
    .bind(trade.peak_decay_enabled)
    .bind(trade.peak_decay_pct)
    .bind(trade.volume_exhaustion_enabled)
    .bind(trade.volume_exhaustion_pct)
    .bind(trade.peak_volume_24h)
    .bind(&trade.close_reason)
    .bind(&trade.tx_hash)
    .bind(&trade.narrative)
    .bind(trade.pump_graduated)
    .bind(trade.updated_at)
    .bind(trade.id)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(affected > 0)
}

pub async fn list_open(pool: &SqlitePool) -> Result<Vec<Trade>> {
    let status1 = &TradeStatus::Bought;
    let status2 = &TradeStatus::VirtualBought;
    let rows = sqlx::query_as::<_, Trade>(
        "SELECT * FROM trades WHERE status = $1 OR status = $2 ORDER BY updated_at DESC",
    )
    .bind(status1)
    .bind(status2)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_open_by_type(pool: &SqlitePool, trade_type: &str) -> Result<Vec<Trade>> {
    let rows = sqlx::query_as::<_, Trade>(
        r#"
        SELECT * FROM trades
        WHERE trade_type = $1 AND (status = 'bought' OR status = 'virtual_bought')
          AND position_size IS NOT NULL
        ORDER BY updated_at DESC
        "#,
    )
    .bind(trade_type)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn open_position_value(pool: &SqlitePool, trade_type: &str) -> Result<f64> {
    let total: Option<f64> = sqlx::query_scalar(
        r#"
        SELECT SUM(position_size) FROM trades
        WHERE trade_type = $1 AND (status = 'bought' OR status = 'virtual_bought')
          AND position_size IS NOT NULL
        "#,
    )
    .bind(trade_type)
    .fetch_one(pool)
    .await?;
    Ok(total.unwrap_or(0.0))
}

pub async fn list_by_statuses(pool: &SqlitePool, statuses: &[TradeStatus]) -> Result<Vec<Trade>> {
    if statuses.is_empty() {
        return Ok(vec![]);
    }
    let mut qb = QueryBuilder::new("SELECT * FROM trades WHERE status IN (");
    let mut sep = qb.separated(", ");
    for s in statuses {
        sep.push_bind(s);
    }
    qb.push(") ORDER BY updated_at DESC");
    let rows = qb.build_query_as::<Trade>().fetch_all(pool).await?;
    Ok(rows)
}

pub async fn count_by_status(pool: &SqlitePool, status: &TradeStatus) -> Result<i32> {
    let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM trades WHERE status = $1")
        .bind(status)
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn list_open_addresses(pool: &SqlitePool) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT address FROM trades
        WHERE status = 'bought' OR status = 'virtual_bought'
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_pnl_trades(pool: &SqlitePool, trade_type: &str) -> Result<Vec<Trade>> {
    let rows = sqlx::query_as::<_, Trade>(
        r#"
        SELECT * FROM trades
        WHERE trade_type = $1
          AND (status = 'sold' OR status = 'virtual_sold')
          AND entry_price IS NOT NULL
          AND exit_price IS NOT NULL
          AND position_size IS NOT NULL
        ORDER BY updated_at DESC
        "#,
    )
    .bind(trade_type)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_ungraduated_open(pool: &SqlitePool) -> Result<Vec<Trade>> {
    let rows = sqlx::query_as::<_, Trade>(
        r#"
        SELECT * FROM trades
        WHERE (status = 'bought' OR status = 'virtual_bought')
          AND pump_graduated = 0
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn count_open_real(pool: &SqlitePool) -> Result<i32> {
    let count: i32 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM trades
        WHERE trade_type = 'real' AND status = 'bought'
        "#,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Trade>> {
    let rows = sqlx::query_as::<_, Trade>("SELECT * FROM trades ORDER BY updated_at DESC")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn update_peak_price(pool: &SqlitePool, id: i64, peak_price: f64) -> Result<bool> {
    let now = chrono::Utc::now().naive_utc();
    let affected = sqlx::query("UPDATE trades SET peak_price = $1, updated_at = $2 WHERE id = $3")
        .bind(peak_price)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected > 0)
}

pub async fn update_peak_volume(pool: &SqlitePool, id: i64, volume: f64) -> Result<bool> {
    let now = chrono::Utc::now().naive_utc();
    let affected =
        sqlx::query("UPDATE trades SET peak_volume_24h = $1, updated_at = $2 WHERE id = $3")
            .bind(volume)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();
    Ok(affected > 0)
}

pub async fn delete_by_id(pool: &SqlitePool, id: i64) -> Result<bool> {
    let affected = sqlx::query("DELETE FROM trades WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected > 0)
}
