use anyhow::Result;
use sqlx::SqlitePool;

use crate::models::research_entry::ResearchEntry;

pub async fn create(pool: &SqlitePool, entry: &ResearchEntry) -> Result<ResearchEntry> {
    let id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO research_entries
            (address, symbol, name, notes, conviction, safety_score, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(&entry.address)
    .bind(&entry.symbol)
    .bind(&entry.name)
    .bind(&entry.notes)
    .bind(entry.conviction)
    .bind(entry.safety_score)
    .bind(entry.created_at)
    .bind(entry.updated_at)
    .fetch_one(pool)
    .await?;

    Ok(ResearchEntry {
        id,
        ..entry.clone()
    })
}

pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<ResearchEntry>> {
    let row = sqlx::query_as::<_, ResearchEntry>("SELECT * FROM research_entries WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_all(pool: &SqlitePool) -> Result<Vec<ResearchEntry>> {
    let rows = sqlx::query_as::<_, ResearchEntry>(
        "SELECT * FROM research_entries ORDER BY updated_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_all_addresses(pool: &SqlitePool) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>("SELECT address FROM research_entries")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn delete_by_id(pool: &SqlitePool, id: i64) -> Result<bool> {
    let affected = sqlx::query("DELETE FROM research_entries WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected > 0)
}
