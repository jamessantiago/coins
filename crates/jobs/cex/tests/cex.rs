mod util;

use coins_config::Config;
use coins_database::queries::{cex_listing, poll_timestamp, sse_event};
use util::setup_memory_pool;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use chrono::Utc;
use serial_test::serial;

fn binance_tickers_response() -> serde_json::Value {
    serde_json::json!({
        "tickers": [
            {
                "coin_id": "bitcoin",
                "base": "BTC",
                "target": "Bitcoin",
                "trade_url": "https://www.binance.com/en/trade/BTC_USDT"
            },
            {
                "coin_id": "ethereum",
                "base": "ETH",
                "target": "Ethereum",
                "trade_url": "https://www.binance.com/en/trade/ETH_USDT"
            },
            {
                "coin_id": "solana",
                "base": "SOL",
                "target": "Solana",
                "trade_url": "https://www.binance.com/en/trade/SOL_USDT"
            }
        ]
    })
}

fn coinbase_assets_response() -> serde_json::Value {
    serde_json::json!({
        "data": {
            "bitcoin": {
                "id": "bitcoin",
                "name": "Bitcoin",
                "symbol": "BTC"
            },
            "ethereum": {
                "id": "ethereum",
                "name": "Ethereum",
                "symbol": "ETH"
            }
        }
    })
}

fn binance_empty_response() -> serde_json::Value {
    serde_json::json!({ "tickers": [] })
}

fn coinbase_assets_array_response() -> serde_json::Value {
    serde_json::json!({
        "data": [
            { "id": "solana", "name": "Solana", "symbol": "SOL" }
        ]
    })
}

#[serial]
#[tokio::test]
async fn test_cex_full_pipeline() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/exchanges/binance/tickers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(binance_tickers_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v2/assets/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(coinbase_assets_response()))
        .mount(&mock_server)
        .await;

    let config = Config {
        cex_binance_tickers_url: Some(format!(
            "{}/api/v3/exchanges/binance/tickers",
            mock_server.uri()
        )),
        cex_coinbase_assets_url: Some(format!("{}/api/v2/assets/info", mock_server.uri())),
        ..Default::default()
    };

    let pool = setup_memory_pool().await;

    coins_cex::run(&pool, &config).await.unwrap();

    let all = cex_listing::list_all(&pool).await.unwrap();
    assert_eq!(all.len(), 5, "3 binance + 2 coinbase");

    let symbols = cex_listing::list_symbols(&pool).await.unwrap();
    assert!(symbols.contains(&"BTC".to_string()));
    assert!(symbols.contains(&"ETH".to_string()));
    assert!(symbols.contains(&"SOL".to_string()));

    let ts = poll_timestamp::get_by_service(&pool, "cex_monitor")
        .await
        .unwrap()
        .expect("poll timestamp should exist");
    assert_eq!(ts.listings_found, 5);

    let events = sse_event::read_since(&pool, 0).await.unwrap();
    let cex_events: Vec<_> = events.iter().filter(|e| e.event == "cex_listing").collect();
    assert_eq!(cex_events.len(), 5);
}

#[serial]
#[tokio::test]
async fn test_cex_dedup() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/exchanges/binance/tickers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(binance_tickers_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v2/assets/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(coinbase_assets_response()))
        .mount(&mock_server)
        .await;

    let config = Config {
        cex_binance_tickers_url: Some(format!(
            "{}/api/v3/exchanges/binance/tickers",
            mock_server.uri()
        )),
        cex_coinbase_assets_url: Some(format!("{}/api/v2/assets/info", mock_server.uri())),
        ..Default::default()
    };

    let pool = setup_memory_pool().await;

    // Seed an existing listing
    let now = Utc::now().naive_utc();
    cex_listing::bulk_create(
        &pool,
        &[coins_database::CexListing {
            exchange: "binance".into(),
            external_id: "bitcoin".into(),
            token_name: "Bitcoin".into(),
            token_symbol: "BTC".into(),
            listing_url: "https://www.binance.com/en/trade/BTC_USDT".into(),
            announced_at: None,
            detected_at: now,
            ..Default::default()
        }],
    )
    .await
    .unwrap();

    coins_cex::run(&pool, &config).await.unwrap();

    let all = cex_listing::list_all(&pool).await.unwrap();
    assert_eq!(
        all.len(),
        4,
        "1 seeded + 3 new (binance: ethereum, solana; coinbase: ethereum; bitcoin deduped across both)"
    );

    let ts = poll_timestamp::get_by_service(&pool, "cex_monitor")
        .await
        .unwrap()
        .expect("poll timestamp should exist");
    assert_eq!(ts.listings_found, 3);
}

