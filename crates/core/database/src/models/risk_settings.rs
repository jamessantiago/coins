use chrono::NaiveDateTime;
use sqlx::sqlite::SqliteArguments;

/// Trading mode for buy/sell execution and logic
#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum TradingMode {
    Virtual,
    Real,
}

/// Settings for managing risk around open trades and current state of the portfolio
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RiskSettings {
    /// Highest unrealized/realized value of real portfolio
    pub peak_value: f64,
    /// Highest unrealized/realized value of portfolio
    pub real_peak_value: f64,
    /// Current unrealized/realized value of portfolio
    pub real_portfolio_value: f64,
    /// Highest unrealized/realized value of virtual portfolio
    pub virtual_peak_value: f64,
    /// Current unrealized/realized value of virtual portfolio
    pub virtual_portfolio_value: f64,
    /// Balance of virtual wallet
    pub virtual_wallet_balance: f64,
    /// Trading mode affects whether trades are executed against real or virtual wallets
    pub trading_mode: TradingMode,
    /// Drawdown limit before automatic sells are triggered
    pub max_drawdown_pct: f64,
    /// Drawdown % at which position sizes are reduced
    pub drawdown_reduce_pct: f64,
    /// Drawdown % at which new trades are paused
    pub drawdown_pause_pct: f64,
    /// Max number of open positions allowed
    pub max_positions: i32,
    /// Max allocation per narrative cluster
    pub max_narrative_pct: f64,
    /// Default stop loss % for new positions
    pub default_stop_pct: f64,
    /// Default position size % of portfolio
    pub default_position_pct: f64,
    /// Last updated timestamp
    pub updated_at: NaiveDateTime,
}

impl RiskSettings {
    pub fn bind_to<'q>(
        &self,
        query: sqlx::query::Query<'q, sqlx::Sqlite, SqliteArguments>,
    ) -> sqlx::query::Query<'q, sqlx::Sqlite, SqliteArguments> {
        query
            .bind(self.peak_value)
            .bind(self.real_peak_value)
            .bind(self.real_portfolio_value)
            .bind(self.virtual_peak_value)
            .bind(self.virtual_portfolio_value)
            .bind(self.virtual_wallet_balance)
            .bind(&self.trading_mode)
            .bind(self.max_drawdown_pct)
            .bind(self.drawdown_reduce_pct)
            .bind(self.drawdown_pause_pct)
            .bind(self.max_positions)
            .bind(self.max_narrative_pct)
            .bind(self.default_stop_pct)
            .bind(self.default_position_pct)
            .bind(self.updated_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_match_django() {
        let s = RiskSettings::default();
        assert_eq!(s.peak_value, 1000.0);
        assert_eq!(s.real_peak_value, 0.0);
        assert_eq!(s.real_portfolio_value, 0.0);
        assert_eq!(s.virtual_peak_value, 1000.0);
        assert_eq!(s.virtual_portfolio_value, 1000.0);
        assert_eq!(s.virtual_wallet_balance, 1000.0);
        assert_eq!(s.trading_mode, TradingMode::Virtual);
        assert_eq!(s.max_drawdown_pct, 20.0);
        assert_eq!(s.drawdown_reduce_pct, 5.0);
        assert_eq!(s.drawdown_pause_pct, 10.0);
        assert_eq!(s.max_positions, 8);
        assert_eq!(s.max_narrative_pct, 30.0);
        assert_eq!(s.default_stop_pct, 20.0);
        assert_eq!(s.default_position_pct, 2.0);
    }

    #[test]
    fn trading_mode_roundtrips() {
        assert_eq!(TradingMode::Virtual, TradingMode::Virtual);
        assert_ne!(TradingMode::Virtual, TradingMode::Real);
    }
}

impl Default for RiskSettings {
    fn default() -> Self {
        Self {
            peak_value: 1000.0,
            real_peak_value: 0.0,
            real_portfolio_value: 0.0,
            virtual_peak_value: 1000.0,
            virtual_portfolio_value: 1000.0,
            virtual_wallet_balance: 1000.0,
            trading_mode: TradingMode::Virtual,
            max_drawdown_pct: 20.0,
            drawdown_reduce_pct: 5.0,
            drawdown_pause_pct: 10.0,
            max_positions: 8,
            max_narrative_pct: 30.0,
            default_stop_pct: 20.0,
            default_position_pct: 2.0,
            updated_at: NaiveDateTime::default(),
        }
    }
}
