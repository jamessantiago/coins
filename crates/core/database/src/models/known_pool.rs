use chrono::NaiveDateTime;

#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct KnownPool {
    pub pool_address: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub symbol: String,
    pub name: String,
    pub first_seen: NaiveDateTime,
    pub last_seen: NaiveDateTime,
}
