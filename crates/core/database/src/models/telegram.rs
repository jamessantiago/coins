use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TelegramChannel {
    pub id: i64,
    pub username: String,
    pub enabled: bool,
    pub chat_id: Option<i64>,
    pub added_at: NaiveDateTime,
}

impl Default for TelegramChannel {
    fn default() -> Self {
        Self {
            id: i64::default(),
            username: String::default(),
            enabled: true,
            chat_id: None,
            added_at: NaiveDateTime::default(),
        }
    }
}

#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct TelegramMessage {
    pub id: i64,
    pub channel_id: i64,
    pub message_id: i32,
    pub text: String,
    pub extracted_addresses: String,
    pub posted_at: NaiveDateTime,
    pub detected_at: NaiveDateTime,
}
