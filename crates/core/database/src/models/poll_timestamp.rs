use chrono::NaiveDateTime;

#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct PollTimestamp {
    pub service: String,
    pub last_run_at: NaiveDateTime,
    pub listings_found: i32,
}
