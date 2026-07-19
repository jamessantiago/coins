mod util;

use coins_database::models::risk_settings::RiskSettings;
use coins_database::models::trade::TradeStatus;
use coins_database::queries::risk_settings::get_risk;
use coins_trading::{virtual_buy, virtual_sell, VirtualBuyRequest, VirtualSellRequest};

async fn seed_risk_settings(pool: &sqlx::SqlitePool, balance: f64) {
    let mut settings = RiskSettings::default();
    settings.virtual_wallet_balance = balance;
    coins_database::queries::risk_settings::upsert_risk(pool, &settings)
        .await
        .unwrap();
}

#[tokio::test]
async fn virtual_buy_creates_trade() {
    let pool = util::setup_pool().await;
    seed_risk_settings(&pool, 100.0).await;

    let trade = virtual_buy(
        &pool,
        VirtualBuyRequest {
            address: "token_mint_abc".into(),
            symbol: "TEST".into(),
            name: "Test Token".into(),
            position_size: 50.0,
            entry_price: 0.001,
            narrative: "test buy".into(),
        },
    )
    .await
    .unwrap();

    assert_eq!(trade.status, TradeStatus::VirtualBought);
    assert_eq!(trade.trade_type, "virtual");
    assert_eq!(trade.symbol, "TEST");
    assert_eq!(trade.position_size, Some(50.0));
    assert_eq!(trade.entry_price, Some(0.001));
    assert!(trade.entry_date.is_some());
    assert_eq!(trade.tx_hash, "");
}

#[tokio::test]
async fn virtual_buy_deducts_from_wallet() {
    let pool = util::setup_pool().await;
    seed_risk_settings(&pool, 100.0).await;

    virtual_buy(
        &pool,
        VirtualBuyRequest {
            address: "token_mint_def".into(),
            symbol: "TST".into(),
            name: "Test".into(),
            position_size: 30.0,
            entry_price: 0.01,
            narrative: "".into(),
        },
    )
    .await
    .unwrap();

    let settings = get_risk(&pool).await.unwrap();
    assert_eq!(settings.virtual_wallet_balance, 70.0);
    assert_eq!(settings.virtual_portfolio_value, 1030.0);
}

#[tokio::test]
async fn virtual_buy_rejects_insufficient_balance() {
    let pool = util::setup_pool().await;
    seed_risk_settings(&pool, 10.0).await;

    let err = virtual_buy(
        &pool,
        VirtualBuyRequest {
            address: "mint_xyz".into(),
            symbol: "XYZ".into(),
            name: "Xyz".into(),
            position_size: 20.0,
            entry_price: 1.0,
            narrative: "".into(),
        },
    )
    .await
    .unwrap_err();

    assert!(err.to_string().contains("insufficient virtual balance"));
}

#[tokio::test]
async fn virtual_sell_updates_trade() {
    let pool = util::setup_pool().await;
    seed_risk_settings(&pool, 200.0).await;

    let trade = virtual_buy(
        &pool,
        VirtualBuyRequest {
            address: "sell_test_mint".into(),
            symbol: "SELL".into(),
            name: "Sell Test".into(),
            position_size: 100.0,
            entry_price: 2.0,
            narrative: "".into(),
        },
    )
    .await
    .unwrap();

    let sold = virtual_sell(
        &pool,
        VirtualSellRequest {
            trade_id: trade.id,
            exit_price: 3.0,
            close_reason: Some("take profit".into()),
        },
    )
    .await
    .unwrap();

    assert_eq!(sold.status, TradeStatus::VirtualSold);
    assert_eq!(sold.exit_price, Some(3.0));
    assert_eq!(sold.close_reason.as_deref(), Some("take profit"));
    assert!(sold.exit_date.is_some());
    assert_eq!(sold.id, trade.id);
}

