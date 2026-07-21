use coins_database::RiskSettings;
use coins_database::models::risk_settings::TradingMode;

#[derive(Debug, Clone, PartialEq)]
pub enum SafetyOutcome {
    Allowed,
    Blocked(String),
}

impl SafetyOutcome {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Blocked(_))
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Blocked(r) => Some(r),
            Self::Allowed => None,
        }
    }

    pub fn and_then(self, next: impl FnOnce() -> Self) -> Self {
        match self {
            Self::Allowed => next(),
            blocked => blocked,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SafetyCheck {
    risk_settings: RiskSettings,
    open_count: i32,
    open_value: f64,
    portfolio_value: f64,
    current_drawdown_pct: f64,
}

impl SafetyCheck {
    pub fn new(
        risk_settings: RiskSettings,
        open_count: i32,
        open_value: f64,
        portfolio_value: f64,
        current_drawdown_pct: f64,
    ) -> Self {
        Self {
            risk_settings,
            open_count,
            open_value,
            portfolio_value,
            current_drawdown_pct,
        }
    }

    pub fn check_max_positions(&self) -> SafetyOutcome {
        if self.open_count >= self.risk_settings.max_positions {
            SafetyOutcome::Blocked(format!(
                "open positions ({}) >= max positions ({})",
                self.open_count, self.risk_settings.max_positions,
            ))
        } else {
            SafetyOutcome::Allowed
        }
    }

    pub fn check_drawdown(&self) -> SafetyOutcome {
        if self.current_drawdown_pct >= self.risk_settings.max_drawdown_pct {
            SafetyOutcome::Blocked(format!(
                "drawdown {:.1}% >= max drawdown {:.1}%",
                self.current_drawdown_pct, self.risk_settings.max_drawdown_pct,
            ))
        } else {
            SafetyOutcome::Allowed
        }
    }

    pub fn check_drawdown_pause(&self) -> SafetyOutcome {
        if self.current_drawdown_pct >= self.risk_settings.drawdown_pause_pct {
            SafetyOutcome::Blocked(format!(
                "drawdown {:.1}% >= pause threshold {:.1}% — new trades paused",
                self.current_drawdown_pct, self.risk_settings.drawdown_pause_pct,
            ))
        } else {
            SafetyOutcome::Allowed
        }
    }

    pub fn check_trading_mode(&self, proposed_trade_type: &str) -> SafetyOutcome {
        match self.risk_settings.trading_mode {
            TradingMode::Virtual if proposed_trade_type == "real" => {
                SafetyOutcome::Blocked("trading mode is Virtual — real trades not allowed".into())
            }
            _ => SafetyOutcome::Allowed,
        }
    }

    pub fn check_position_size(&self, position_size: f64) -> SafetyOutcome {
        let max_position = self.portfolio_value * self.risk_settings.default_position_pct / 100.0;
        let max_narrative = self.portfolio_value * self.risk_settings.max_narrative_pct / 100.0;
        let total_after = self.open_value + position_size;

        if position_size > max_position {
            SafetyOutcome::Blocked(format!(
                "position size {:.4} exceeds max per-position limit {:.4} ({:.1}% of portfolio)",
                position_size, max_position, self.risk_settings.default_position_pct,
            ))
        } else if total_after > max_narrative {
            SafetyOutcome::Blocked(format!(
                "total position value after trade ({:.4}) would exceed {:.1}% narrative \
                 allocation limit ({:.4})",
                total_after, self.risk_settings.max_narrative_pct, max_narrative,
            ))
        } else {
            SafetyOutcome::Allowed
        }
    }

    pub fn run_all(&self) -> SafetyOutcome {
        self.check_drawdown_pause()
            .and_then(|| self.check_drawdown())
            .and_then(|| self.check_max_positions())
    }

    pub fn run_all_for_trade(&self, trade_type: &str, position_size: f64) -> SafetyOutcome {
        self.check_drawdown_pause()
            .and_then(|| self.check_drawdown())
            .and_then(|| self.check_max_positions())
            .and_then(|| self.check_trading_mode(trade_type))
            .and_then(|| self.check_position_size(position_size))
    }
}
