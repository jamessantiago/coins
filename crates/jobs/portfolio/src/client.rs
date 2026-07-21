use coins_config::http_client;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct DexPair {
    pub price_usd: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub volume_24h: Option<f64>,
}

pub async fn search_pairs(url: &str, address: &str) -> Vec<DexPair> {
    let client = match http_client(15) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("failed to build http client: {e:#}");
            return vec![];
        }
    };

    let full_url = format!("{}?q={}", url, address);
    let body: Value = match client.get(&full_url).send().await {
        Ok(resp) if resp.status().is_success() => resp.json().await.unwrap_or_default(),
        Ok(resp) => {
            tracing::warn!("DexScreener returned HTTP {} for {address}", resp.status());
            return vec![];
        }
        Err(e) => {
            tracing::warn!("DexScreener search failed for {address}: {e:#}");
            return vec![];
        }
    };

    let pairs = match body.get("pairs").and_then(|p| p.as_array()) {
        Some(arr) => arr,
        None => return vec![],
    };

    pairs
        .iter()
        .filter_map(|p| {
            let chain_id = p.get("chainId").and_then(|v| v.as_str()).unwrap_or("");
            if chain_id != "solana" {
                return None;
            }

            let price_usd = p
                .get("priceUsd")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let liquidity_usd = p.pointer("/liquidity/usd").and_then(|v| v.as_f64());
            let volume_24h = p.pointer("/volume/h24").and_then(|v| v.as_f64());

            Some(DexPair {
                price_usd,
                liquidity_usd,
                volume_24h,
            })
        })
        .collect()
}
