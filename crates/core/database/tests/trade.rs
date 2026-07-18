mod util;

use chrono::NaiveDateTime;
use coins_database::queries::trade;
use coins_database::{Trade, TradeStatus};
use util::setup_memory_pool;

fn sample_trade(address: &str, status: TradeStatus) -> Trade {
    Trade {
        id: 0,
        address: address.into(),
        symbol: "".into(),
        name: "".into(),
        status,
        trade_type: "virtual".into(),
        entry_price: None,
        entry_date: None,
        position_size: None,
        exit_price: None,
        exit_date: None,
        notes: "".into(),
        stop_loss_pct: None,
        stop_price: None,
        trailing_stop: false,
        peak_price: None,
        stop_loss_enabled: true,
        take_profit_enabled: false,
        take_profit_multiplier: None,
        peak_decay_enabled: false,
        peak_decay_pct: None,
        volume_exhaustion_enabled: false,
        volume_exhaustion_pct: None,
        peak_volume_24h: None,
        close_reason: None,
        tx_hash: "".into(),
        narrative: "".into(),
        pump_graduated: false,
        created_at: NaiveDateTime::default(),
        updated_at: NaiveDateTime::default(),
    }
}

#[tokio::test]
async fn create_and_get_by_id() {
    let pool = setup_memory_pool().await;
    let t = sample_trade("addr1", TradeStatus::Watching);
    let created = trade::create(&pool, &t).await.unwrap();
    assert!(created.id > 0);
    assert_eq!(created.address, "addr1");

    let fetched = trade::get_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(fetched.address, "addr1");
    assert_eq!(fetched.status, TradeStatus::Watching);
}

#[tokio::test]
async fn update_modifies_trade() {
    let pool = setup_memory_pool().await;
    let t = sample_trade("addr_up", TradeStatus::Watching);
    let created = trade::create(&pool, &t).await.unwrap();

    let mut updated = created.clone();
    updated.status = TradeStatus::Bought;
    updated.entry_price = Some(100.0);
    trade::update(&pool, &updated).await.unwrap();

    let fetched = trade::get_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(fetched.status, TradeStatus::Bought);
    assert_eq!(fetched.entry_price, Some(100.0));
}

#[tokio::test]
async fn list_open_returns_bought_and_virtual_bought() {
    let pool = setup_memory_pool().await;
    trade::create(&pool, &sample_trade("watching", TradeStatus::Watching))
        .await
        .unwrap();
    trade::create(&pool, &sample_trade("bought", TradeStatus::Bought))
        .await
        .unwrap();
    trade::create(&pool, &sample_trade("vbought", TradeStatus::VirtualBought))
        .await
        .unwrap();
    trade::create(&pool, &sample_trade("sold", TradeStatus::Sold))
        .await
        .unwrap();

    let open = trade::list_open(&pool).await.unwrap();
    assert_eq!(open.len(), 2);
    let addresses: Vec<_> = open.iter().map(|t| t.address.as_str()).collect();
    assert!(addresses.contains(&"bought"));
    assert!(addresses.contains(&"vbought"));
}

#[tokio::test]
async fn open_position_value_sums() {
    let pool = setup_memory_pool().await;
    let mut t = sample_trade("pos1", TradeStatus::Bought);
    t.trade_type = "virtual".into();
    t.position_size = Some(50.0);
    trade::create(&pool, &t).await.unwrap();

    let mut t2 = sample_trade("pos2", TradeStatus::VirtualBought);
    t2.trade_type = "virtual".into();
    t2.position_size = Some(30.0);
    trade::create(&pool, &t2).await.unwrap();

    let val = trade::open_position_value(&pool, "virtual").await.unwrap();
    assert_eq!(val, 80.0);
}

#[tokio::test]
async fn count_by_status_counts_correctly() {
    let pool = setup_memory_pool().await;
    trade::create(&pool, &sample_trade("a", TradeStatus::Bought))
        .await
        .unwrap();
    trade::create(&pool, &sample_trade("b", TradeStatus::Bought))
        .await
        .unwrap();
    trade::create(&pool, &sample_trade("c", TradeStatus::Sold))
        .await
        .unwrap();

    assert_eq!(
        trade::count_by_status(&pool, &TradeStatus::Bought)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        trade::count_by_status(&pool, &TradeStatus::Sold)
            .await
            .unwrap(),
        1
    );
}

#[tokio::test]
async fn list_by_statuses_filters() {
    let pool = setup_memory_pool().await;
    trade::create(&pool, &sample_trade("a", TradeStatus::Bought))
        .await
        .unwrap();
    trade::create(&pool, &sample_trade("b", TradeStatus::Sold))
        .await
        .unwrap();
    trade::create(&pool, &sample_trade("c", TradeStatus::VirtualBought))
        .await
        .unwrap();

    let results = trade::list_by_statuses(&pool, &[TradeStatus::Bought, TradeStatus::Sold])
        .await
        .unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn count_open_real() {
    let pool = setup_memory_pool().await;
    let mut t1 = sample_trade("real1", TradeStatus::Bought);
    t1.trade_type = "real".into();
    trade::create(&pool, &t1).await.unwrap();

    trade::create(&pool, &sample_trade("virt1", TradeStatus::Bought))
        .await
        .unwrap();

    assert_eq!(trade::count_open_real(&pool).await.unwrap(), 1);
}

#[tokio::test]
async fn delete_by_id_removes() {
    let pool = setup_memory_pool().await;
    let t = trade::create(&pool, &sample_trade("del", TradeStatus::Watching))
        .await
        .unwrap();
    assert!(trade::delete_by_id(&pool, t.id).await.unwrap());
    assert!(trade::get_by_id(&pool, t.id).await.unwrap().is_none());
}

#[tokio::test]
async fn list_open_addresses() {
    let pool = setup_memory_pool().await;
    trade::create(&pool, &sample_trade("open1", TradeStatus::Bought))
        .await
        .unwrap();
    trade::create(&pool, &sample_trade("open2", TradeStatus::VirtualBought))
        .await
        .unwrap();
    trade::create(&pool, &sample_trade("closed", TradeStatus::Sold))
        .await
        .unwrap();

    let addrs = trade::list_open_addresses(&pool).await.unwrap();
    assert_eq!(addrs.len(), 2);
}

#[tokio::test]
async fn list_pnl_trades_filters() {
    let pool = setup_memory_pool().await;
    let mut t1 = sample_trade("won", TradeStatus::Sold);
    t1.trade_type = "real".into();
    t1.entry_price = Some(10.0);
    t1.exit_price = Some(20.0);
    t1.position_size = Some(1.0);
    trade::create(&pool, &t1).await.unwrap();

    trade::create(&pool, &sample_trade("no_pnl", TradeStatus::Sold))
        .await
        .unwrap();

    let pnl = trade::list_pnl_trades(&pool, "real").await.unwrap();
    assert_eq!(pnl.len(), 1);
    assert_eq!(pnl[0].address, "won");
}

#[tokio::test]
async fn list_ungraduated_open() {
    let pool = setup_memory_pool().await;
    let mut t1 = sample_trade("ungrad", TradeStatus::Bought);
    t1.pump_graduated = false;
    trade::create(&pool, &t1).await.unwrap();

    let mut t2 = sample_trade("grad", TradeStatus::Bought);
    t2.pump_graduated = true;
    trade::create(&pool, &t2).await.unwrap();

    let ungrad = trade::list_ungraduated_open(&pool).await.unwrap();
    assert_eq!(ungrad.len(), 1);
    assert_eq!(ungrad[0].address, "ungrad");
}
