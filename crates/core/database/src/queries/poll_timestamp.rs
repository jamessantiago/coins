use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::SqlitePool;

use crate::models::poll_timestamp::PollTimestamp;

pub async fn upsert(
    pool: &SqlitePool,
    service: &str,
    last_run_at: NaiveDateTime,
    listings_found: i32,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO poll_timestamps (service, last_run_at, listings_found)
        VALUES ($1, $2, $3)
        ON CONFLICT(service) DO UPDATE SET
            last_run_at    = excluded.last_run_at,
            listings_found = excluded.listings_found
        "#,
    )
    .bind(service)
    .bind(last_run_at)
    .bind(listings_found)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_by_service(pool: &SqlitePool, service: &str) -> Result<Option<PollTimestamp>> {
    let row =
        sqlx::query_as::<_, PollTimestamp>("SELECT * FROM poll_timestamps WHERE service = $1")
            .bind(service)
            .fetch_optional(pool)
            .await?;
    Ok(row)
}

pub async fn list_all(pool: &SqlitePool) -> Result<Vec<PollTimestamp>> {
    let rows = sqlx::query_as::<_, PollTimestamp>("SELECT * FROM poll_timestamps")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}
