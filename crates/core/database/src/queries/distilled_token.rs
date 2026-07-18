use anyhow::Result;
use sqlx::QueryBuilder;
use sqlx::SqlitePool;

use crate::models::distilled_token::DistilledToken;

pub async fn upsert(pool: &SqlitePool, token: &DistilledToken) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO distilled_tokens (
            address, symbol, name, first_seen, last_seen, sources,
            safety_score, liquidity_usd, volume_24h, fdv,
            narrative_clusters, telegram_mentions, cex_listed,
            research_conviction, dexscreener_url, ranking_score,
            price_change_24h, price_change_1h, vol_liq_ratio, buy_sell_ratio,
            updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                  $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                  $21)
        ON CONFLICT(address) DO UPDATE SET
            symbol               = excluded.symbol,
            name                 = excluded.name,
            last_seen            = excluded.last_seen,
            sources              = excluded.sources,
            safety_score         = excluded.safety_score,
            liquidity_usd        = excluded.liquidity_usd,
            volume_24h           = excluded.volume_24h,
            fdv                  = excluded.fdv,
            narrative_clusters   = excluded.narrative_clusters,
            telegram_mentions    = excluded.telegram_mentions,
            cex_listed           = excluded.cex_listed,
            research_conviction  = excluded.research_conviction,
            dexscreener_url      = excluded.dexscreener_url,
            ranking_score        = excluded.ranking_score,
            price_change_24h     = excluded.price_change_24h,
            price_change_1h      = excluded.price_change_1h,
            vol_liq_ratio        = excluded.vol_liq_ratio,
            buy_sell_ratio       = excluded.buy_sell_ratio,
            updated_at           = excluded.updated_at
        "#,
    )
    .bind(&token.address)
    .bind(&token.symbol)
    .bind(&token.name)
    .bind(token.first_seen)
    .bind(token.last_seen)
    .bind(&token.sources)
    .bind(token.safety_score)
    .bind(token.liquidity_usd)
    .bind(token.volume_24h)
    .bind(token.fdv)
    .bind(&token.narrative_clusters)
    .bind(token.telegram_mentions)
    .bind(token.cex_listed)
    .bind(token.research_conviction)
    .bind(&token.dexscreener_url)
    .bind(token.ranking_score)
    .bind(token.price_change_24h)
    .bind(token.price_change_1h)
    .bind(token.vol_liq_ratio)
    .bind(token.buy_sell_ratio)
    .bind(token.updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_by_address(pool: &SqlitePool, address: &str) -> Result<Option<DistilledToken>> {
    let row =
        sqlx::query_as::<_, DistilledToken>("SELECT * FROM distilled_tokens WHERE address = $1")
            .bind(address)
            .fetch_optional(pool)
            .await?;
    Ok(row)
}

pub async fn list_unscored(pool: &SqlitePool) -> Result<Vec<DistilledToken>> {
    let rows = sqlx::query_as::<_, DistilledToken>(
        "SELECT * FROM distilled_tokens WHERE ranking_score = 0",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn count(pool: &SqlitePool) -> Result<i64> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM distilled_tokens")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn list_narrative_clusters(pool: &SqlitePool) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT narrative_clusters FROM distilled_tokens WHERE narrative_clusters != ''",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_filtered_simple(
    pool: &SqlitePool,
    sort_by: &str,
    limit: usize,
) -> Result<Vec<DistilledToken>> {
    let order = match sort_by {
        "liquidity" => "liquidity_usd DESC NULLS LAST",
        "volume" => "volume_24h DESC NULLS LAST",
        "score" => "ranking_score DESC",
        "mentions" => "telegram_mentions DESC",
        _ => "ranking_score DESC",
    };
    let mut qb = QueryBuilder::new("SELECT * FROM distilled_tokens ORDER BY ");
    qb.push(order);
    qb.push(" LIMIT ");
    qb.push_bind(limit as i64);
    let rows = qb
        .build_query_as::<DistilledToken>()
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn update_ranking_score(pool: &SqlitePool, address: &str, score: f64) -> Result<bool> {
    let affected = sqlx::query("UPDATE distilled_tokens SET ranking_score = $1 WHERE address = $2")
        .bind(score)
        .bind(address)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected > 0)
}
