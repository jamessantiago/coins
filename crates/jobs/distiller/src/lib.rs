mod client;

use std::collections::{HashMap, HashSet};

use chrono::Utc;
use coins_config::Config;
use coins_database::models::distilled_token::DistilledToken;
use coins_database::queries::{
    cex_listing as cex_queries, distilled_token as dt_queries, poll_timestamp,
    research_entry as research_queries, sse_event, telegram as tg_queries, token as token_queries,
};
use coins_scanner::cluster;
use sqlx::SqlitePool;

const SOURCE_SCANNER: &str = "scanner";
const SOURCE_TELEGRAM: &str = "telegram";
const SOURCE_RESEARCH: &str = "research";

fn compute_ranking_score(
    safety_score: Option<f64>,
    liquidity_usd: Option<f64>,
    volume_24h: Option<f64>,
    telegram_mentions: i32,
    research_conviction: Option<i32>,
    cex_listed: bool,
    price_change_24h: Option<f64>,
    vol_liq_ratio: Option<f64>,
    buy_sell_ratio: Option<f64>,
) -> f64 {
    let score = (safety_score.unwrap_or(0.0)) * 2.5
        + (liquidity_usd.unwrap_or(0.0) / 100_000.0).min(1.0) * 15.0
        + (volume_24h.unwrap_or(0.0) / 50_000.0).min(1.0) * 10.0
        + (telegram_mentions as f64 / 10.0).min(1.0) * 10.0
        + (research_conviction.unwrap_or(0) as f64 / 5.0) * 10.0
        + if cex_listed { 10.0 } else { 0.0 }
        + (price_change_24h.unwrap_or(0.0).max(0.0) / 10.0).min(1.0) * 10.0
        + vol_liq_ratio.unwrap_or(0.0).min(1.0) * 5.0
        + buy_sell_ratio.unwrap_or(0.0).min(1.0) * 5.0;

    (score * 10.0).round() / 10.0
}

fn should_recheck_safety(
    existing: Option<&DistilledToken>,
    sources: &HashSet<String>,
) -> bool {
    match existing {
        None => true,
        Some(t) => {
            if t.safety_score.is_none() {
                return true;
            }
            if sources.contains(SOURCE_TELEGRAM) || sources.contains(SOURCE_RESEARCH) {
                return true;
            }
            false
        }
    }
}

