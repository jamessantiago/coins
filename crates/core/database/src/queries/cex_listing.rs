use anyhow::Result;
use sqlx::SqlitePool;

use crate::models::cex_listing::CexListing;

pub async fn bulk_create(pool: &SqlitePool, listings: &[CexListing]) -> Result<()> {
    let mut tx = pool.begin().await?;
    for l in listings {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO cex_listings
                (exchange, external_id, token_name, token_symbol, listing_url, announced_at, detected_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&l.exchange)
        .bind(&l.external_id)
        .bind(&l.token_name)
        .bind(&l.token_symbol)
        .bind(&l.listing_url)
        .bind(l.announced_at)
        .bind(l.detected_at)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn list_external_ids(pool: &SqlitePool) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>("SELECT external_id FROM cex_listings")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn list_symbols(pool: &SqlitePool) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>("SELECT DISTINCT token_symbol FROM cex_listings")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn list_recent(pool: &SqlitePool, limit: usize) -> Result<Vec<CexListing>> {
    let rows = sqlx::query_as::<_, CexListing>(
        "SELECT * FROM cex_listings ORDER BY detected_at DESC LIMIT $1",
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_by_exchange(
    pool: &SqlitePool,
    exchange: &str,
    limit: usize,
) -> Result<Vec<CexListing>> {
    let rows = sqlx::query_as::<_, CexListing>(
        "SELECT * FROM cex_listings WHERE exchange = $1 ORDER BY detected_at DESC LIMIT $2",
    )
    .bind(exchange)
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn filter_by_symbol(pool: &SqlitePool, symbol: &str) -> Result<Vec<CexListing>> {
    let rows = sqlx::query_as::<_, CexListing>(
        "SELECT * FROM cex_listings WHERE token_symbol = $1 ORDER BY detected_at DESC",
    )
    .bind(symbol)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_all(pool: &SqlitePool) -> Result<Vec<CexListing>> {
    let rows =
        sqlx::query_as::<_, CexListing>("SELECT * FROM cex_listings ORDER BY detected_at DESC")
            .fetch_all(pool)
            .await?;
    Ok(rows)
}
