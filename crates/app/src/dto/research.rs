use serde::Serialize;
use coins_database::models::research_entry::ResearchEntry;

use crate::dto::common::fmt_ts;

#[derive(Serialize)]
pub struct ResearchEntryResponse {
    pub id: i64,
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub notes: String,
    pub conviction: i32,
    pub safety_score: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ResearchEntry> for ResearchEntryResponse {
    fn from(r: ResearchEntry) -> Self {
        Self {
            id: r.id,
            address: r.address,
            symbol: r.symbol,
            name: r.name,
            notes: r.notes,
            conviction: r.conviction,
            safety_score: r.safety_score,
            created_at: fmt_ts(r.created_at),
            updated_at: fmt_ts(r.updated_at),
        }
    }
}
