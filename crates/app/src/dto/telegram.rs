use serde::Serialize;
use coins_database::models::telegram::{TelegramChannel, TelegramMessage};

use crate::dto::common::fmt_ts;

#[derive(Serialize)]
pub struct TelegramChannelResponse {
    pub id: i64,
    pub username: String,
    pub enabled: bool,
    pub chat_id: Option<i64>,
    pub added_at: String,
}

impl From<TelegramChannel> for TelegramChannelResponse {
    fn from(c: TelegramChannel) -> Self {
        Self {
            id: c.id,
            username: c.username,
            enabled: c.enabled,
            chat_id: c.chat_id,
            added_at: fmt_ts(c.added_at),
        }
    }
}

#[derive(Serialize)]
pub struct TelegramMessageResponse {
    pub id: i64,
    pub channel_id: i64,
    pub message_id: i32,
    pub text: String,
    pub extracted_addresses: String,
    pub posted_at: String,
    pub detected_at: String,
}

impl From<TelegramMessage> for TelegramMessageResponse {
    fn from(m: TelegramMessage) -> Self {
        Self {
            id: m.id,
            channel_id: m.channel_id,
            message_id: m.message_id,
            text: m.text,
            extracted_addresses: m.extracted_addresses,
            posted_at: fmt_ts(m.posted_at),
            detected_at: fmt_ts(m.detected_at),
        }
    }
}
