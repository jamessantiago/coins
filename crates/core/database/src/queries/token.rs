use anyhow::Result;
use sqlx::SqlitePool;

use crate::models::token::Token;

pub async fn bulk_create(pool: &SqlitePool, tokens: &[Token]) -> Result<()> {
    let mut tx = pool.begin().await?;
    for t in tokens {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO tokens (address, symbol, name, chain_id, first_seen, last_seen)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&t.address)
        .bind(&t.symbol)
        .bind(&t.name)
        .bind(&t.chain_id)
        .bind(t.first_seen)
        .bind(t.last_seen)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn list_all_addresses(pool: &SqlitePool) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>("SELECT address FROM tokens")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn exists_by_address(pool: &SqlitePool, address: &str) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tokens WHERE address = $1",
    )
    .bind(address)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}