#[tokio::test]
async fn virtual_sell_adds_proceeds() {
    let pool = util::setup_pool().await;
    seed_risk_settings(&pool, 100.0).await;

    let trade = virtual_buy(
        &pool,
        VirtualBuyRequest {
            address: "proceeds_mint".into(),
            symbol: "PRO".into(),
            name: "Proceeds".into(),
            position_size: 50.0,
            entry_price: 0.5,
            narrative: "".into(),
        },
    )
    .await
    .unwrap();

    virtual_sell(
        &pool,
        VirtualSellRequest {
            trade_id: trade.id,
            exit_price: 1.0,
            close_reason: None,
        },
    )
    .await
    .unwrap();

    let settings = get_risk(&pool).await.unwrap();
    // proceeds = (50 / 0.5) * 1.0 = 100, wallet was 50, so 50 + 100 = 150
    assert!((settings.virtual_wallet_balance - 150.0).abs() < 0.001);
    assert!((settings.virtual_portfolio_value - 1000.0).abs() < 0.001);
}

#[tokio::test]
async fn virtual_sell_rejects_non_bought() {
    let pool = util::setup_pool().await;
    seed_risk_settings(&pool, 100.0).await;

    let trade = virtual_buy(
        &pool,
        VirtualBuyRequest {
            address: "double_sell".into(),
            symbol: "DS".into(),
            name: "Double".into(),
            position_size: 10.0,
            entry_price: 1.0,
            narrative: "".into(),
        },
    )
    .await
    .unwrap();

    virtual_sell(
        &pool,
        VirtualSellRequest {
            trade_id: trade.id,
            exit_price: 2.0,
            close_reason: None,
        },
    )
    .await
    .unwrap();

    let err = virtual_sell(
        &pool,
        VirtualSellRequest {
            trade_id: trade.id,
            exit_price: 3.0,
            close_reason: None,
        },
    )
    .await
    .unwrap_err();

    assert!(err.to_string().contains("not VirtualBought"));
}

#[tokio::test]
async fn virtual_sell_rejects_nonexistent() {
    let pool = util::setup_pool().await;
    seed_risk_settings(&pool, 100.0).await;

    let err = virtual_sell(
        &pool,
        VirtualSellRequest {
            trade_id: 999,
            exit_price: 1.0,
            close_reason: None,
        },
    )
    .await
    .unwrap_err();

    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn virtual_buy_updates_peak_value() {
    let pool = util::setup_pool().await;
    seed_risk_settings(&pool, 500.0).await;

    virtual_buy(
        &pool,
        VirtualBuyRequest {
            address: "peak_test".into(),
            symbol: "PEAK".into(),
            name: "Peak".into(),
            position_size: 200.0,
            entry_price: 1.0,
            narrative: "".into(),
        },
    )
    .await
    .unwrap();

    let settings = get_risk(&pool).await.unwrap();
    assert!((settings.virtual_peak_value - 1200.0).abs() < 0.001);
}

#[tokio::test]
async fn virtual_buy_sets_stop_loss() {
    let pool = util::setup_pool().await;
    seed_risk_settings(&pool, 100.0).await;

    let trade = virtual_buy(
        &pool,
        VirtualBuyRequest {
            address: "sl_test".into(),
            symbol: "SL".into(),
            name: "Stop Loss".into(),
            position_size: 10.0,
            entry_price: 0.1,
            narrative: "".into(),
        },
    )
    .await
    .unwrap();

    assert_eq!(trade.stop_loss_pct, Some(20.0));
    assert!(trade.stop_loss_enabled);
    assert!(trade.trailing_stop == false);
}

#[tokio::test]
async fn virtual_buy_and_sell_preserves_trade_fields() {
    let pool = util::setup_pool().await;
    seed_risk_settings(&pool, 100.0).await;

    let trade = virtual_buy(
        &pool,
        VirtualBuyRequest {
            address: "field_test".into(),
            symbol: "FLD".into(),
            name: "Fields".into(),
            position_size: 25.0,
            entry_price: 0.5,
            narrative: "narrative_test".into(),
        },
    )
    .await
    .unwrap();

    let sold = virtual_sell(
        &pool,
        VirtualSellRequest {
            trade_id: trade.id,
            exit_price: 1.0,
            close_reason: Some("goal".into()),
        },
    )
    .await
    .unwrap();

    assert_eq!(sold.address, "field_test");
    assert_eq!(sold.symbol, "FLD");
    assert_eq!(sold.narrative, "narrative_test");
    assert_eq!(sold.entry_price, Some(0.5));
    assert_eq!(sold.position_size, Some(25.0));
    assert_eq!(sold.stop_loss_pct, Some(20.0));
}
