pub mod client;
pub mod extract;

use chrono::Utc;
use coins_config::Config;
use coins_database::models::telegram::TelegramMessage;
use coins_database::queries::telegram::{
    self as telegram_queries, list_enabled_channels, list_seen_message_ids,
};
use coins_database::queries::{poll_timestamp, sse_event};
use sqlx::SqlitePool;

pub use client::RawMessage;
pub use extract::extract_addresses;

const TEXT_TRUNCATE_LEN: usize = 2000;

fn truncate(text: &str, max: usize) -> String {
    if text.len() > max {
        text.chars().take(max).collect()
    } else {
        text.to_string()
    }
}

pub async fn run(pool: &SqlitePool, config: &Config) -> anyhow::Result<()> {
    let now = Utc::now().naive_utc();
    let bot_token = config.telegram_bot_token();

    tracing::info!("telegram monitor cycle starting");

    let channels = list_enabled_channels(pool).await?;
    let mut total_new = 0i32;

    for ch in &channels {
        let seen_ids: std::collections::HashSet<i32> = list_seen_message_ids(pool, ch.id)
            .await?
            .into_iter()
            .collect();

        // Try public scraper first, fall back to Bot API
        let raw_messages = if !bot_token.is_empty() {
            let bot_msgs = client::fetch_bot_messages(&ch.username, &bot_token).await;
            if !bot_msgs.is_empty() {
                bot_msgs
            } else {
                client::fetch_public_messages(&ch.username).await
            }
        } else {
            client::fetch_public_messages(&ch.username).await
        };

        for msg in &raw_messages {
            if seen_ids.contains(&msg.message_id) {
                continue;
            }

            let addresses = extract_addresses(&msg.text);
            let extracted = addresses.join("\n");

            let record = TelegramMessage {
                id: 0,
                channel_id: ch.id,
                message_id: msg.message_id,
                text: truncate(&msg.text, TEXT_TRUNCATE_LEN),
                extracted_addresses: extracted,
                posted_at: msg.posted_at,
                detected_at: now,
            };

            if let Err(e) = telegram_queries::create_message(pool, &record).await {
                tracing::warn!(
                    "failed to save message {} for channel {}: {e:#}",
                    msg.message_id,
                    ch.username
                );
                continue;
            }

            if !addresses.is_empty() {
                let data = serde_json::json!({
                    "channel": ch.username,
                    "preview": msg.text.chars().take(100).collect::<String>(),
                    "addresses": addresses.len(),
                });
                if let Err(e) = sse_event::create(
                    pool,
                    "telegram_message",
                    &serde_json::to_string(&data).unwrap_or_default(),
                )
                .await
                {
                    tracing::warn!("failed to emit telegram_message SSE event: {e:#}");
                }
            }

            total_new += 1;
        }
    }

    poll_timestamp::upsert(pool, "telegram_monitor", now, total_new).await?;

    tracing::info!(
        "telegram monitor cycle finished: {total_new} new messages from {} channels",
        channels.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_text() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long_text() {
        let long = "a".repeat(3000);
        let result = truncate(&long, 2000);
        assert_eq!(result.len(), 2000);
    }
}
