mod util;

use chrono::{TimeDelta, Utc};
use coins_database::queries::sse_event;
use util::setup_memory_pool;

#[tokio::test]
async fn create_and_read_since() {
    let pool = setup_memory_pool().await;
    let e1 = sse_event::create(&pool, "trade", r#"{"price": 100}"#)
        .await
        .unwrap();
    sse_event::create(&pool, "trade", r#"{"price": 200}"#)
        .await
        .unwrap();

    let events = sse_event::read_since(&pool, e1.id).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, r#"{"price": 200}"#);
}

#[tokio::test]
async fn exists_by_event() {
    let pool = setup_memory_pool().await;
    sse_event::create(&pool, "test_event", "{}").await.unwrap();
    assert!(
        sse_event::exists_by_event(&pool, "test_event")
            .await
            .unwrap()
    );
    assert!(!sse_event::exists_by_event(&pool, "missing").await.unwrap());
}

#[tokio::test]
async fn prune_removes_old_events() {
    let pool = setup_memory_pool().await;
    sse_event::create(&pool, "old", "{}").await.unwrap();
    sse_event::create(&pool, "new", "{}").await.unwrap();

    let cutoff = Utc::now().naive_utc() + TimeDelta::try_seconds(1).unwrap();
    let removed = sse_event::prune(&pool, cutoff).await.unwrap();
    assert!(removed > 0, "expected >0 removed, got {removed}");
    assert_eq!(sse_event::read_since(&pool, 0).await.unwrap().len(), 0);
}

#[tokio::test]
async fn delete_all_clears() {
    let pool = setup_memory_pool().await;
    sse_event::create(&pool, "e1", "{}").await.unwrap();
    sse_event::create(&pool, "e2", "{}").await.unwrap();

    assert!(sse_event::delete_all(&pool).await.unwrap() > 0);
    assert_eq!(sse_event::read_since(&pool, 0).await.unwrap().len(), 0);
}
