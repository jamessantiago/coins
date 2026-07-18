mod util;

use chrono::Utc;
use coins_database::queries::telegram::{self, get_or_create_channel, list_enabled_channels};
use coins_database::queries::{poll_timestamp, sse_event};
use serial_test::serial;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PUBLIC_HTML: &str = r#"
<html>
<body>
<div class="tgme_widget_message_wrap js-widget_message_wrap" data-post="testchannel/101">
    <div class="tgme_widget_message">
        <div class="tgme_widget_message_text">gm frens H8hJ9jKkMmNnPpQqRrSsTtUuVvWwXxYyZzA1aB2bC3cD</div>
        <time class="datetime" datetime="2025-07-18T12:00:00Z"></time>
    </div>
</div>
<div class="tgme_widget_message_wrap js-widget_message_wrap" data-post="testchannel/102">
    <div class="tgme_widget_message">
        <div class="tgme_widget_message_text">no address here</div>
        <time class="datetime" datetime="2025-07-18T12:01:00Z"></time>
    </div>
</div>
</body>
</html>
"#;

#[tokio::test]
#[serial]
async fn test_telegram_full_pipeline() {
    let pool = util::setup_memory_pool().await;
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/testchannel"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PUBLIC_HTML))
        .mount(&mock_server)
        .await;

    // Seed a channel in the DB
    let (ch, created) = get_or_create_channel(&pool, "testchannel")
        .await
        .expect("get_or_create failed");
    assert!(created);

    // Run the poll via the public scraper with mock server URL
    let raw =
        coins_telegram::client::fetch_public_messages_from("testchannel", &mock_server.uri()).await;
    assert_eq!(raw.len(), 2);

    let now = Utc::now().naive_utc();
    for msg in &raw {
        let addresses = coins_telegram::extract_addresses(&msg.text);
        let extracted = addresses.join("\n");
        let record = coins_database::models::telegram::TelegramMessage {
            id: 0,
            channel_id: ch.id,
            message_id: msg.message_id,
            text: msg.text.clone(),
            extracted_addresses: extracted,
            posted_at: msg.posted_at,
            detected_at: now,
        };
        telegram::create_message(&pool, &record)
            .await
            .expect("create_message failed");
    }

    // Verify messages were stored
    let msgs = telegram::list_recent_messages(&pool, 10)
        .await
        .expect("list_recent failed");
    assert_eq!(msgs.len(), 2);

    // Verify address extraction
    assert!(msgs[0].message_id == 101 || msgs[1].message_id == 101);
    let msg_101 = msgs.iter().find(|m| m.message_id == 101).unwrap();
    assert!(!msg_101.extracted_addresses.is_empty());
    assert!(
        msg_101
            .extracted_addresses
            .contains("H8hJ9jKkMmNnPpQqRrSsTtUuVvWwXxYyZzA1aB2bC3cD")
    );

    let msg_102 = msgs.iter().find(|m| m.message_id == 102).unwrap();
    assert!(msg_102.extracted_addresses.is_empty());
}

#[tokio::test]
#[serial]
async fn test_telegram_dedup() {
    let pool = util::setup_memory_pool().await;
    let (ch, _) = get_or_create_channel(&pool, "testchannel")
        .await
        .expect("get_or_create failed");

    let now = Utc::now().naive_utc();

    // Insert a message that would be duplicated
    let msg = coins_database::models::telegram::TelegramMessage {
        id: 0,
        channel_id: ch.id,
        message_id: 101,
        text: "first".to_string(),
        extracted_addresses: String::new(),
        posted_at: now,
        detected_at: now,
    };
    telegram::create_message(&pool, &msg)
        .await
        .expect("create_message failed");

    // Query seen IDs
    let seen = telegram::list_seen_message_ids(&pool, ch.id)
        .await
        .expect("list_seen failed");
    assert!(seen.contains(&101));

    // Simulate running the poll again and checking seen_ids
    let raw = vec![coins_telegram::RawMessage {
        message_id: 101,
        text: "first".to_string(),
        posted_at: now,
    }];

    let seen_set: std::collections::HashSet<i32> = seen.into_iter().collect();
    let new: Vec<_> = raw
        .into_iter()
        .filter(|m| !seen_set.contains(&m.message_id))
        .collect();
    assert!(new.is_empty());
}

#[tokio::test]
#[serial]
async fn test_telegram_api_error_does_not_crash() {
    let _pool = util::setup_memory_pool().await;
    let mock_server = MockServer::start().await;

    // Return 500
    Mock::given(method("GET"))
        .and(path("/testchannel"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let msgs =
        coins_telegram::client::fetch_public_messages_from("testchannel", &mock_server.uri()).await;
    assert!(msgs.is_empty());
}

#[tokio::test]
#[serial]
async fn test_telegram_extract_no_addresses() {
    let text = "Just a regular message without any addresses";
    let addrs = coins_telegram::extract_addresses(text);
    assert!(addrs.is_empty());
}

#[tokio::test]
#[serial]
async fn test_telegram_sse_events_emitted() {
    let pool = util::setup_memory_pool().await;
    let (_ch, _) = get_or_create_channel(&pool, "testchannel")
        .await
        .expect("get_or_create failed");

    let _now = Utc::now().naive_utc();
    let data = serde_json::json!({
        "channel": "testchannel",
        "preview": "gm frens",
        "addresses": 1,
    });
    sse_event::create(
        &pool,
        "telegram_message",
        &serde_json::to_string(&data).unwrap_or_default(),
    )
    .await
    .expect("sse_event::create failed");

    let events = sse_event::read_since(&pool, 0)
        .await
        .expect("read_since failed");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event, "telegram_message");
}

#[tokio::test]
#[serial]
async fn test_telegram_poll_timestamp_updated() {
    let pool = util::setup_memory_pool().await;

    let now = Utc::now().naive_utc();
    poll_timestamp::upsert(&pool, "telegram_monitor", now, 5)
        .await
        .expect("upsert failed");

    let ts = poll_timestamp::get_by_service(&pool, "telegram_monitor")
        .await
        .expect("get_by_service failed")
        .expect("expected poll_timestamp");
    assert_eq!(ts.service, "telegram_monitor");
    assert_eq!(ts.listings_found, 5);
}

#[tokio::test]
#[serial]
async fn test_telegram_enabled_channels_only() {
    let pool = util::setup_memory_pool().await;

    let (ch1, _) = get_or_create_channel(&pool, "enabled_channel")
        .await
        .expect("get_or_create failed");
    let (ch2, _) = get_or_create_channel(&pool, "disabled_channel")
        .await
        .expect("get_or_create failed");

    // Disable ch2
    telegram::toggle_channel_enabled(&pool, ch2.id)
        .await
        .expect("toggle failed");

    let enabled = list_enabled_channels(&pool)
        .await
        .expect("list_enabled failed");
    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].id, ch1.id);
}
