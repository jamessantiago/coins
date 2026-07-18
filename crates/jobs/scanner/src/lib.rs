mod client;
mod cluster;

pub use client::{TokenProfile, extract_tokens};

use std::collections::HashMap;

use chrono::Utc;
use coins_config::Config;
use coins_database::models::known_pool::KnownPool;
use coins_database::models::pump_bonding_curve::PumpBondingCurve;
use coins_database::models::token::Token;
use coins_database::queries::{
    cluster_count, known_pool, poll_timestamp, pump_bonding_curve, sse_event, token,
};
use sqlx::SqlitePool;

fn current_bucket() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M").to_string()
}

pub async fn run(pool: &SqlitePool, config: &Config) -> anyhow::Result<()> {
    let now = Utc::now().naive_utc();
    let bucket = current_bucket();

    tracing::info!("scanner cycle starting");

    let profiles_url = config.token_profiles_url();
    let rpc_url = config.solana_rpc_url();
    let threshold = config.spike_threshold();
    let windows = config.baseline_windows();
    let min_count = config.new_narrative_min_count();

    let profiles = client::fetch_latest_token_profiles(&profiles_url).await;

    let seen_addresses = token::list_all_addresses(pool).await?;
    let mut seen: std::collections::HashSet<String> = seen_addresses.into_iter().collect();

    let all_tokens = client::extract_tokens(&profiles);
    let dexscreener_new: Vec<client::TokenProfile> = all_tokens
        .into_iter()
        .filter(|t| !seen.contains(&t.address))
        .collect();

    for t in &dexscreener_new {
        seen.insert(t.address.clone());
    }
    let dexscreener_count = dexscreener_new.len();

    let raydium_pools = client::fetch_raydium_pools(&rpc_url).await;
    let known_pool_addresses: std::collections::HashSet<String> =
        known_pool::list_pool_addresses(pool)
            .await?
            .into_iter()
            .collect();
    let known_base_mints: std::collections::HashSet<String> = known_pool::list_base_mints(pool)
        .await?
        .into_iter()
        .collect();

    let mut new_pools: Vec<KnownPool> = Vec::new();
    let mut raydium_new: Vec<client::TokenProfile> = Vec::new();

    for p in &raydium_pools {
        if known_pool_addresses.contains(&p.pool_address) {
            continue;
        }
        new_pools.push(KnownPool {
            pool_address: p.pool_address.clone(),
            base_mint: p.base_mint.clone(),
            quote_mint: p.quote_mint.clone(),
            symbol: String::new(),
            name: String::new(),
            first_seen: now,
            last_seen: now,
        });
        if !seen.contains(&p.base_mint) && !known_base_mints.contains(&p.base_mint) {
            raydium_new.push(client::TokenProfile {
                address: p.base_mint.clone(),
                symbol: String::new(),
                name: String::new(),
                description: String::new(),
            });
            seen.insert(p.base_mint.clone());
        }
    }
    let raydium_count = raydium_new.len();

    if !new_pools.is_empty() {
        known_pool::bulk_create(pool, &new_pools).await?;
        tracing::info!("Raydium: {} new pools discovered", new_pools.len());
    }

    let pumpfun_curves = client::fetch_pumpfun_bonding_curves(&rpc_url).await;
    let known_bc: std::collections::HashSet<String> = pump_bonding_curve::list_bonding_curves(pool)
        .await?
        .into_iter()
        .collect();

    let mut new_curves: Vec<PumpBondingCurve> = Vec::new();
    let mut pumpfun_new: Vec<client::TokenProfile> = Vec::new();

    for c in &pumpfun_curves {
        if known_bc.contains(&c.pubkey) {
            continue;
        }
        let mint = client::fetch_mint_for_bonding_curve(&c.pubkey, &rpc_url).await;
        let mint = match mint {
            Some(m) => m,
            None => continue,
        };
        new_curves.push(PumpBondingCurve {
            bonding_curve: c.pubkey.clone(),
            mint: mint.clone(),
            first_seen: now,
        });
        if !seen.contains(&mint) {
            pumpfun_new.push(client::TokenProfile {
                address: mint,
                symbol: String::new(),
                name: String::new(),
                description: String::new(),
            });
        }
    }
    let pumpfun_count = pumpfun_new.len();

    if !new_curves.is_empty() {
        pump_bonding_curve::bulk_create(pool, &new_curves).await?;
        tracing::info!(
            "Pump.fun: {} new bonding curves discovered",
            new_curves.len()
        );
    }

    let all_new: Vec<client::TokenProfile> = dexscreener_new
        .into_iter()
        .chain(raydium_new)
        .chain(pumpfun_new)
        .collect();

    let mut cluster_hits: HashMap<String, i32> = HashMap::new();
    let mut token_records: Vec<Token> = Vec::new();

    for t in &all_new {
        token_records.push(Token {
            address: t.address.clone(),
            symbol: t.symbol.clone(),
            name: t.name.clone(),
            chain_id: "solana".to_string(),
            first_seen: now,
            last_seen: now,
        });
        for c in cluster::match_clusters(&format!("{} {}", t.description, t.name), &t.symbol) {
            *cluster_hits.entry(c).or_insert(0) += 1;
        }
    }

    if !token_records.is_empty() {
        token::bulk_create(pool, &token_records).await?;
    }

    let mut alerts: Vec<serde_json::Value> = Vec::new();

    let mut sorted_clusters: Vec<(String, i32)> = cluster_hits.into_iter().collect();
    sorted_clusters.sort_by_key(|b| std::cmp::Reverse(b.1));

    for (cluster, count) in &sorted_clusters {
        cluster_count::upsert(pool, cluster, &bucket, *count).await?;

        let baseline_counts = cluster_count::get_baseline(pool, cluster, &bucket, windows).await?;
        let baseline = if baseline_counts.is_empty() {
            0.0
        } else {
            baseline_counts.iter().sum::<i32>() as f64 / baseline_counts.len() as f64
        };

        let is_spike = if baseline > 0.0 {
            *count as f64 >= baseline * threshold
        } else {
            *count >= min_count
        };

        if is_spike {
            let alert = serde_json::json!({
                "cluster": cluster,
                "count": count,
                "baseline": (baseline * 10.0).round() / 10.0,
            });
            alerts.push(alert.clone());
            tracing::info!(
                "Narrative spike: {} count={} baseline={:.1}",
                cluster,
                count,
                baseline
            );

            let data_str = serde_json::to_string(&alert).unwrap_or_default();
            sse_event::create(pool, "narrative_spike", &data_str).await?;
        }
    }

    let total_new = all_new.len() as i32;
    poll_timestamp::upsert(pool, "scanner", now, total_new).await?;

    tracing::info!(
        "Scan complete: {} new tokens ({} DexScreener, {} Raydium, {} pump.fun), {} alerts",
        total_new,
        dexscreener_count,
        raydium_count,
        pumpfun_count,
        alerts.len()
    );

    tracing::info!("scanner cycle finished");
    Ok(())
}