#[serial]
#[tokio::test]
async fn test_cex_api_error_does_not_crash() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/exchanges/binance/tickers"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v2/assets/info"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let config = Config {
        cex_binance_tickers_url: Some(format!(
            "{}/api/v3/exchanges/binance/tickers",
            mock_server.uri()
        )),
        cex_coinbase_assets_url: Some(format!("{}/api/v2/assets/info", mock_server.uri())),
        ..Default::default()
    };

    let pool = setup_memory_pool().await;

    coins_cex::run(&pool, &config).await.unwrap();

    let all = cex_listing::list_all(&pool).await.unwrap();
    assert!(all.is_empty());

    let ts = poll_timestamp::get_by_service(&pool, "cex_monitor")
        .await
        .unwrap()
        .expect("poll timestamp should exist");
    assert_eq!(ts.listings_found, 0);
}

#[serial]
#[tokio::test]
async fn test_cex_coinbase_array_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/exchanges/binance/tickers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(binance_empty_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v2/assets/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(coinbase_assets_array_response()))
        .mount(&mock_server)
        .await;

    let config = Config {
        cex_binance_tickers_url: Some(format!(
            "{}/api/v3/exchanges/binance/tickers",
            mock_server.uri()
        )),
        cex_coinbase_assets_url: Some(format!("{}/api/v2/assets/info", mock_server.uri())),
        ..Default::default()
    };

    let pool = setup_memory_pool().await;

    coins_cex::run(&pool, &config).await.unwrap();

    let all = cex_listing::list_all(&pool).await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].exchange, "coinbase");
    assert_eq!(all[0].token_symbol, "SOL");
}

#[test]
fn test_extract_binance_listings_parsing() {
    let tickers = serde_json::json!([
        {
            "coin_id": "bitcoin",
            "base": "BTC",
            "target": "Bitcoin",
            "trade_url": "https://www.binance.com/en/trade/BTC_USDT"
        },
        {
            "coin_id": "ethereum",
            "base": "ETH",
            "target": "Ethereum",
            "trade_url": ""
        }
    ]);

    let result = coins_cex::extract_binance_listings(&tickers.as_array().unwrap());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].exchange, "binance");
    assert_eq!(result[0].external_id, "bitcoin");
    assert_eq!(result[0].token_symbol, "BTC");
    assert_eq!(result[0].token_name, "Bitcoin");
}

#[test]
fn test_extract_binance_listings_dedup() {
    let tickers = serde_json::json!([
        {
            "coin_id": "bitcoin",
            "base": "BTC",
            "target": "Bitcoin",
            "trade_url": ""
        },
        {
            "coin_id": "bitcoin-cash",
            "base": "BCH",
            "target": "Bitcoin",
            "trade_url": ""
        }
    ]);

    let result = coins_cex::extract_binance_listings(&tickers.as_array().unwrap());
    assert_eq!(result.len(), 1, "should deduplicate by target name");
}

#[test]
fn test_extract_binance_listings_skips_empty() {
    let tickers = serde_json::json!([
        {
            "coin_id": "",
            "base": "",
            "target": "",
            "trade_url": ""
        }
    ]);

    let result = coins_cex::extract_binance_listings(&tickers.as_array().unwrap());
    assert!(result.is_empty());
}

#[test]
fn test_extract_coinbase_listings_parsing() {
    let assets = serde_json::json!([
        {
            "id": "bitcoin",
            "name": "Bitcoin",
            "symbol": "BTC"
        },
        {
            "id": "ethereum",
            "name": "Ethereum",
            "symbol": "ETH"
        }
    ]);

    let result = coins_cex::extract_coinbase_listings(&assets.as_array().unwrap());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].exchange, "coinbase");
    assert_eq!(result[0].external_id, "bitcoin");
    assert_eq!(result[0].token_symbol, "BTC");
    assert_eq!(
        result[0].listing_url,
        "https://www.coinbase.com/price/bitcoin"
    );
}

#[test]
fn test_extract_coinbase_listings_skips_empty_slug() {
    let assets = serde_json::json!([
        {
            "id": "",
            "name": "Empty",
            "symbol": "EMP"
        }
    ]);

    let result = coins_cex::extract_coinbase_listings(&assets.as_array().unwrap());
    assert!(result.is_empty());
}
