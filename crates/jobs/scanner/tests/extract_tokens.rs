use coins_scanner::extract_tokens;
use serde_json::json;

fn profile(overrides: &[(&str, serde_json::Value)]) -> serde_json::Value {
    let mut base = json!({
        "chainId": "solana",
        "tokenAddress": "A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0",
        "baseToken": {
            "name": "TestToken",
            "symbol": "TT"
        },
        "description": "A test token"
    });
    for (key, val) in overrides {
        base[key] = val.clone();
    }
    base
}

#[tokio::test]
async fn test_extract_solana_only() {
    let profiles = vec![
        profile(&[]),
        profile(&[("chainId", json!("ethereum"))]),
        profile(&[("chainId", json!("bsc"))]),
    ];
    let tokens = extract_tokens(&profiles);
    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0].address,
        "A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0"
    );
}

#[tokio::test]
async fn test_extract_fields() {
    let profiles = vec![profile(&[])];
    let tokens = extract_tokens(&profiles);
    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0].address,
        "A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0"
    );
    assert_eq!(tokens[0].symbol, "TT");
    assert_eq!(tokens[0].name, "TestToken");
    assert_eq!(tokens[0].description, "A test token");
}

#[tokio::test]
async fn test_extract_missing_address() {
    let profiles = vec![profile(&[("tokenAddress", json!(""))])];
    let tokens = extract_tokens(&profiles);
    assert!(tokens.is_empty());
}

#[tokio::test]
async fn test_extract_no_base_token() {
    let profiles = vec![json!({
        "chainId": "solana",
        "tokenAddress": "Addr1",
        "name": "TopLevel",
        "symbol": "TL",
        "description": "desc"
    })];
    let tokens = extract_tokens(&profiles);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].name, "TopLevel");
    assert_eq!(tokens[0].symbol, "TL");
}

#[tokio::test]
async fn test_extract_base_token_without_name() {
    let profiles = vec![json!({
        "chainId": "solana",
        "tokenAddress": "Addr2",
        "baseToken": {},
        "description": "desc"
    })];
    let tokens = extract_tokens(&profiles);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].name, "");
    assert_eq!(tokens[0].symbol, "");
}

#[tokio::test]
async fn test_extract_empty_profiles() {
    let tokens = extract_tokens(&[]);
    assert!(tokens.is_empty());
}

#[tokio::test]
async fn test_extract_unknown_chain() {
    let profiles = vec![json!({
        "chainId": "unknown-chain",
        "tokenAddress": "Addr3"
    })];
    let tokens = extract_tokens(&profiles);
    assert!(tokens.is_empty());
}

#[tokio::test]
async fn test_extract_partial_overrides() {
    let profiles = vec![profile(&[(
        "description",
        json!("AI agent token for neural networks"),
    )])];
    let tokens = extract_tokens(&profiles);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].description, "AI agent token for neural networks");
}
