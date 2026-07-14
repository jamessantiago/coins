use chrono::NaiveDateTime;

#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct SseEvent {
    pub id: i64,
    pub event: String,
    pub data: String,
    pub created_at: NaiveDateTime,
}
