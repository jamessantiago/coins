use sqlx::SqlitePool;

use coins_database::models::risk_settings::{RiskSettings, TradingMode};
use coins_database::models::trade::TradeStatus;
use coins_database::queries::risk_settings::get_risk;
use coins_database::queries::trade;
use coins_trading::SafetyCheck;
use coins_trading::safety::SafetyOutcome;

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct CheckResult {
    pub name: &'static str,
    pub passed: bool,
    pub message: Option<String>,
}

fn compute_drawdown(settings: &RiskSettings) -> f64 {
    let (peak, current) = match settings.trading_mode {
        TradingMode::Virtual => (
            settings.virtual_peak_value,
            settings.virtual_portfolio_value,
        ),
        TradingMode::Real => (settings.real_peak_value, settings.real_portfolio_value),
    };
    if peak <= 0.0 {
        return 0.0;
    }
    ((peak - current) / peak * 100.0).max(0.0)
}

async fn load_check(pool: &SqlitePool) -> anyhow::Result<SafetyCheck> {
    let settings = get_risk(pool).await?;
    let open_count = trade::count_by_status(pool, &TradeStatus::Bought).await?
        + trade::count_by_status(pool, &TradeStatus::VirtualBought).await?;
    let open_value = trade::open_position_value(pool, "real").await?
        + trade::open_position_value(pool, "virtual").await?;
    let drawdown = compute_drawdown(&settings);
    let portfolio_value = match settings.trading_mode {
        TradingMode::Virtual => settings.virtual_portfolio_value,
        TradingMode::Real => settings.real_portfolio_value,
    };

    Ok(SafetyCheck::new(
        settings.trading_mode,
        settings.max_drawdown_pct,
        settings.drawdown_pause_pct,
        settings.max_positions,
        settings.max_narrative_pct,
        settings.default_position_pct,
        open_count,
        open_value,
        portfolio_value,
        drawdown,
    ))
}

fn collect(name: &'static str, outcome: SafetyOutcome) -> CheckResult {
    match outcome {
        SafetyOutcome::Allowed => CheckResult {
            name,
            passed: true,
            message: None,
        },
        SafetyOutcome::Blocked(msg) => CheckResult {
            name,
            passed: false,
            message: Some(msg),
        },
    }
}

pub async fn run_general(pool: &SqlitePool) -> anyhow::Result<Vec<CheckResult>> {
    let check = load_check(pool).await?;
    Ok(vec![
        collect("drawdown_pause", check.check_drawdown_pause()),
        collect("drawdown", check.check_drawdown()),
        collect("max_positions", check.check_max_positions()),
    ])
}

pub async fn run_trade_check(
    pool: &SqlitePool,
    trade_type: &str,
    position_size: f64,
) -> anyhow::Result<Vec<CheckResult>> {
    let check = load_check(pool).await?;
    Ok(vec![
        collect("drawdown_pause", check.check_drawdown_pause()),
        collect("drawdown", check.check_drawdown()),
        collect("max_positions", check.check_max_positions()),
        collect("trading_mode", check.check_trading_mode(trade_type)),
        collect("position_size", check.check_position_size(position_size)),
    ])
}
