use coins_database::models::risk_settings::TradingMode;
use coins_trading::{SafetyCheck, SafetyOutcome};

fn check() -> SafetyCheck {
    SafetyCheck::new(TradingMode::Virtual, 20.0, 10.0, 8, 30.0, 2.0, 0, 0.0, 1000.0, 0.0)
}

#[test]
fn pass_when_all_clear() {
    assert_eq!(check().run_all(), SafetyOutcome::Allowed);
}

#[test]
fn block_on_max_positions() {
    let c = SafetyCheck::new(
        TradingMode::Virtual, 20.0, 10.0, 8, 30.0, 2.0, 8, 0.0, 1000.0, 0.0,
    );
    assert!(c.run_all().is_blocked());
}

#[test]
fn block_on_drawdown() {
    let c = SafetyCheck::new(
        TradingMode::Virtual, 20.0, 10.0, 8, 30.0, 2.0, 0, 0.0, 1000.0, 25.0,
    );
    assert!(c.run_all().is_blocked());
}

#[test]
fn block_on_drawdown_pause() {
    let c = SafetyCheck::new(
        TradingMode::Virtual, 20.0, 10.0, 8, 30.0, 2.0, 0, 0.0, 1000.0, 12.0,
    );
    assert!(c.run_all().is_blocked());
}

#[test]
fn pass_drawdown_under_pause() {
    let c = SafetyCheck::new(
        TradingMode::Virtual, 20.0, 10.0, 8, 30.0, 2.0, 0, 0.0, 1000.0, 8.0,
    );
    assert_eq!(c.run_all(), SafetyOutcome::Allowed);
}

#[test]
fn block_real_trade_in_virtual_mode() {
    assert!(check().check_trading_mode("real").is_blocked());
    assert!(check().check_trading_mode("virtual").is_allowed());
}

#[test]
fn allow_real_trade_in_real_mode() {
    let c = SafetyCheck::new(
        TradingMode::Real, 20.0, 10.0, 8, 30.0, 2.0, 0, 0.0, 1000.0, 0.0,
    );
    assert!(c.check_trading_mode("real").is_allowed());
}

#[test]
fn block_position_size_exceeds_default() {
    let c = check();
    let max_pos = 1000.0 * 2.0 / 100.0;
    assert!(c.check_position_size(max_pos + 1.0).is_blocked());
    assert!(c.check_position_size(max_pos).is_allowed());
}

#[test]
fn block_narrative_allocation() {
    let c = SafetyCheck::new(
        TradingMode::Virtual, 20.0, 10.0, 8, 30.0, 2.0, 0, 290.0, 1000.0, 0.0,
    );
    assert!(c.check_position_size(15.0).is_blocked());
}

#[test]
fn pass_narrative_allocation() {
    let c = SafetyCheck::new(
        TradingMode::Virtual, 20.0, 10.0, 8, 30.0, 2.0, 0, 280.0, 1000.0, 0.0,
    );
    assert!(c.check_position_size(15.0).is_allowed());
}

#[test]
fn run_all_for_trade_full_flow() {
    assert_eq!(
        check().run_all_for_trade("virtual", 10.0),
        SafetyOutcome::Allowed
    );
}

#[test]
fn run_all_for_trade_blocked_by_mode() {
    assert!(check().run_all_for_trade("real", 10.0).is_blocked());
}

#[test]
fn reason_returns_message() {
    let outcome = check().check_trading_mode("real");
    assert!(outcome.reason().unwrap().contains("Virtual"));
    assert_eq!(SafetyOutcome::Allowed.reason(), None);
}
