use coins_app::dto::settings::{
    AddFundsRequest, RiskSettingsResponse, UpdateRiskSettingsRequest,
};
use coins_database::models::risk_settings::{RiskSettings, TradingMode};
use validator::Validate;

fn valid_update_req() -> UpdateRiskSettingsRequest {
    UpdateRiskSettingsRequest {
        peak_value: 1000.0,
        real_peak_value: 0.0,
        real_portfolio_value: 0.0,
        virtual_peak_value: 1000.0,
        virtual_portfolio_value: 1000.0,
        virtual_wallet_balance: 1000.0,
        trading_mode: "virtual".into(),
        max_drawdown_pct: 20.0,
        drawdown_reduce_pct: 5.0,
        drawdown_pause_pct: 10.0,
        max_positions: 8,
        max_narrative_pct: 30.0,
        default_stop_pct: 20.0,
        default_position_pct: 2.0,
    }
}

#[test]
fn response_from_default_model() {
    let model = RiskSettings::default();
    let resp = RiskSettingsResponse::from(model);
    assert_eq!(resp.peak_value, 1000.0);
    assert_eq!(resp.real_peak_value, 0.0);
    assert_eq!(resp.trading_mode, "virtual");
    assert_eq!(resp.max_drawdown_pct, 20.0);
    assert_eq!(resp.max_positions, 8);
    assert_eq!(resp.default_position_pct, 2.0);
}

#[test]
fn response_from_model_with_real_mode() {
    let mut model = RiskSettings::default();
    model.trading_mode = TradingMode::Real;
    model.peak_value = 5000.0;
    let resp = RiskSettingsResponse::from(model);
    assert_eq!(resp.peak_value, 5000.0);
    assert_eq!(resp.trading_mode, "real");
}

#[test]
fn into_model_roundtrips() {
    let req = valid_update_req();
    let model = req.into_model();
    assert_eq!(model.peak_value, 1000.0);
    assert_eq!(model.trading_mode, TradingMode::Virtual);
    assert_eq!(model.max_drawdown_pct, 20.0);
    assert_eq!(model.max_positions, 8);
}

#[test]
fn into_model_parses_real_mode() {
    let mut req = valid_update_req();
    req.trading_mode = "real".into();
    let model = req.into_model();
    assert_eq!(model.trading_mode, TradingMode::Real);
}

#[test]
fn into_model_sets_updated_at() {
    let req = valid_update_req();
    let before = chrono::Utc::now().naive_utc() - chrono::Duration::seconds(5);
    let after = chrono::Utc::now().naive_utc() + chrono::Duration::seconds(5);
    let model = req.into_model();
    assert!(model.updated_at > before);
    assert!(model.updated_at < after);
}

#[test]
fn valid_update_passes_validation() {
    let req = valid_update_req();
    assert!(req.validate().is_ok());
}

#[test]
fn invalid_drawdown_fails_validation() {
    let mut req = valid_update_req();
    req.max_drawdown_pct = 150.0;
    assert!(req.validate().is_err());
}

#[test]
fn negative_peak_fails_validation() {
    let mut req = valid_update_req();
    req.peak_value = -1.0;
    assert!(req.validate().is_err());
}

#[test]
fn invalid_trading_mode_fails_validation() {
    let mut req = valid_update_req();
    req.trading_mode = "invalid".into();
    assert!(req.validate().is_err());
}

#[test]
fn zero_positions_fails_validation() {
    let mut req = valid_update_req();
    req.max_positions = 0;
    assert!(req.validate().is_err());
}

#[test]
fn add_funds_negative_amount_fails() {
    let req = AddFundsRequest { amount: -10.0 };
    assert!(req.validate().is_err());
}

#[test]
fn add_funds_valid_passes() {
    let req = AddFundsRequest { amount: 100.0 };
    assert!(req.validate().is_ok());
}
