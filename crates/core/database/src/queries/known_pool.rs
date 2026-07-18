use anyhow::Result;
use sqlx::SqlitePool;

use crate::models::known_pool::KnownPool;

pub async fn bulk_create(pool: &SqlitePool, pools: &[KnownPool]) -> Result<()> {
    let mut tx = pool.begin().await?;
    for p in pools {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO known_pools
                (pool_address, base_mint, quote_mint, symbol, name, first_seen, last_seen)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&p.pool_address)
        .bind(&p.base_mint)
        .bind(&p.quote_mint)
        .bind(&p.symbol)
        .bind(&p.name)
        .bind(p.first_seen)
        .bind(p.last_seen)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn list_pool_addresses(pool: &SqlitePool) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>("SELECT pool_address FROM known_pools")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn list_base_mints(pool: &SqlitePool) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>("SELECT DISTINCT base_mint FROM known_pools")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn exists_by_address(pool: &SqlitePool, address: &str) -> Result<bool> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM known_pools WHERE pool_address = $1")
        .bind(address)
        .fetch_one(pool)
        .await?;
    Ok(count > 0)
}

pub async fn count(pool: &SqlitePool) -> Result<i64> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM known_pools")
        .fetch_one(pool)
        .await?;
    Ok(count)
}
