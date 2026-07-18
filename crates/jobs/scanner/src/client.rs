use std::time::Duration;

use base64::Engine;
use coins_config::scanner;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct TokenProfile {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct RaydiumPool {
    pub pool_address: String,
    pub base_mint: String,
    pub quote_mint: String,
}

#[derive(Debug, Clone)]
pub struct PumpfunCurve {
    pub pubkey: String,
}

fn known_quote_mints() -> [&'static str; 3] {
    [
        scanner::SOL_ADDRESS,
        scanner::USDC_ADDRESS,
        scanner::USDT_ADDRESS,
    ]
}

pub async fn fetch_latest_token_profiles(token_profiles_url: &str) -> Vec<Value> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();

    match client.get(token_profiles_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<Vec<Value>>().await.unwrap_or_default()
        }
        Err(e) => {
            tracing::warn!("failed to fetch token profiles from DexScreener: {e:#}");
            vec![]
        }
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!(
                "failed to fetch token profiles from DexScreener (HTTP {status}): {body}"
            );
            vec![]
        }
    }
}

pub fn extract_tokens(profiles: &[Value]) -> Vec<TokenProfile> {
    profiles
        .iter()
        .filter_map(|p| {
            let chain_id = p.get("chainId").and_then(|c| c.as_str()).unwrap_or("");
            if chain_id != "solana" {
                return None;
            }
            let address = p
                .get("tokenAddress")
                .and_then(|a| a.as_str())
                .unwrap_or("")
                .to_string();
            if address.is_empty() {
                return None;
            }
            let base = p.get("baseToken");
            let name = base
                .and_then(|b| b.get("name"))
                .and_then(|n| n.as_str())
                .or_else(|| p.get("name").and_then(|n| n.as_str()))
                .unwrap_or("")
                .to_string();
            let symbol = base
                .and_then(|b| b.get("symbol"))
                .and_then(|s| s.as_str())
                .or_else(|| p.get("symbol").and_then(|s| s.as_str()))
                .unwrap_or("")
                .to_string();
            let description = p
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            Some(TokenProfile {
                address,
                symbol,
                name,
                description,
            })
        })
        .collect()
}

async fn rpc_call(
    solana_rpc_url: &str,
    method: &str,
    params: Value,
    timeout_secs: u64,
) -> Option<Value> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .ok()?;

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });

    let resp = client
        .post(solana_rpc_url)
        .json(&payload)
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    resp.json::<Value>().await.ok()
}

fn decode_pubkey(data: &[u8], offset: usize) -> String {
    let bytes = &data[offset..offset + 32];
    bs58::encode(bytes).into_string()
}

pub async fn fetch_raydium_pools(solana_rpc_url: &str) -> Vec<RaydiumPool> {
    let params = serde_json::json!([
        scanner::RAYDIUM_AMM_PROGRAM,
        {
            "encoding": "base64",
            "filters": [{"dataSize": scanner::POOL_DATA_SIZE}],
        },
    ]);

    let result = match rpc_call(solana_rpc_url, "getProgramAccounts", params, 60).await {
        Some(r) => r,
        None => return vec![],
    };

    let items = match result.get("result") {
        Some(Value::Array(items)) => items,
        _ => return vec![],
    };

    let known_quote = known_quote_mints();
    let mut pools = Vec::new();

    for item in items {
        let pubkey = item
            .get("pubkey")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string();
        if pubkey.is_empty() {
            continue;
        }

        let account = match item.get("account") {
            Some(a) => a,
            None => continue,
        };

        let raw_data = match account.get("data") {
            Some(Value::Array(arr)) => arr.first().and_then(|d| d.as_str()).unwrap_or(""),
            Some(Value::String(s)) => s.as_str(),
            _ => continue,
        };

        if raw_data.is_empty() {
            continue;
        }

        let data = match base64::engine::general_purpose::STANDARD.decode(raw_data) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if data.len() < scanner::PC_MINT_OFFSET + 32 {
            continue;
        }

        let base_mint = decode_pubkey(&data, scanner::COIN_MINT_OFFSET);
        let quote_mint = decode_pubkey(&data, scanner::PC_MINT_OFFSET);

        if known_quote.contains(&base_mint.as_str()) {
            continue;
        }

        pools.push(RaydiumPool {
            pool_address: pubkey,
            base_mint,
            quote_mint,
        });
    }

    pools
}

pub async fn fetch_pumpfun_bonding_curves(solana_rpc_url: &str) -> Vec<PumpfunCurve> {
    let params = serde_json::json!([
        scanner::PUMPFUN_PROGRAM,
        {
            "encoding": "base64",
            "filters": [{"dataSize": scanner::PUMPFUN_BC_SIZE}],
        },
    ]);

    let result = match rpc_call(solana_rpc_url, "getProgramAccounts", params, 60).await {
        Some(r) => r,
        None => return vec![],
    };

    let items = match result.get("result") {
        Some(Value::Array(items)) => items,
        _ => return vec![],
    };

    let mut curves = Vec::new();

    for item in items {
        let pubkey = item
            .get("pubkey")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string();
        if pubkey.is_empty() {
            continue;
        }

        let account = match item.get("account") {
            Some(a) => a,
            None => continue,
        };

        let raw_data = match account.get("data") {
            Some(Value::Array(arr)) => arr.first().and_then(|d| d.as_str()).unwrap_or(""),
            Some(Value::String(s)) => s.as_str(),
            _ => continue,
        };

        if raw_data.is_empty() {
            continue;
        }

        let data = match base64::engine::general_purpose::STANDARD.decode(raw_data) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if data.len() < scanner::PUMPFUN_BC_SIZE as usize {
            continue;
        }

        curves.push(PumpfunCurve { pubkey });
    }

    curves
}

pub async fn fetch_mint_for_bonding_curve(bc_pubkey: &str, solana_rpc_url: &str) -> Option<String> {
    let params = serde_json::json!([
        bc_pubkey,
        { "programId": scanner::TOKEN_PROGRAM_ID },
        { "encoding": "base64" },
    ]);

    let result = rpc_call(solana_rpc_url, "getTokenAccountsByOwner", params, 15).await?;
    let accounts = result.get("result")?.get("value")?.as_array()?;

    for ta in accounts {
        let raw = ta.get("account")?.get("data")?;
        let raw_data = match raw {
            Value::Array(arr) => arr.first().and_then(|d| d.as_str()).unwrap_or(""),
            Value::String(s) => s.as_str(),
            _ => continue,
        };

        if raw_data.is_empty() {
            continue;
        }

        let data = base64::engine::general_purpose::STANDARD
            .decode(raw_data)
            .ok()?;
        if data.len() < 32 {
            continue;
        }

        return Some(bs58::encode(&data[0..32]).into_string());
    }

    None
}
