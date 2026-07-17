use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use sqlx::SqlitePool;

use crate::models::sse_event::SseEvent;

pub async fn create(pool: &SqlitePool, event: &str, data: &str) -> Result<SseEvent> {
    let now = Utc::now().naive_utc();
    let id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO sse_events (event, data, created_at)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(event)
    .bind(data)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(SseEvent {
        id,
        event: event.to_string(),
        data: data.to_string(),
        created_at: now,
    })
}

pub async fn read_since(pool: &SqlitePool, last_id: i64) -> Result<Vec<SseEvent>> {
    let rows = sqlx::query_as::<_, SseEvent>(
        "SELECT * FROM sse_events WHERE id > $1 ORDER BY id",
    )
    .bind(last_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn prune(pool: &SqlitePool, cutoff: NaiveDateTime) -> Result<u64> {
    let affected = sqlx::query("DELETE FROM sse_events WHERE created_at < $1")
        .bind(cutoff)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected)
}

pub async fn exists_by_event(pool: &SqlitePool, event: &str) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sse_events WHERE event = $1",
    )
    .bind(event)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub async fn delete_all(pool: &SqlitePool) -> Result<u64> {
    let affected = sqlx::query("DELETE FROM sse_events")
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected)
}
