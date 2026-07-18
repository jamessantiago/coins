mod util;

use coins_database::{RiskSettings, TradingMode, get_risk, upsert_risk};
use util::setup_memory_pool;

#[tokio::test]
async fn get_risk_creates_default_row() {
    let pool = setup_memory_pool().await;
    let risk = get_risk(&pool).await.unwrap();

    assert_eq!(risk.trading_mode, TradingMode::Virtual);
    assert_eq!(risk.peak_value, 1000.0);
    assert_eq!(risk.max_positions, 8);
    assert_eq!(risk.default_position_pct, 2.0);
}

#[tokio::test]
async fn upsert_risk_persists_values() {
    let pool = setup_memory_pool().await;

    let mut settings = RiskSettings::default();
    settings.peak_value = 999.0;
    settings.real_portfolio_value = 500.0;
    settings.trading_mode = TradingMode::Real;
    settings.max_positions = 4;
    upsert_risk(&pool, &settings).await.unwrap();

    let fetched = get_risk(&pool).await.unwrap();
    assert_eq!(fetched.peak_value, 999.0);
    assert_eq!(fetched.real_portfolio_value, 500.0);
    assert_eq!(fetched.trading_mode, TradingMode::Real);
    assert_eq!(fetched.max_positions, 4);
}

#[tokio::test]
async fn upsert_risk_overwrites_existing() {
    let pool = setup_memory_pool().await;

    let mut v1 = RiskSettings::default();
    v1.peak_value = 100.0;
    upsert_risk(&pool, &v1).await.unwrap();

    let mut v2 = RiskSettings::default();
    v2.peak_value = 200.0;
    upsert_risk(&pool, &v2).await.unwrap();

    let fetched = get_risk(&pool).await.unwrap();
    assert_eq!(fetched.peak_value, 200.0);
    assert_eq!(fetched.real_peak_value, 0.0);
}

#[tokio::test]
async fn get_risk_is_idempotent() {
    let pool = setup_memory_pool().await;

    let a = get_risk(&pool).await.unwrap();
    let b = get_risk(&pool).await.unwrap();

    assert_eq!(a.peak_value, b.peak_value);
}