pub async fn run(pool: &SqlitePool, config: &Config) -> anyhow::Result<()> {
    let now = Utc::now().naive_utc();
    let search_url = config.dexscreener_search_url();

    tracing::info!("distiller cycle starting");

    // ---- Step 1: Collect addresses from all sources ----
    let mut sources: HashMap<String, HashSet<String>> = HashMap::new();

    for addr in token_queries::list_all_addresses(pool).await? {
        sources.entry(addr).or_default().insert(SOURCE_SCANNER.to_string());
    }

    for msg in tg_queries::iterate_messages_with_addresses(pool).await? {
        for line in msg.extracted_addresses.lines() {
            let addr = line.trim().to_string();
            if !addr.is_empty() {
                sources.entry(addr).or_default().insert(SOURCE_TELEGRAM.to_string());
            }
        }
    }

    for addr in research_queries::list_all_addresses(pool).await? {
        sources.entry(addr).or_default().insert(SOURCE_RESEARCH.to_string());
    }

    // ---- Step 1b: Collect telegram mention counts ----
    let telegram_counts: HashMap<String, i32> = {
        let mut counts: HashMap<String, i32> = HashMap::new();
        for msg in tg_queries::iterate_messages_with_addresses(pool).await? {
            for line in msg.extracted_addresses.lines() {
                let addr = line.trim().to_string();
                if !addr.is_empty() {
                    *counts.entry(addr).or_insert(0) += 1;
                }
            }
        }
        counts
    };

    // ---- Step 1c: Collect research data ----
    let research_data: HashMap<String, Option<i32>> = research_queries::list_all(pool)
        .await?
        .into_iter()
        .map(|r| (r.address, Some(r.conviction)))
        .collect();

    // ---- Step 1d: Collect CEX symbols ----
    let cex_symbols: HashSet<String> = cex_queries::list_symbols(pool)
        .await?
        .into_iter()
        .map(|s| s.to_uppercase())
        .collect();

    // ---- Step 2: Process each unique address ----
    let mut merged: Vec<(String, HashSet<String>)> = sources.into_iter().collect();
    merged.sort_by(|a, b| a.0.cmp(&b.0));

    let mut processed = 0i32;

    for (address, srcs) in &merged {
        let mut source_list: Vec<&str> = srcs.iter().map(|s| s.as_str()).collect();
        source_list.sort();
        let source_str = source_list.join(",");

        let existing = dt_queries::get_by_address(pool, address).await?;

        // ---- 2a: Fetch DexScreener pairs ----
        let pairs = client::search_pairs(&search_url, address).await;

        // ---- 2b: Pick best pair by liquidity ----
        let best_pair = pairs
            .into_iter()
            .max_by(|a, b| {
                a.liquidity_usd
                    .unwrap_or(0.0)
                    .partial_cmp(&b.liquidity_usd.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        // ---- 2c: Extract/fallback fields ----
        let symbol = best_pair
            .as_ref()
            .map(|p| p.symbol.clone())
            .filter(|s| !s.is_empty())
            .or_else(|| existing.as_ref().map(|t| t.symbol.clone()))
            .unwrap_or_default();

        let name = best_pair
            .as_ref()
            .map(|p| p.name.clone())
            .filter(|s| !s.is_empty())
            .or_else(|| existing.as_ref().map(|t| t.name.clone()))
            .unwrap_or_default();

        let liq = best_pair
            .as_ref()
            .and_then(|p| p.liquidity_usd)
            .or_else(|| existing.as_ref().and_then(|t| t.liquidity_usd));

        let vol = best_pair
            .as_ref()
            .and_then(|p| p.volume_24h)
            .or_else(|| existing.as_ref().and_then(|t| t.volume_24h));

        let fdv = best_pair
            .as_ref()
            .and_then(|p| p.fdv)
            .or_else(|| existing.as_ref().and_then(|t| t.fdv));

        let price_change_24h = best_pair
            .as_ref()
            .and_then(|p| p.price_change_24h)
            .or_else(|| existing.as_ref().and_then(|t| t.price_change_24h));

        let price_change_1h = best_pair
            .as_ref()
            .and_then(|p| p.price_change_1h)
            .or_else(|| existing.as_ref().and_then(|t| t.price_change_1h));

        let txn_buys = best_pair.as_ref().and_then(|p| p.txn_buys_24h);
        let txn_sells = best_pair.as_ref().and_then(|p| p.txn_sells_24h);

        // ---- 2d: Build DexScreener URL ----
        let dexscreener_url = best_pair
            .as_ref()
            .and_then(|p| {
                if p.pair_address.is_empty() {
                    None
                } else {
                    Some(format!("https://dexscreener.com/solana/{}", p.pair_address))
                }
            })
            .unwrap_or_default();

        // ---- 2e: Cluster matching ----
        let clusters = cluster::match_clusters(&name, &symbol);
        let cluster_str = if clusters.is_empty() {
            existing
                .as_ref()
                .map(|t| t.narrative_clusters.clone())
                .unwrap_or_default()
        } else {
            clusters.join(",")
        };

        // ---- 2f: Telegram mention count ----
        let tg_count = telegram_counts
            .get(address)
            .copied()
            .or_else(|| existing.as_ref().map(|t| t.telegram_mentions))
            .unwrap_or(0);

        // ---- 2g: CEX listing check ----
        let cex_listed = if !symbol.is_empty() {
            cex_symbols.contains(&symbol.to_uppercase())
        } else {
            existing.as_ref().map(|t| t.cex_listed).unwrap_or(false)
        };

        // ---- 2h: Research conviction ----
        let conviction = research_data
            .get(address)
            .copied()
            .flatten()
            .or_else(|| existing.as_ref().and_then(|t| t.research_conviction));

        // ---- 2i: Safety check (skip for now; pluggable) ----
        let fresh_safety: Option<f64> = None;
        let safety_score = fresh_safety.or_else(|| {
            if should_recheck_safety(existing.as_ref(), srcs) {
                None
            } else {
                existing.as_ref().and_then(|t| t.safety_score)
            }
        });

        // ---- 2j: Compute ranking score ----
        let vol_liq_ratio = match (vol, liq) {
            (Some(v), Some(l)) if l > 0.0 => Some((v / l).min(10.0) / 10.0),
            _ => existing.as_ref().and_then(|t| t.vol_liq_ratio),
        };

        let buy_sell_ratio = match (txn_buys, txn_sells) {
            (Some(b), Some(s)) if s > 0 => Some((b as f64 / s as f64).min(3.0) / 3.0),
            _ => existing.as_ref().and_then(|t| t.buy_sell_ratio),
        };

        let first_seen = existing
            .as_ref()
            .map(|t| t.first_seen)
            .unwrap_or(now);

        let ranking_score = compute_ranking_score(
            safety_score,
            liq,
            vol,
            tg_count,
            conviction,
            cex_listed,
            price_change_24h,
            vol_liq_ratio,
            buy_sell_ratio,
        );

        // ---- 2k: Save ----
        let token = DistilledToken {
            address: address.clone(),
            symbol: symbol.chars().take(32).collect(),
            name: name.chars().take(128).collect(),
            first_seen,
            last_seen: now,
            sources: source_str,
            safety_score: safety_score.or_else(|| existing.as_ref().and_then(|t| t.safety_score)),
            liquidity_usd: liq,
            volume_24h: vol,
            fdv,
            narrative_clusters: cluster_str,
            telegram_mentions: tg_count,
            cex_listed,
            research_conviction: conviction,
            dexscreener_url,
            ranking_score,
            price_change_24h,
            price_change_1h,
            vol_liq_ratio,
            buy_sell_ratio,
            updated_at: now,
        };

        let is_new = existing.is_none();
        dt_queries::upsert(pool, &token).await?;

        // ---- 2l: Emit SSE event if new ----
        if is_new {
            let data = serde_json::json!({
                "address": address,
                "symbol": if !token.symbol.is_empty() { &token.symbol } else { &address[..address.len().min(8)] },
            });
            let data_str = serde_json::to_string(&data).unwrap_or_default();
            if let Err(e) = sse_event::create(pool, "distilled_token", &data_str).await {
                tracing::warn!("failed to emit distilled_token SSE event: {e:#}");
            }
        }

        processed += 1;
    }

    // ---- Step 3: Re-score existing tokens with ranking_score = 0 ----
    let unscored = dt_queries::list_unscored(pool).await?;
    let unscored_count = unscored.len();
    for t in &unscored {
        let score = compute_ranking_score(
            t.safety_score,
            t.liquidity_usd,
            t.volume_24h,
            t.telegram_mentions,
            t.research_conviction,
            t.cex_listed,
            t.price_change_24h,
            t.vol_liq_ratio,
            t.buy_sell_ratio,
        );
        dt_queries::update_ranking_score(pool, &t.address, score).await?;
        processed += 1;
    }

    // ---- Step 4: Update poll timestamp ----
    poll_timestamp::upsert(pool, "distiller", now, processed).await?;

    tracing::info!(
        "distiller cycle finished: {} total processed ({} new/rescored, {} unscored fallback)",
        processed,
        merged.len(),
        unscored_count,
    );

    Ok(())
}
