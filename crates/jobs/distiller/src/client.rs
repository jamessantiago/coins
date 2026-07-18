use coins_config::http_client;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct DexPair {
    pub pair_address: String,
    pub symbol: String,
    pub name: String,
    pub liquidity_usd: Option<f64>,
    pub volume_24h: Option<f64>,
    pub fdv: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub price_change_1h: Option<f64>,
    pub txn_buys_24h: Option<i64>,
    pub txn_sells_24h: Option<i64>,
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

            let base = p.get("baseToken")?;
            let symbol = base.get("symbol").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = base.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let pair_address = p.get("pairAddress").and_then(|v| v.as_str()).unwrap_or("").to_string();

            let liquidity_usd = p
                .pointer("/liquidity/usd")
                .and_then(|v| v.as_f64());

            let volume_24h = p
                .pointer("/volume/h24")
                .and_then(|v| v.as_f64());

            let fdv = p.get("fdv").and_then(|v| v.as_f64());

            let price_change_24h = p
                .pointer("/priceChange/h24")
                .and_then(|v| v.as_f64());

            let price_change_1h = p
                .pointer("/priceChange/h1")
                .and_then(|v| v.as_f64());

            let txn_buys_24h = p
                .pointer("/txns/h24/buys")
                .and_then(|v| v.as_i64());

            let txn_sells_24h = p
                .pointer("/txns/h24/sells")
                .and_then(|v| v.as_i64());

            Some(DexPair {
                pair_address,
                symbol,
                name,
                liquidity_usd,
                volume_24h,
                fdv,
                price_change_24h,
                price_change_1h,
                txn_buys_24h,
                txn_sells_24h,
            })
        })
        .collect()
}
