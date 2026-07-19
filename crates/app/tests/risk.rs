mod util;

use coins_app::risk;
use coins_database::models::risk_settings::TradingMode;

#[tokio::test]
async fn get_current_returns_defaults() {
    let pool = util::test_pool().await;
    let settings = risk::get_current(&pool).await.unwrap();
    assert_eq!(settings.peak_value, 1000.0);
    assert_eq!(settings.virtual_wallet_balance, 1000.0);
    assert_eq!(settings.trading_mode, TradingMode::Virtual);
    assert_eq!(settings.max_positions, 8);
}

#[tokio::test]
async fn upsert_and_get_roundtrip() {
    let pool = util::test_pool().await;
    let mut settings = risk::get_current(&pool).await.unwrap();
    settings.peak_value = 5000.0;
    settings.trading_mode = TradingMode::Real;
    settings.max_positions = 12;
    risk::upsert(&pool, &settings).await.unwrap();

    let loaded = risk::get_current(&pool).await.unwrap();
    assert_eq!(loaded.peak_value, 5000.0);
    assert_eq!(loaded.trading_mode, TradingMode::Real);
    assert_eq!(loaded.max_positions, 12);
}

#[tokio::test]
async fn reset_drawdown_virtual_mode() {
    let pool = util::test_pool().await;
    let mut settings = risk::get_current(&pool).await.unwrap();
    settings.virtual_portfolio_value = 800.0;
    settings.virtual_peak_value = 1000.0;
    risk::upsert(&pool, &settings).await.unwrap();

    let updated = risk::reset_drawdown(&pool).await.unwrap();
    assert_eq!(updated.virtual_peak_value, 800.0);
}

#[tokio::test]
async fn reset_drawdown_real_mode() {
    let pool = util::test_pool().await;
    let mut settings = risk::get_current(&pool).await.unwrap();
    settings.trading_mode = TradingMode::Real;
    settings.real_portfolio_value = 500.0;
    settings.real_peak_value = 1000.0;
    risk::upsert(&pool, &settings).await.unwrap();

    let updated = risk::reset_drawdown(&pool).await.unwrap();
    assert_eq!(updated.real_peak_value, 500.0);
}

#[tokio::test]
async fn add_funds_increases_balance() {
    let pool = util::test_pool().await;
    let updated = risk::add_funds(&pool, 500.0).await.unwrap();
    assert_eq!(updated.virtual_wallet_balance, 1500.0);

    let updated = risk::add_funds(&pool, 250.0).await.unwrap();
    assert_eq!(updated.virtual_wallet_balance, 1750.0);
}

#[tokio::test]
async fn get_current_is_idempotent() {
    let pool = util::test_pool().await;
    let a = risk::get_current(&pool).await.unwrap();
    let b = risk::get_current(&pool).await.unwrap();
    assert_eq!(a.peak_value, b.peak_value);
    assert_eq!(a.trading_mode, b.trading_mode);
}
