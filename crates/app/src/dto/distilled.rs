use coins_database::models::distilled_token::DistilledToken;
use serde::Serialize;

use crate::dto::common::fmt_ts;

#[derive(Serialize)]
pub struct DistilledTokenResponse {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub first_seen: String,
    pub last_seen: String,
    pub sources: String,
    pub safety_score: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub volume_24h: Option<f64>,
    pub fdv: Option<f64>,
    pub narrative_clusters: String,
    pub telegram_mentions: i32,
    pub cex_listed: bool,
    pub research_conviction: Option<i32>,
    pub dexscreener_url: String,
    pub ranking_score: f64,
    pub price_change_24h: Option<f64>,
    pub price_change_1h: Option<f64>,
    pub vol_liq_ratio: Option<f64>,
    pub buy_sell_ratio: Option<f64>,
    pub updated_at: String,
}

impl From<DistilledToken> for DistilledTokenResponse {
    fn from(t: DistilledToken) -> Self {
        Self {
            address: t.address,
            symbol: t.symbol,
            name: t.name,
            first_seen: fmt_ts(t.first_seen),
            last_seen: fmt_ts(t.last_seen),
            sources: t.sources,
            safety_score: t.safety_score,
            liquidity_usd: t.liquidity_usd,
            volume_24h: t.volume_24h,
            fdv: t.fdv,
            narrative_clusters: t.narrative_clusters,
            telegram_mentions: t.telegram_mentions,
            cex_listed: t.cex_listed,
            research_conviction: t.research_conviction,
            dexscreener_url: t.dexscreener_url,
            ranking_score: t.ranking_score,
            price_change_24h: t.price_change_24h,
            price_change_1h: t.price_change_1h,
            vol_liq_ratio: t.vol_liq_ratio,
            buy_sell_ratio: t.buy_sell_ratio,
            updated_at: fmt_ts(t.updated_at),
        }
    }
}
