mod util;

use coins_database::queries::cluster_count;
use util::setup_memory_pool;

#[tokio::test]
async fn upsert_creates_and_updates() {
    let pool = setup_memory_pool().await;

    cluster_count::upsert(&pool, "defi", "2024-01", 10).await.unwrap();
    let rows = cluster_count::get_by_bucket(&pool, "2024-01").await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].count, 10);

    cluster_count::upsert(&pool, "defi", "2024-01", 20).await.unwrap();
    let rows = cluster_count::get_by_bucket(&pool, "2024-01").await.unwrap();
    assert_eq!(rows[0].count, 20);
}

#[tokio::test]
async fn get_narrative_totals_aggregates() {
    let pool = setup_memory_pool().await;
    cluster_count::upsert(&pool, "defi", "a", 5).await.unwrap();
    cluster_count::upsert(&pool, "defi", "b", 10).await.unwrap();
    cluster_count::upsert(&pool, "meme", "a", 3).await.unwrap();

    let totals = cluster_count::get_narrative_totals(&pool).await.unwrap();
    assert_eq!(totals.len(), 2);
    let defi = totals.iter().find(|t| t.cluster == "defi").unwrap();
    assert_eq!(defi.total, 15);
}

#[tokio::test]
async fn get_distinct_buckets_returns_ordered() {
    let pool = setup_memory_pool().await;
    cluster_count::upsert(&pool, "a", "z", 1).await.unwrap();
    cluster_count::upsert(&pool, "a", "y", 1).await.unwrap();
    cluster_count::upsert(&pool, "a", "x", 1).await.unwrap();

    let buckets = cluster_count::get_distinct_buckets(&pool, 10).await.unwrap();
    assert_eq!(buckets, vec!["z", "y", "x"]);
}

#[tokio::test]
async fn get_by_clusters_and_buckets_filters() {
    let pool = setup_memory_pool().await;
    cluster_count::upsert(&pool, "defi", "a", 1).await.unwrap();
    cluster_count::upsert(&pool, "meme", "a", 2).await.unwrap();
    cluster_count::upsert(&pool, "defi", "b", 3).await.unwrap();

    let rows = cluster_count::get_by_clusters_and_buckets(
        &pool, &["a".to_string()],
    )
    .await
    .unwrap();
    assert_eq!(rows.len(), 2);
}
