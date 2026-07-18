use anyhow::Result;
use sqlx::QueryBuilder;
use sqlx::SqlitePool;

use crate::models::cluster_count::ClusterCount;

pub async fn upsert(pool: &SqlitePool, cluster: &str, bucket: &str, count: i32) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO cluster_counts (cluster, bucket, count)
        VALUES ($1, $2, $3)
        ON CONFLICT(cluster, bucket) DO UPDATE SET count = excluded.count
        "#,
    )
    .bind(cluster)
    .bind(bucket)
    .bind(count)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_baseline(
    pool: &SqlitePool,
    cluster: &str,
    bucket: &str,
    windows: usize,
) -> Result<Vec<i32>> {
    let rows = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT count FROM cluster_counts
        WHERE cluster = $1 AND bucket < $2
        ORDER BY bucket DESC
        LIMIT $3
        "#,
    )
    .bind(cluster)
    .bind(bucket)
    .bind(windows as i64)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_by_bucket(pool: &SqlitePool, bucket: &str) -> Result<Vec<ClusterCount>> {
    let rows = sqlx::query_as::<_, ClusterCount>(
        r#"
        SELECT cluster, bucket, count
        FROM cluster_counts
        WHERE bucket = $1
        ORDER BY count DESC
        "#,
    )
    .bind(bucket)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(Debug, sqlx::FromRow)]
pub struct NarrativeTotal {
    pub cluster: String,
    pub total: i64,
}

pub async fn get_narrative_totals(pool: &SqlitePool) -> Result<Vec<NarrativeTotal>> {
    let rows = sqlx::query_as::<_, NarrativeTotal>(
        r#"
        SELECT cluster, SUM(count) as total
        FROM cluster_counts
        GROUP BY cluster
        ORDER BY total DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_distinct_buckets(pool: &SqlitePool, limit: usize) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT bucket FROM cluster_counts
        ORDER BY bucket DESC
        LIMIT $1
        "#,
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_by_clusters_and_buckets(
    pool: &SqlitePool,
    buckets: &[String],
) -> Result<Vec<ClusterCount>> {
    if buckets.is_empty() {
        return Ok(vec![]);
    }
    let mut qb =
        QueryBuilder::new("SELECT cluster, bucket, count FROM cluster_counts WHERE bucket IN (");
    let mut sep = qb.separated(", ");
    for b in buckets {
        sep.push_bind(b);
    }
    qb.push(") ORDER BY cluster, bucket");
    let rows = qb.build_query_as::<ClusterCount>().fetch_all(pool).await?;
    Ok(rows)
}
