use chrono::NaiveDateTime;

/// Ticker data from CEX binance and coinbase
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct CexListing {
    pub id: i64,
    /// exchange that reported the token
    pub exchange: String,
    pub external_id: String,
    pub token_name: String,
    pub token_symbol: String,
    pub listing_url: String,
    pub announced_at: Option<NaiveDateTime>,
    pub detected_at: NaiveDateTime,
}
