mod util;

use chrono::NaiveDateTime;
use coins_database::CexListing;
use coins_database::queries::cex_listing;
use util::setup_memory_pool;

fn sample_listing(exchange: &str, ext_id: &str) -> CexListing {
    CexListing {
        id: 0,
        exchange: exchange.into(),
        external_id: ext_id.into(),
        token_name: "Test".into(),
        token_symbol: "TST".into(),
        listing_url: "".into(),
        announced_at: None,
        detected_at: NaiveDateTime::default(),
    }
}

#[tokio::test]
async fn bulk_create_and_list() {
    let pool = setup_memory_pool().await;
    let listings = vec![
        sample_listing("binance", "1"),
        sample_listing("coinbase", "2"),
    ];
    cex_listing::bulk_create(&pool, &listings).await.unwrap();

    let ids = cex_listing::list_external_ids(&pool).await.unwrap();
    assert_eq!(ids.len(), 2);
}

#[tokio::test]
async fn bulk_create_ignores_duplicates() {
    let pool = setup_memory_pool().await;
    let l = sample_listing("binance", "dup");
    cex_listing::bulk_create(&pool, &[l.clone()]).await.unwrap();
    cex_listing::bulk_create(&pool, &[l]).await.unwrap();

    assert_eq!(
        cex_listing::list_external_ids(&pool).await.unwrap().len(),
        1
    );
}

#[tokio::test]
async fn list_symbols_returns_distinct() {
    let pool = setup_memory_pool().await;
    let mut l1 = sample_listing("binance", "a");
    l1.token_symbol = "BTC".into();
    let mut l2 = sample_listing("coinbase", "b");
    l2.token_symbol = "BTC".into();
    cex_listing::bulk_create(&pool, &[l1, l2]).await.unwrap();

    let symbols = cex_listing::list_symbols(&pool).await.unwrap();
    assert_eq!(symbols, vec!["BTC"]);
}

#[tokio::test]
async fn filter_by_symbol_matches() {
    let pool = setup_memory_pool().await;
    let mut l1 = sample_listing("binance", "a");
    l1.token_symbol = "BTC".into();
    let mut l2 = sample_listing("coinbase", "b");
    l2.token_symbol = "ETH".into();
    cex_listing::bulk_create(&pool, &[l1, l2]).await.unwrap();

    let results = cex_listing::filter_by_symbol(&pool, "BTC").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].external_id, "a");
}
