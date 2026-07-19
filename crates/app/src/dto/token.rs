use coins_database::models::token::Token;
use serde::Serialize;

use crate::dto::common::fmt_ts;

#[derive(Serialize)]
pub struct TokenResponse {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub chain_id: String,
    pub first_seen: String,
    pub last_seen: String,
}

impl From<Token> for TokenResponse {
    fn from(t: Token) -> Self {
        Self {
            address: t.address,
            symbol: t.symbol,
            name: t.name,
            chain_id: t.chain_id,
            first_seen: fmt_ts(t.first_seen),
            last_seen: fmt_ts(t.last_seen),
        }
    }
}
