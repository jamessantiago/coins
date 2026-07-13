/// Trading mode for buy/sell execution and logic
#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum TradingMode {
    Virtual,
    Real
}

/// Settings for managing risk around open trades and current state of the portfolio
#[derive(sqlx::FromRow)]
pub struct RiskSettings {
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
    ///
    pub drawdown_reduce_pct: f64,
    pub drawdown_pause_pct: f64,
}

impl Default for RiskSettings {
    fn default() -> Self {
        Self {
            real_peak_value: 0.0,
            real_portfolio_value: 0.0,
            virtual_portfolio_value: 0.0,
            virtual_peak_value: 1000.0,
            virtual_wallet_balance: 1000.0,
            trading_mode: TradingMode::Virtual,
            max_drawdown_pct: 20.0,
        }
    }
}