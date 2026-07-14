use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResearchEntry {
    pub id: i64,
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub notes: String,
    pub conviction: i32,
    pub safety_score: Option<f64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for ResearchEntry {
    fn default() -> Self {
        Self {
            id: i64::default(),
            address: String::default(),
            symbol: String::default(),
            name: String::default(),
            notes: String::default(),
            conviction: 3,
            safety_score: None,
            created_at: NaiveDateTime::default(),
            updated_at: NaiveDateTime::default(),
        }
    }
}
