mod util;

use chrono::NaiveDateTime;
use coins_database::queries::token;
use coins_database::Token;
use util::setup_memory_pool;

#[tokio::test]
async fn bulk_create_inserts_tokens() {
    let pool = setup_memory_pool().await;
    let now = NaiveDateTime::default();
    let tokens = vec![
        Token { address: "addr1".into(), symbol: "BTC".into(), name: "Bitcoin".into(), chain_id: "solana".into(), first_seen: now, last_seen: now },
        Token { address: "addr2".into(), symbol: "ETH".into(), name: "Ethereum".into(), chain_id: "solana".into(), first_seen: now, last_seen: now },
    ];
    token::bulk_create(&pool, &tokens).await.unwrap();

    let addresses = token::list_all_addresses(&pool).await.unwrap();
    assert_eq!(addresses.len(), 2);
    assert!(addresses.contains(&"addr1".to_string()));
}

#[tokio::test]
async fn bulk_create_ignores_duplicates() {
    let pool = setup_memory_pool().await;
    let now = NaiveDateTime::default();
    let t = Token { address: "dup".into(), symbol: "X".into(), name: "X".into(), chain_id: "solana".into(), first_seen: now, last_seen: now };
    token::bulk_create(&pool, &[t.clone()]).await.unwrap();
    token::bulk_create(&pool, &[t]).await.unwrap();

    assert_eq!(token::list_all_addresses(&pool).await.unwrap().len(), 1);
}

#[tokio::test]
async fn exists_by_address_returns_true_for_existing() {
    let pool = setup_memory_pool().await;
    let now = NaiveDateTime::default();
    let t = Token { address: "exists".into(), symbol: "X".into(), name: "X".into(), chain_id: "solana".into(), first_seen: now, last_seen: now };
    token::bulk_create(&pool, &[t]).await.unwrap();

    assert!(token::exists_by_address(&pool, "exists").await.unwrap());
    assert!(!token::exists_by_address(&pool, "missing").await.unwrap());
}
