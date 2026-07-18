use chrono::NaiveDateTime;

#[derive(Default, Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum TradeStatus {
    #[default]
    Watching,
    VirtualBought,
    VirtualSold,
    Bought,
    Sold,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Trade {
    pub id: i64,
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub status: TradeStatus,
    pub trade_type: String,
    pub entry_price: Option<f64>,
    pub entry_date: Option<NaiveDateTime>,
    pub position_size: Option<f64>,
    pub exit_price: Option<f64>,
    pub exit_date: Option<NaiveDateTime>,
    pub notes: String,
    pub stop_loss_pct: Option<f64>,
    pub stop_price: Option<f64>,
    pub trailing_stop: bool,
    pub peak_price: Option<f64>,
    pub stop_loss_enabled: bool,
    pub take_profit_enabled: bool,
    pub take_profit_multiplier: Option<f64>,
    pub peak_decay_enabled: bool,
    pub peak_decay_pct: Option<f64>,
    pub volume_exhaustion_enabled: bool,
    pub volume_exhaustion_pct: Option<f64>,
    pub peak_volume_24h: Option<f64>,
    pub close_reason: Option<String>,
    pub tx_hash: String,
    pub narrative: String,
    pub pump_graduated: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for Trade {
    fn default() -> Self {
        Self {
            id: i64::default(),
            address: String::default(),
            symbol: String::default(),
            name: String::default(),
            status: TradeStatus::default(),
            trade_type: "virtual".into(),
            entry_price: None,
            entry_date: None,
            position_size: None,
            exit_price: None,
            exit_date: None,
            notes: String::default(),
            stop_loss_pct: None,
            stop_price: None,
            trailing_stop: bool::default(),
            peak_price: None,
            stop_loss_enabled: true,
            take_profit_enabled: bool::default(),
            take_profit_multiplier: None,
            peak_decay_enabled: bool::default(),
            peak_decay_pct: None,
            volume_exhaustion_enabled: bool::default(),
            volume_exhaustion_pct: None,
            peak_volume_24h: None,
            close_reason: None,
            tx_hash: String::default(),
            narrative: String::default(),
            pump_graduated: bool::default(),
            created_at: NaiveDateTime::default(),
            updated_at: NaiveDateTime::default(),
        }
    }
}
