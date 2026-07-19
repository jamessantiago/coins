use coins_database::models::cex_listing::CexListing;
use serde::Serialize;

use crate::dto::common::{fmt_ts, fmt_ts_opt};

#[derive(Serialize)]
pub struct CexListingResponse {
    pub id: i64,
    pub exchange: String,
    pub external_id: String,
    pub token_name: String,
    pub token_symbol: String,
    pub listing_url: String,
    pub announced_at: Option<String>,
    pub detected_at: String,
}

impl From<CexListing> for CexListingResponse {
    fn from(l: CexListing) -> Self {
        Self {
            id: l.id,
            exchange: l.exchange,
            external_id: l.external_id,
            token_name: l.token_name,
            token_symbol: l.token_symbol,
            listing_url: l.listing_url,
            announced_at: fmt_ts_opt(l.announced_at),
            detected_at: fmt_ts(l.detected_at),
        }
    }
}
