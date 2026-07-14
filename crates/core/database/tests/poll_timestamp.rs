mod util;

use chrono::NaiveDateTime;
use coins_database::queries::poll_timestamp;
use util::setup_memory_pool;

#[tokio::test]
async fn upsert_creates_and_updates() {
    let pool = setup_memory_pool().await;
    let now = NaiveDateTime::default();

    poll_timestamp::upsert(&pool, "scanner", now, 5).await.unwrap();

    let row = poll_timestamp::get_by_service(&pool, "scanner").await.unwrap().unwrap();
    assert_eq!(row.service, "scanner");
    assert_eq!(row.listings_found, 5);

    poll_timestamp::upsert(&pool, "scanner", now, 10).await.unwrap();
    let row = poll_timestamp::get_by_service(&pool, "scanner").await.unwrap().unwrap();
    assert_eq!(row.listings_found, 10);
}

#[tokio::test]
async fn get_by_service_returns_none_for_missing() {
    let pool = setup_memory_pool().await;
    let result = poll_timestamp::get_by_service(&pool, "nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn list_all_returns_all_services() {
    let pool = setup_memory_pool().await;
    let now = NaiveDateTime::default();
    poll_timestamp::upsert(&pool, "scanner", now, 1).await.unwrap();
    poll_timestamp::upsert(&pool, "distiller", now, 2).await.unwrap();

    let all = poll_timestamp::list_all(&pool).await.unwrap();
    assert_eq!(all.len(), 2);
}
