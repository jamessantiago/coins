mod util;

use base64::Engine;
use coins_config::Config;
use coins_database::models::token::Token;
use coins_database::queries::{
    cluster_count, known_pool, poll_timestamp, pump_bonding_curve, sse_event, token,
};
use util::{pubkey_bytes, pubkey_str, setup_memory_pool};
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use chrono::Utc;
use serial_test::serial;

fn test_config(mock_uri: &str) -> Config {
    Config {
        solana_rpc_url: Some(mock_uri.to_string()),
        dexscreener_token_profiles_url: Some(format!("{}/token-profiles/latest/v1", mock_uri)),
        new_narrative_min_count: Some(1),
        ..Default::default()
    }
}

fn make_raydium_pool_data() -> Vec<u8> {
    let mut data = vec![0u8; 752];
    data[144..176].copy_from_slice(&pubkey_bytes(1));
    data[176..208].copy_from_slice(&pubkey_bytes(2));
    data
}

fn make_pumpfun_curve_data() -> Vec<u8> {
    vec![0u8; 105]
}

fn make_token_account_data() -> Vec<u8> {
    let mut data = vec![0u8; 165];
    data[0..32].copy_from_slice(&pubkey_bytes(4));
    data
}

fn base64_encode(data: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn dexscreener_response() -> serde_json::Value {
    serde_json::json!([
        {
            "chainId": "solana",
            "tokenAddress": pubkey_str(10),
            "baseToken": { "name": "AI Agent Token", "symbol": "AIAGT" },
            "description": "An autonomous AI agent token with neural compute"
        },
        {
            "chainId": "solana",
            "tokenAddress": pubkey_str(11),
            "baseToken": { "name": "Doge Moon Cat", "symbol": "DOGECAT" },
            "description": "A meme token for the people"
        },
        {
            "chainId": "ethereum",
            "tokenAddress": "0xNotSolana",
            "baseToken": { "name": "Eth Token", "symbol": "ETH" },
            "description": "Not on Solana"
        }
    ])
}

fn solana_raydium_response() -> serde_json::Value {
    let pool_data = base64_encode(&make_raydium_pool_data());
    serde_json::json!({
        "jsonrpc": "2.0",
        "result": [{
            "pubkey": pubkey_str(20),
            "account": { "data": [pool_data, "base64"] }
        }],
        "id": 1
    })
}

fn solana_pumpfun_response() -> serde_json::Value {
    let curve_data = base64_encode(&make_pumpfun_curve_data());
    serde_json::json!({
        "jsonrpc": "2.0",
        "result": [{
            "pubkey": pubkey_str(30),
            "account": { "data": [curve_data, "base64"] }
        }],
        "id": 1
    })
}

fn solana_token_account_response() -> serde_json::Value {
    let account_data = base64_encode(&make_token_account_data());
    serde_json::json!({
        "jsonrpc": "2.0",
        "result": {
            "value": [{
                "account": { "data": [account_data, "base64"] }
            }]
        },
        "id": 1
    })
}

async fn seed_existing_token(pool: &sqlx::SqlitePool) {
    let now = Utc::now().naive_utc();
    token::bulk_create(
        pool,
        &[Token {
            address: pubkey_str(10),
            symbol: "EXISTING".to_string(),
            name: "Existing Token".to_string(),
            chain_id: "solana".to_string(),
            first_seen: now,
            last_seen: now,
        }],
    )
    .await
    .unwrap();
}

async fn seed_known_base_mint(pool: &sqlx::SqlitePool) {
    let now = Utc::now().naive_utc();
    known_pool::bulk_create(
        pool,
        &[coins_database::models::known_pool::KnownPool {
            pool_address: "known-pool-1".to_string(),
            base_mint: pubkey_str(1),
            quote_mint: pubkey_str(2),
            symbol: String::new(),
            name: String::new(),
            first_seen: now,
            last_seen: now,
        }],
    )
    .await
    .unwrap();
}

#[serial]
#[tokio::test]
async fn test_scanner_full_pipeline() {
    let mock_server = MockServer::start().await;
    let config = test_config(&mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/token-profiles/latest/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(dexscreener_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getProgramAccounts"))
        .and(body_string_contains(
            "675kPX9MHTjS2zt1q1frNYHuzeLXfQM9H24wFSUt1Mp8",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(solana_raydium_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getProgramAccounts"))
        .and(body_string_contains(
            "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(solana_pumpfun_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getTokenAccountsByOwner"))
        .respond_with(ResponseTemplate::new(200).set_body_json(solana_token_account_response()))
        .mount(&mock_server)
        .await;

    let pool = setup_memory_pool().await;
    seed_existing_token(&pool).await;

    coins_scanner::run(&pool, &config).await.unwrap();

    let all_addresses = token::list_all_addresses(&pool).await.unwrap();
    assert_eq!(all_addresses.len(), 4, "1 seeded + 3 new tokens");
    assert!(all_addresses.contains(&pubkey_str(10)));
    assert!(all_addresses.contains(&pubkey_str(11)));
    assert!(all_addresses.contains(&pubkey_str(1)));
    assert!(all_addresses.contains(&pubkey_str(4)));

    let pool_addresses = known_pool::list_pool_addresses(&pool).await.unwrap();
    assert_eq!(pool_addresses.len(), 1);
    assert!(pool_addresses.contains(&pubkey_str(20)));

    let bc_addresses = pump_bonding_curve::list_bonding_curves(&pool)
        .await
        .unwrap();
    assert_eq!(bc_addresses.len(), 1);
    assert!(bc_addresses.contains(&pubkey_str(30)));

    let base_mints = known_pool::list_base_mints(&pool).await.unwrap();
    assert!(base_mints.contains(&pubkey_str(1)));

    let ts = poll_timestamp::get_by_service(&pool, "scanner")
        .await
        .unwrap()
        .expect("poll timestamp should exist");
    assert_eq!(ts.listings_found, 3);

    let events = sse_event::read_since(&pool, 0).await.unwrap();
    assert!(!events.is_empty(), "should have spike alerts");
    let spike_events: Vec<_> = events
        .iter()
        .filter(|e| e.event == "narrative_spike")
        .collect();
    assert!(
        !spike_events.is_empty(),
        "should have narrative_spike events"
    );
}

#[serial]
#[tokio::test]
async fn test_scanner_dedup_raydium_base_mint() {
    let mock_server = MockServer::start().await;
    let config = test_config(&mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/token-profiles/latest/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(dexscreener_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getProgramAccounts"))
        .and(body_string_contains(
            "675kPX9MHTjS2zt1q1frNYHuzeLXfQM9H24wFSUt1Mp8",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(solana_raydium_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getProgramAccounts"))
        .and(body_string_contains(
            "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(solana_pumpfun_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getTokenAccountsByOwner"))
        .respond_with(ResponseTemplate::new(200).set_body_json(solana_token_account_response()))
        .mount(&mock_server)
        .await;

    let pool = setup_memory_pool().await;
    seed_existing_token(&pool).await;
    seed_known_base_mint(&pool).await;

    coins_scanner::run(&pool, &config).await.unwrap();

    let pool_addresses = known_pool::list_pool_addresses(&pool).await.unwrap();
    assert_eq!(pool_addresses.len(), 2, "1 seeded pool + 1 new pool");

    let all_addresses = token::list_all_addresses(&pool).await.unwrap();
    assert_eq!(
        all_addresses.len(),
        3,
        "1 seeded + 1 dexscreener + 1 pumpfun; raydium base_mint is known"
    );
}

#[serial]
#[tokio::test]
async fn test_scanner_empty_dexscreener() {
    let mock_server = MockServer::start().await;
    let config = test_config(&mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/token-profiles/latest/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getProgramAccounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(solana_raydium_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getTokenAccountsByOwner"))
        .respond_with(ResponseTemplate::new(200).set_body_json(solana_token_account_response()))
        .mount(&mock_server)
        .await;

    let pool = setup_memory_pool().await;

    coins_scanner::run(&pool, &config).await.unwrap();

    let all_addresses = token::list_all_addresses(&pool).await.unwrap();
    assert_eq!(all_addresses.len(), 2);

    let ts = poll_timestamp::get_by_service(&pool, "scanner")
        .await
        .unwrap()
        .expect("poll timestamp should exist");
    assert_eq!(ts.listings_found, 2);
}

#[serial]
#[tokio::test]
async fn test_scanner_rpc_error_does_not_crash() {
    let mock_server = MockServer::start().await;
    let config = test_config(&mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/token-profiles/latest/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(dexscreener_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getProgramAccounts"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getTokenAccountsByOwner"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let pool = setup_memory_pool().await;
    seed_existing_token(&pool).await;

    coins_scanner::run(&pool, &config).await.unwrap();

    let all_addresses = token::list_all_addresses(&pool).await.unwrap();
    assert_eq!(all_addresses.len(), 2);
}

#[serial]
#[tokio::test]
async fn test_scanner_cluster_count_upserted() {
    let mock_server = MockServer::start().await;
    let config = test_config(&mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/token-profiles/latest/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(dexscreener_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getProgramAccounts"))
        .and(body_string_contains(
            "675kPX9MHTjS2zt1q1frNYHuzeLXfQM9H24wFSUt1Mp8",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(solana_raydium_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getProgramAccounts"))
        .and(body_string_contains(
            "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(solana_pumpfun_response()))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getTokenAccountsByOwner"))
        .respond_with(ResponseTemplate::new(200).set_body_json(solana_token_account_response()))
        .mount(&mock_server)
        .await;

    let pool = setup_memory_pool().await;
    seed_existing_token(&pool).await;

    coins_scanner::run(&pool, &config).await.unwrap();

    let bucket = Utc::now().format("%Y-%m-%dT%H:%M").to_string();
    let counts = cluster_count::get_by_bucket(&pool, &bucket).await.unwrap();
    assert!(
        !counts.is_empty(),
        "cluster counts should exist for the current bucket"
    );

    for cc in &counts {
        assert!(!cc.cluster.is_empty());
        assert!(cc.count > 0);
    }
}

#[serial]
#[tokio::test]
async fn test_scanner_no_tokens_then_no_alerts() {
    let mock_server = MockServer::start().await;
    let config = test_config(&mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/token-profiles/latest/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_string_contains("getProgramAccounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": [],
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let pool = setup_memory_pool().await;

    coins_scanner::run(&pool, &config).await.unwrap();

    let events = sse_event::read_since(&pool, 0).await.unwrap();
    let spike_events: Vec<_> = events
        .iter()
        .filter(|e| e.event == "narrative_spike")
        .collect();
    assert!(spike_events.is_empty());

    let ts = poll_timestamp::get_by_service(&pool, "scanner")
        .await
        .unwrap()
        .expect("poll timestamp should exist");
    assert_eq!(ts.listings_found, 0);
}
