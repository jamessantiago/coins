use chrono::{DateTime, NaiveDateTime};
use coins_config::http_client;

#[derive(Debug, Clone)]
pub struct RawMessage {
    pub message_id: i32,
    pub text: String,
    pub posted_at: NaiveDateTime,
}

const TME_BASE: &str = "https://t.me/s";

pub async fn fetch_public_messages(channel_username: &str) -> Vec<RawMessage> {
    fetch_public_messages_from(channel_username, TME_BASE).await
}

pub async fn fetch_public_messages_from(channel_username: &str, base_url: &str) -> Vec<RawMessage> {
    let client = match http_client(15) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("failed to build http client: {e:#}");
            return vec![];
        }
    };

    let url = format!("{base_url}/{channel_username}");
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            tracing::warn!(
                "t.me/s/ returned HTTP {} for {channel_username}",
                r.status()
            );
            return vec![];
        }
        Err(e) => {
            tracing::warn!("failed to fetch t.me/s/{channel_username}: {e:#}");
            return vec![];
        }
    };

    let html = match resp.text().await {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!("failed to read response body: {e:#}");
            return vec![];
        }
    };

    let messages = parse_public_page(&html);
    tracing::info!(
        "scraped {} messages from t.me/s/{channel_username}",
        messages.len()
    );
    messages
}

fn parse_public_page(html: &str) -> Vec<RawMessage> {
    let document = scraper::Html::parse_document(html);

    let message_sel =
        scraper::Selector::parse("div.tgme_widget_message_wrap.js-widget_message_wrap").unwrap();

    let text_sel = scraper::Selector::parse("div.tgme_widget_message_text").unwrap();
    let time_sel = scraper::Selector::parse("time.datetime").unwrap();

    let mut messages = Vec::new();

    for wrap in document.select(&message_sel) {
        let post_id = wrap.value().attr("data-post").unwrap_or("").to_string();

        let message_id = match post_id.rsplit_once('/') {
            Some((_, id_str)) => id_str.parse::<i32>().unwrap_or(0),
            None => 0,
        };

        if message_id == 0 {
            continue;
        }

        let posted_at = wrap
            .select(&time_sel)
            .next()
            .and_then(|el| el.value().attr("datetime"))
            .and_then(|dt| NaiveDateTime::parse_from_str(dt, "%Y-%m-%dT%H:%M:%S%.fZ").ok())
            .unwrap_or_default();

        let text = wrap
            .select(&text_sel)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" "))
            .unwrap_or_default()
            .trim()
            .to_string();

        if text.is_empty() {
            continue;
        }

        messages.push(RawMessage {
            message_id,
            text,
            posted_at,
        });
    }

    messages
}

pub async fn fetch_bot_messages(channel_username: &str, bot_token: &str) -> Vec<RawMessage> {
    let client = match http_client(15) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("failed to build http client: {e:#}");
            return vec![];
        }
    };

    let get_updates_url = format!("https://api.telegram.org/bot{bot_token}/getUpdates?timeout=10");

    let resp = match client.get(&get_updates_url).send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            tracing::warn!(
                "Telegram Bot API returned HTTP {} for {channel_username}",
                r.status()
            );
            return vec![];
        }
        Err(e) => {
            tracing::warn!("Telegram Bot API request failed for {channel_username}: {e:#}");
            return vec![];
        }
    };

    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("failed to parse Bot API response: {e:#}");
            return vec![];
        }
    };

    let result = match body.get("result").and_then(|r| r.as_array()) {
        Some(arr) => arr,
        None => return vec![],
    };

    let channel_username_lower = channel_username.to_lowercase();
    let mut messages = Vec::new();

    for update in result {
        let msg = match update.get("channel_post").or_else(|| update.get("message")) {
            Some(m) => m,
            None => continue,
        };

        let chat = match msg.get("chat") {
            Some(c) => c,
            None => continue,
        };

        let chat_username = match chat.get("username").and_then(|u| u.as_str()) {
            Some(u) => u.to_lowercase(),
            None => continue,
        };

        if chat_username != channel_username_lower {
            continue;
        }

        let message_id = match msg.get("message_id").and_then(|m| m.as_i64()) {
            Some(id) => id as i32,
            None => continue,
        };

        let text = msg
            .get("text")
            .or_else(|| msg.get("caption"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        if text.is_empty() {
            continue;
        }

        let posted_at = msg
            .get("date")
            .and_then(|d| d.as_i64())
            .and_then(|ts| DateTime::from_timestamp(ts, 0).map(|dt| dt.naive_utc()))
            .unwrap_or_default();

        messages.push(RawMessage {
            message_id,
            text,
            posted_at,
        });
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_public_page_extracts_messages() {
        let html = r#"
        <html>
        <body>
        <div class="tgme_widget_message_wrap js-widget_message_wrap" data-post="testchannel/123">
            <div class="tgme_widget_message">
                <div class="tgme_widget_message_text">Hello world</div>
                <time class="datetime" datetime="2024-01-15T10:30:00Z"></time>
            </div>
        </div>
        <div class="tgme_widget_message_wrap js-widget_message_wrap" data-post="testchannel/456">
            <div class="tgme_widget_message">
                <div class="tgme_widget_message_text">Another message with Aa1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u</div>
                <time class="datetime" datetime="2024-01-15T10:31:00Z"></time>
            </div>
        </div>
        </body>
        </html>
        "#;

        let messages = parse_public_page(html);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].message_id, 123);
        assert_eq!(messages[0].text, "Hello world");
        assert_eq!(messages[0].posted_at.to_string(), "2024-01-15 10:30:00");
        assert_eq!(messages[1].message_id, 456);
    }

    #[test]
    fn test_parse_public_page_skips_empty_text() {
        let html = r#"
        <html>
        <div class="tgme_widget_message_wrap js-widget_message_wrap" data-post="ch/1">
            <div class="tgme_widget_message">
                <div class="tgme_widget_message_text"></div>
                <time class="datetime" datetime="2024-01-15T10:30:00Z"></time>
            </div>
        </div>
        </html>
        "#;

        let messages = parse_public_page(html);
        assert!(messages.is_empty());
    }

    #[test]
    fn test_parse_public_page_skips_bad_post_id() {
        let html = r#"
        <html>
        <div class="tgme_widget_message_wrap js-widget_message_wrap" data-post="">
            <div class="tgme_widget_message">
                <div class="tgme_widget_message_text">text</div>
                <time class="datetime" datetime="2024-01-15T10:30:00Z"></time>
            </div>
        </div>
        </html>
        "#;

        let messages = parse_public_page(html);
        assert!(messages.is_empty());
    }
}
