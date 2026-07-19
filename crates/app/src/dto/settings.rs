use serde::{Deserialize, Serialize};
use validator::Validate;

use coins_database::models::risk_settings::{RiskSettings, TradingMode};

use crate::dto::common::fmt_ts;

fn trading_mode_str(m: &TradingMode) -> String {
    match m {
        TradingMode::Virtual => "virtual",
        TradingMode::Real => "real",
    }
    .to_string()
}

fn validate_trading_mode(v: &str) -> Result<(), validator::ValidationError> {
    if matches!(v, "virtual" | "real") {
        Ok(())
    } else {
        Err(validator::ValidationError::new("trading_mode"))
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RiskSettingsResponse {
    pub peak_value: f64,
    pub real_peak_value: f64,
    pub real_portfolio_value: f64,
    pub virtual_peak_value: f64,
    pub virtual_portfolio_value: f64,
    pub virtual_wallet_balance: f64,
    pub trading_mode: String,
    pub max_drawdown_pct: f64,
    pub drawdown_reduce_pct: f64,
    pub drawdown_pause_pct: f64,
    pub max_positions: i32,
    pub max_narrative_pct: f64,
    pub default_stop_pct: f64,
    pub default_position_pct: f64,
    pub updated_at: String,
}

impl From<RiskSettings> for RiskSettingsResponse {
    fn from(s: RiskSettings) -> Self {
        Self {
            peak_value: s.peak_value,
            real_peak_value: s.real_peak_value,
            real_portfolio_value: s.real_portfolio_value,
            virtual_peak_value: s.virtual_peak_value,
            virtual_portfolio_value: s.virtual_portfolio_value,
            virtual_wallet_balance: s.virtual_wallet_balance,
            trading_mode: trading_mode_str(&s.trading_mode),
            max_drawdown_pct: s.max_drawdown_pct,
            drawdown_reduce_pct: s.drawdown_reduce_pct,
            drawdown_pause_pct: s.drawdown_pause_pct,
            max_positions: s.max_positions,
            max_narrative_pct: s.max_narrative_pct,
            default_stop_pct: s.default_stop_pct,
            default_position_pct: s.default_position_pct,
            updated_at: fmt_ts(s.updated_at),
        }
    }
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateRiskSettingsRequest {
    #[validate(range(min = 0.0))]
    pub peak_value: f64,

    #[validate(range(min = 0.0))]
    pub real_peak_value: f64,

    #[validate(range(min = 0.0))]
    pub real_portfolio_value: f64,

    #[validate(range(min = 0.0))]
    pub virtual_peak_value: f64,

    #[validate(range(min = 0.0))]
    pub virtual_portfolio_value: f64,

    #[validate(range(min = 0.0))]
    pub virtual_wallet_balance: f64,

    #[validate(custom(function = "validate_trading_mode"))]
    pub trading_mode: String,

    #[validate(range(min = 0.0, max = 100.0))]
    pub max_drawdown_pct: f64,

    #[validate(range(min = 0.0, max = 100.0))]
    pub drawdown_reduce_pct: f64,

    #[validate(range(min = 0.0, max = 100.0))]
    pub drawdown_pause_pct: f64,

    #[validate(range(min = 1))]
    pub max_positions: i32,

    #[validate(range(min = 0.0, max = 100.0))]
    pub max_narrative_pct: f64,

    #[validate(range(min = 0.0, max = 100.0))]
    pub default_stop_pct: f64,

    #[validate(range(min = 0.0, max = 100.0))]
    pub default_position_pct: f64,
}

impl UpdateRiskSettingsRequest {
    pub fn into_model(self) -> RiskSettings {
        RiskSettings {
            peak_value: self.peak_value,
            real_peak_value: self.real_peak_value,
            real_portfolio_value: self.real_portfolio_value,
            virtual_peak_value: self.virtual_peak_value,
            virtual_portfolio_value: self.virtual_portfolio_value,
            virtual_wallet_balance: self.virtual_wallet_balance,
            trading_mode: match self.trading_mode.as_str() {
                "virtual" => TradingMode::Virtual,
                _ => TradingMode::Real,
            },
            max_drawdown_pct: self.max_drawdown_pct,
            drawdown_reduce_pct: self.drawdown_reduce_pct,
            drawdown_pause_pct: self.drawdown_pause_pct,
            max_positions: self.max_positions,
            max_narrative_pct: self.max_narrative_pct,
            default_stop_pct: self.default_stop_pct,
            default_position_pct: self.default_position_pct,
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct AddFundsRequest {
    #[validate(range(min = 0.0))]
    pub amount: f64,
}
