use coins_database::RiskSettings;
use coins_database::models::risk_settings::TradingMode;
use coins_trading::{SafetyCheck, SafetyOutcome};

fn settings() -> RiskSettings {
    RiskSettings::default()
}

// ---------------------------------------------------------------------------
// SafetyOutcome
// ---------------------------------------------------------------------------

#[test]
fn is_allowed_returns_true() {
    assert!(SafetyOutcome::Allowed.is_allowed());
    assert!(!SafetyOutcome::Blocked("x".into()).is_allowed());
}

#[test]
fn is_blocked_returns_true() {
    assert!(SafetyOutcome::Blocked("x".into()).is_blocked());
    assert!(!SafetyOutcome::Allowed.is_blocked());
}

#[test]
fn reason_returns_message() {
    assert_eq!(SafetyOutcome::Blocked("test".into()).reason(), Some("test"));
    assert_eq!(SafetyOutcome::Allowed.reason(), None);
}

#[test]
fn and_then_chains_on_allowed() {
    let result = SafetyOutcome::Allowed.and_then(|| SafetyOutcome::Blocked("second".into()));
    assert_eq!(result.reason(), Some("second"));
}

#[test]
fn and_then_short_circuits_blocked() {
    let result = SafetyOutcome::Blocked("first".into()).and_then(|| SafetyOutcome::Allowed);
    assert_eq!(result.reason(), Some("first"));
}

// ---------------------------------------------------------------------------
// run_all / check_*
// ---------------------------------------------------------------------------

#[test]
fn pass_when_all_clear() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 0.0);
    assert_eq!(c.run_all(), SafetyOutcome::Allowed);
}

#[test]
fn block_on_max_positions() {
    let c = SafetyCheck::new(settings(), 8, 0.0, 1000.0, 0.0);
    assert!(c.run_all().is_blocked());
}

#[test]
fn block_on_drawdown() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 25.0);
    assert!(c.run_all().is_blocked());
}

#[test]
fn block_on_drawdown_pause() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 12.0);
    assert!(c.run_all().is_blocked());
}

#[test]
fn pass_drawdown_under_pause() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 8.0);
    assert_eq!(c.run_all(), SafetyOutcome::Allowed);
}

#[test]
fn check_drawdown_boundary() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 20.0);
    assert!(c.check_drawdown().is_blocked());

    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 19.999);
    assert!(c.check_drawdown().is_allowed());
}

#[test]
fn check_drawdown_pause_threshold() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 10.0);
    assert!(c.check_drawdown_pause().is_blocked());
}

#[test]
fn check_max_positions_boundary() {
    let c = SafetyCheck::new(settings(), 8, 0.0, 1000.0, 0.0);
    assert!(c.check_max_positions().is_blocked());

    let c = SafetyCheck::new(settings(), 7, 0.0, 1000.0, 0.0);
    assert!(c.check_max_positions().is_allowed());
}

#[test]
fn short_circuits_on_first_block() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 15.0);
    let result = c.run_all();
    assert!(result.is_blocked());
    assert!(result.reason().unwrap().contains("pause"));
}

// ---------------------------------------------------------------------------
// check_trading_mode
// ---------------------------------------------------------------------------

#[test]
fn block_real_trade_in_virtual_mode() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 0.0);
    assert!(c.check_trading_mode("real").is_blocked());
    assert!(c.check_trading_mode("virtual").is_allowed());
}

#[test]
fn allow_real_trade_in_real_mode() {
    let c = SafetyCheck::new(
        RiskSettings {
            trading_mode: TradingMode::Real,
            ..Default::default()
        },
        0,
        0.0,
        1000.0,
        0.0,
    );
    assert!(c.check_trading_mode("real").is_allowed());
}

// ---------------------------------------------------------------------------
// check_position_size
// ---------------------------------------------------------------------------

#[test]
fn block_position_size_exceeds_default() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 0.0);
    let max_pos = 1000.0 * 2.0 / 100.0;
    assert!(c.check_position_size(max_pos + 1.0).is_blocked());
    assert!(c.check_position_size(max_pos).is_allowed());
}

#[test]
fn check_position_size_boundary() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 0.0);
    let max_pos = 1000.0 * 2.0 / 100.0;
    assert!(c.check_position_size(max_pos).is_allowed());
    assert!(c.check_position_size(max_pos + 0.001).is_blocked());
}

#[test]
fn block_narrative_allocation() {
    let c = SafetyCheck::new(settings(), 0, 290.0, 1000.0, 0.0);
    assert!(c.check_position_size(15.0).is_blocked());
}

#[test]
fn pass_narrative_allocation() {
    let c = SafetyCheck::new(settings(), 0, 280.0, 1000.0, 0.0);
    assert!(c.check_position_size(15.0).is_allowed());
}

// ---------------------------------------------------------------------------
// run_all_for_trade
// ---------------------------------------------------------------------------

#[test]
fn run_all_for_trade_full_flow() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 0.0);
    assert_eq!(
        c.run_all_for_trade("virtual", 10.0),
        SafetyOutcome::Allowed
    );
}

#[test]
fn run_all_for_trade_blocked_by_mode() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 0.0);
    assert!(c.run_all_for_trade("real", 10.0).is_blocked());
}

#[test]
fn run_all_for_trade_blocked_by_drawdown() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 25.0);
    assert!(c.run_all_for_trade("virtual", 10.0).is_blocked());
}

#[test]
fn run_all_for_trade_blocked_by_position_size() {
    let c = SafetyCheck::new(settings(), 0, 0.0, 1000.0, 0.0);
    assert!(c.run_all_for_trade("virtual", 100.0).is_blocked());
}
