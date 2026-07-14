mod util;

use chrono::NaiveDateTime;
use coins_database::queries::known_pool;
use coins_database::KnownPool;
use util::setup_memory_pool;

fn sample_pool(address: &str, mint: &str) -> KnownPool {
    KnownPool {
        pool_address: address.into(),
        base_mint: mint.into(),
        quote_mint: "USDC".into(),
        symbol: "".into(),
        name: "".into(),
        first_seen: NaiveDateTime::default(),
        last_seen: NaiveDateTime::default(),
    }
}

#[tokio::test]
async fn bulk_create_and_list() {
    let pool = setup_memory_pool().await;
    let pools = vec![sample_pool("pool1", "mint_a"), sample_pool("pool2", "mint_b")];
    known_pool::bulk_create(&pool, &pools).await.unwrap();

    let addresses = known_pool::list_pool_addresses(&pool).await.unwrap();
    assert_eq!(addresses.len(), 2);

    let mints = known_pool::list_base_mints(&pool).await.unwrap();
    assert_eq!(mints.len(), 2);
}

#[tokio::test]
async fn bulk_create_ignores_duplicates() {
    let pool = setup_memory_pool().await;
    let p = sample_pool("dup", "mint");
    known_pool::bulk_create(&pool, &[p.clone()]).await.unwrap();
    known_pool::bulk_create(&pool, &[p]).await.unwrap();
    assert_eq!(known_pool::list_pool_addresses(&pool).await.unwrap().len(), 1);
}

#[tokio::test]
async fn exists_by_address() {
    let pool = setup_memory_pool().await;
    known_pool::bulk_create(&pool, &[sample_pool("exists", "m")]).await.unwrap();
    assert!(known_pool::exists_by_address(&pool, "exists").await.unwrap());
    assert!(!known_pool::exists_by_address(&pool, "missing").await.unwrap());
    assert_eq!(known_pool::count(&pool).await.unwrap(), 1);
}
