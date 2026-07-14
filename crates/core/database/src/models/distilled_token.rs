use chrono::NaiveDateTime;

/// Tokens distilled from all sources
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct DistilledToken {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub first_seen: NaiveDateTime,
    pub last_seen: NaiveDateTime,
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
    pub updated_at: NaiveDateTime,
}
