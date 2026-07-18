use std::collections::HashSet;
use std::time::Duration;

use serde_json::Value;

fn http_client(timeout_secs: u64) -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
}

pub async fn fetch_binance_tickers(url: &str) -> Vec<Value> {
    let client = match http_client(15) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("failed to build http client: {e:#}");
            return vec![];
        }
    };

    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let body: Value = resp.json().await.unwrap_or_default();
            body.get("tickers")
                .and_then(|t| t.as_array())
                .cloned()
                .unwrap_or_default()
        }
        Ok(resp) => {
            let status = resp.status();
            tracing::warn!("CoinGecko returned HTTP {status}");
            vec![]
        }
        Err(e) => {
            tracing::warn!("failed to fetch Binance tickers from CoinGecko: {e:#}");
            vec![]
        }
    }
}

pub fn extract_binance_listings(tickers: &[Value]) -> Vec<CexListingCandidate> {
    let mut seen = HashSet::new();
    let mut listings = Vec::new();

    for t in tickers {
        let target = t.get("target").and_then(|v| v.as_str()).unwrap_or("");
        if target.is_empty() || seen.contains(target) {
            continue;
        }
        seen.insert(target.to_string());

        let coin_id = t.get("coin_id").and_then(|v| v.as_str()).unwrap_or("");
        let base = t.get("base").and_then(|v| v.as_str()).unwrap_or("");
        let trade_url = t.get("trade_url").and_then(|v| v.as_str()).unwrap_or("");

        listings.push(CexListingCandidate {
            exchange: "binance".into(),
            external_id: coin_id.to_string(),
            token_name: target.to_string(),
            token_symbol: base.to_string(),
            listing_url: trade_url.to_string(),
        });
    }

    listings
}

pub async fn fetch_coinbase_assets(url: &str) -> Vec<Value> {
    let client = match http_client(15) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("failed to build http client: {e:#}");
            return vec![];
        }
    };

    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let body: Value = resp.json().await.unwrap_or_default();
            match body.get("data") {
                Some(Value::Object(map)) => map.values().cloned().collect(),
                Some(Value::Array(arr)) => arr.clone(),
                _ => vec![],
            }
        }
        Ok(resp) => {
            let status = resp.status();
            tracing::warn!("Coinbase returned HTTP {status}");
            vec![]
        }
        Err(e) => {
            tracing::warn!("failed to fetch Coinbase assets: {e:#}");
            vec![]
        }
    }
}

pub fn extract_coinbase_listings(assets: &[Value]) -> Vec<CexListingCandidate> {
    let mut listings = Vec::new();

    for a in assets {
        let slug = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if slug.is_empty() {
            continue;
        }

        let name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let symbol = a.get("symbol").and_then(|v| v.as_str()).unwrap_or("");

        listings.push(CexListingCandidate {
            exchange: "coinbase".into(),
            external_id: slug.to_string(),
            token_name: name.to_string(),
            token_symbol: symbol.to_string(),
            listing_url: format!("https://www.coinbase.com/price/{}", slug),
        });
    }

    listings
}

#[derive(Debug, Clone)]
pub struct CexListingCandidate {
    pub exchange: String,
    pub external_id: String,
    pub token_name: String,
    pub token_symbol: String,
    pub listing_url: String,
}
