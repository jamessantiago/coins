use chrono::Utc;
use sqlx::SqlitePool;

use coins_database::models::risk_settings::{RiskSettings, TradingMode};
use coins_database::queries::risk_settings::{get_risk, upsert_risk};

pub async fn get_current(pool: &SqlitePool) -> anyhow::Result<RiskSettings> {
    get_risk(pool).await
}

pub async fn upsert(pool: &SqlitePool, settings: &RiskSettings) -> anyhow::Result<()> {
    upsert_risk(pool, settings).await
}

pub async fn reset_drawdown(pool: &SqlitePool) -> anyhow::Result<RiskSettings> {
    let mut settings = get_risk(pool).await?;
    match settings.trading_mode {
        TradingMode::Virtual => settings.virtual_peak_value = settings.virtual_portfolio_value,
        TradingMode::Real => settings.real_peak_value = settings.real_portfolio_value,
    }
    settings.updated_at = Utc::now().naive_utc();
    upsert_risk(pool, &settings).await?;
    Ok(settings)
}

pub async fn add_funds(pool: &SqlitePool, amount: f64) -> anyhow::Result<RiskSettings> {
    let mut settings = get_risk(pool).await?;
    settings.virtual_wallet_balance += amount;
    settings.updated_at = Utc::now().naive_utc();
    upsert_risk(pool, &settings).await?;
    Ok(settings)
}
