mod client;

pub use client::{CexListingCandidate, extract_binance_listings, extract_coinbase_listings};

use chrono::Utc;
use coins_config::Config;
use coins_database::models::cex_listing::CexListing;
use coins_database::queries::{cex_listing, poll_timestamp, sse_event};
use sqlx::SqlitePool;

pub async fn run(pool: &SqlitePool, config: &Config) -> anyhow::Result<()> {
    let now = Utc::now().naive_utc();

    tracing::info!("CEX scan cycle starting");

    let known_ids: std::collections::HashSet<String> = cex_listing::list_external_ids(pool)
        .await?
        .into_iter()
        .collect();

    let mut new_listings: Vec<CexListing> = Vec::new();
    let mut total_candidates = 0;

    let binance_url = config.cex_binance_tickers_url();
    let coinbase_url = config.cex_coinbase_assets_url();

    // Binance (via CoinGecko proxy)
    {
        let tickers = client::fetch_binance_tickers(&binance_url).await;
        let candidates = client::extract_binance_listings(&tickers);
        total_candidates += candidates.len();

        for c in candidates {
            if known_ids.contains(&c.external_id) {
                continue;
            }
            new_listings.push(CexListing {
                exchange: c.exchange,
                external_id: c.external_id,
                token_name: c.token_name,
                token_symbol: c.token_symbol,
                listing_url: c.listing_url,
                announced_at: None,
                detected_at: now,
                ..Default::default()
            });
        }
    }

    // Coinbase
    {
        let assets = client::fetch_coinbase_assets(&coinbase_url).await;
        let candidates = client::extract_coinbase_listings(&assets);
        total_candidates += candidates.len();

        for c in candidates {
            if known_ids.contains(&c.external_id) {
                continue;
            }
            new_listings.push(CexListing {
                exchange: c.exchange,
                external_id: c.external_id,
                token_name: c.token_name,
                token_symbol: c.token_symbol,
                listing_url: c.listing_url,
                announced_at: None,
                detected_at: now,
                ..Default::default()
            });
        }
    }

    let new_count = new_listings.len() as i32;

    if !new_listings.is_empty() {
        cex_listing::bulk_create(pool, &new_listings).await?;

        for l in &new_listings {
            let data = serde_json::json!({
                "exchange": l.exchange,
                "symbol": l.token_symbol,
                "name": l.token_name,
            });
            sse_event::create(
                pool,
                "cex_listing",
                &serde_json::to_string(&data).unwrap_or_default(),
            )
            .await?;
        }

        tracing::info!("New CEX listings ({})", new_listings.len());
    }

    poll_timestamp::upsert(pool, "cex_monitor", now, new_count).await?;

    tracing::info!(
        "CEX scan complete: {} total candidates, {} new listings",
        total_candidates,
        new_count,
    );

    tracing::info!("CEX scan cycle finished");
    Ok(())
}
