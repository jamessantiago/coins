use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::SqlitePool;

use crate::models::telegram::{TelegramChannel, TelegramMessage};

// ---------------------------------------------------------------------------
// TelegramChannel
// ---------------------------------------------------------------------------

pub async fn get_or_create_channel(pool: &SqlitePool, username: &str) -> Result<(TelegramChannel, bool)> {
    let existing = sqlx::query_as::<_, TelegramChannel>(
        "SELECT * FROM telegram_channels WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    if let Some(ch) = existing {
        return Ok((ch, false));
    }

    let id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO telegram_channels (username, enabled, chat_id, added_at)
        VALUES ($1, 1, NULL, $2)
        RETURNING id
        "#,
    )
    .bind(username)
    .bind(NaiveDateTime::default())
    .fetch_one(pool)
    .await?;

    let ch = TelegramChannel {
        id,
        username: username.to_string(),
        enabled: true,
        chat_id: None,
        added_at: NaiveDateTime::default(),
    };
    Ok((ch, true))
}

pub async fn list_enabled_channels(pool: &SqlitePool) -> Result<Vec<TelegramChannel>> {
    let rows = sqlx::query_as::<_, TelegramChannel>(
        "SELECT * FROM telegram_channels WHERE enabled = 1 ORDER BY username",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_channel_by_id(pool: &SqlitePool, id: i64) -> Result<Option<TelegramChannel>> {
    let row = sqlx::query_as::<_, TelegramChannel>(
        "SELECT * FROM telegram_channels WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_all_channels(pool: &SqlitePool) -> Result<Vec<TelegramChannel>> {
    let rows = sqlx::query_as::<_, TelegramChannel>(
        "SELECT * FROM telegram_channels ORDER BY username",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn toggle_channel_enabled(pool: &SqlitePool, id: i64) -> Result<bool> {
    let affected = sqlx::query(
        r#"
        UPDATE telegram_channels
        SET enabled = CASE WHEN enabled = 1 THEN 0 ELSE 1 END
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(affected > 0)
}

pub async fn delete_channel(pool: &SqlitePool, id: i64) -> Result<bool> {
    let affected = sqlx::query("DELETE FROM telegram_channels WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected > 0)
}

pub async fn channel_count(pool: &SqlitePool) -> Result<i64> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM telegram_channels")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

// ---------------------------------------------------------------------------
// TelegramMessage
// ---------------------------------------------------------------------------

pub async fn create_message(pool: &SqlitePool, msg: &TelegramMessage) -> Result<TelegramMessage> {
    let id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO telegram_messages
            (channel_id, message_id, text, extracted_addresses, posted_at, detected_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(msg.channel_id)
    .bind(msg.message_id)
    .bind(&msg.text)
    .bind(&msg.extracted_addresses)
    .bind(msg.posted_at)
    .bind(msg.detected_at)
    .fetch_one(pool)
    .await?;

    Ok(TelegramMessage { id, ..msg.clone() })
}

pub async fn list_seen_message_ids(pool: &SqlitePool, channel_id: i64) -> Result<Vec<i32>> {
    let rows = sqlx::query_scalar::<_, i32>(
        "SELECT message_id FROM telegram_messages WHERE channel_id = $1",
    )
    .bind(channel_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_message_by_id(pool: &SqlitePool, id: i64) -> Result<Option<TelegramMessage>> {
    let row = sqlx::query_as::<_, TelegramMessage>(
        "SELECT * FROM telegram_messages WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn delete_message(pool: &SqlitePool, id: i64) -> Result<bool> {
    let affected = sqlx::query("DELETE FROM telegram_messages WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected > 0)
}

pub async fn list_recent_messages(pool: &SqlitePool, limit: usize) -> Result<Vec<TelegramMessage>> {
    let rows = sqlx::query_as::<_, TelegramMessage>(
        "SELECT * FROM telegram_messages ORDER BY detected_at DESC LIMIT $1",
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn message_count(pool: &SqlitePool) -> Result<i64> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM telegram_messages")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn iterate_messages_with_addresses(pool: &SqlitePool) -> Result<Vec<TelegramMessage>> {
    let rows = sqlx::query_as::<_, TelegramMessage>(
        "SELECT * FROM telegram_messages WHERE extracted_addresses != ''",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
