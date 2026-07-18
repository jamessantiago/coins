mod util;

use chrono::NaiveDateTime;
use coins_database::ResearchEntry;
use coins_database::queries::research_entry;
use util::setup_memory_pool;

fn sample(addr: &str) -> ResearchEntry {
    ResearchEntry {
        id: 0,
        address: addr.into(),
        symbol: "".into(),
        name: "".into(),
        notes: "".into(),
        conviction: 3,
        safety_score: None,
        created_at: NaiveDateTime::default(),
        updated_at: NaiveDateTime::default(),
    }
}

#[tokio::test]
async fn create_and_get_by_id() {
    let pool = setup_memory_pool().await;
    let entry = research_entry::create(&pool, &sample("addr1"))
        .await
        .unwrap();
    assert!(entry.id > 0);

    let fetched = research_entry::get_by_id(&pool, entry.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.address, "addr1");
    assert_eq!(fetched.conviction, 3);
}

#[tokio::test]
async fn delete_by_id_removes_row() {
    let pool = setup_memory_pool().await;
    let entry = research_entry::create(&pool, &sample("addr_del"))
        .await
        .unwrap();
    assert!(research_entry::delete_by_id(&pool, entry.id).await.unwrap());
    assert!(
        research_entry::get_by_id(&pool, entry.id)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn list_all_returns_all() {
    let pool = setup_memory_pool().await;
    research_entry::create(&pool, &sample("a")).await.unwrap();
    research_entry::create(&pool, &sample("b")).await.unwrap();

    let all = research_entry::list_all(&pool).await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn list_all_addresses_returns_addresses() {
    let pool = setup_memory_pool().await;
    research_entry::create(&pool, &sample("addr_x"))
        .await
        .unwrap();
    let addrs = research_entry::list_all_addresses(&pool).await.unwrap();
    assert!(addrs.contains(&"addr_x".to_string()));
}
