use chrono::NaiveDateTime;

#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct Token {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub chain_id: String,
    pub first_seen: NaiveDateTime,
    pub last_seen: NaiveDateTime,
}
