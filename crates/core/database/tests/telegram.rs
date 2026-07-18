mod util;

use chrono::NaiveDateTime;
use coins_database::TelegramMessage;
use coins_database::queries::telegram;
use util::setup_memory_pool;

#[tokio::test]
async fn get_or_create_channel_creates_new() {
    let pool = setup_memory_pool().await;
    let (ch, created) = telegram::get_or_create_channel(&pool, "test_channel")
        .await
        .unwrap();
    assert!(created);
    assert_eq!(ch.username, "test_channel");
    assert!(ch.enabled);
}

#[tokio::test]
async fn get_or_create_channel_returns_existing() {
    let pool = setup_memory_pool().await;
    telegram::get_or_create_channel(&pool, "existing")
        .await
        .unwrap();
    let (ch, created) = telegram::get_or_create_channel(&pool, "existing")
        .await
        .unwrap();
    assert!(!created);
    assert_eq!(ch.username, "existing");
}

#[tokio::test]
async fn list_enabled_channels() {
    let pool = setup_memory_pool().await;
    telegram::get_or_create_channel(&pool, "ch1").await.unwrap();
    telegram::get_or_create_channel(&pool, "ch2").await.unwrap();

    let channels = telegram::list_enabled_channels(&pool).await.unwrap();
    assert_eq!(channels.len(), 2);
}

#[tokio::test]
async fn toggle_channel_enabled_flips() {
    let pool = setup_memory_pool().await;
    let (ch, _) = telegram::get_or_create_channel(&pool, "tog").await.unwrap();

    telegram::toggle_channel_enabled(&pool, ch.id)
        .await
        .unwrap();
    let toggled = telegram::get_channel_by_id(&pool, ch.id)
        .await
        .unwrap()
        .unwrap();
    assert!(!toggled.enabled);

    telegram::toggle_channel_enabled(&pool, ch.id)
        .await
        .unwrap();
    let toggled_back = telegram::get_channel_by_id(&pool, ch.id)
        .await
        .unwrap()
        .unwrap();
    assert!(toggled_back.enabled);
}

#[tokio::test]
async fn delete_channel_removes() {
    let pool = setup_memory_pool().await;
    let (ch, _) = telegram::get_or_create_channel(&pool, "del").await.unwrap();
    assert!(telegram::delete_channel(&pool, ch.id).await.unwrap());
    assert!(
        telegram::get_channel_by_id(&pool, ch.id)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn channel_count_returns_count() {
    let pool = setup_memory_pool().await;
    telegram::get_or_create_channel(&pool, "c1").await.unwrap();
    telegram::get_or_create_channel(&pool, "c2").await.unwrap();
    assert_eq!(telegram::channel_count(&pool).await.unwrap(), 2);
}

#[tokio::test]
async fn message_lifecycle() {
    let pool = setup_memory_pool().await;
    let (ch, _) = telegram::get_or_create_channel(&pool, "msg_ch")
        .await
        .unwrap();

    let msg = TelegramMessage {
        id: 0,
        channel_id: ch.id,
        message_id: 100,
        text: "hello".into(),
        extracted_addresses: "addr1".into(),
        posted_at: NaiveDateTime::default(),
        detected_at: NaiveDateTime::default(),
    };
    let created = telegram::create_message(&pool, &msg).await.unwrap();
    assert!(created.id > 0);

    let seen = telegram::list_seen_message_ids(&pool, ch.id).await.unwrap();
    assert_eq!(seen, vec![100]);

    let recent = telegram::list_recent_messages(&pool, 10).await.unwrap();
    assert_eq!(recent.len(), 1);

    let with_addrs = telegram::iterate_messages_with_addresses(&pool)
        .await
        .unwrap();
    assert_eq!(with_addrs.len(), 1);

    assert!(telegram::delete_message(&pool, created.id).await.unwrap());
    assert_eq!(telegram::message_count(&pool).await.unwrap(), 0);
}
