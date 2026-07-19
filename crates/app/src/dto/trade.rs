use serde::Serialize;
use coins_database::models::trade::{Trade, TradeStatus};

use crate::dto::common::{fmt_ts, fmt_ts_opt};

#[derive(Serialize)]
pub struct TradeResponse {
    pub id: i64,
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub status: String,
    pub trade_type: String,
    pub entry_price: Option<f64>,
    pub entry_date: Option<String>,
    pub position_size: Option<f64>,
    pub exit_price: Option<f64>,
    pub exit_date: Option<String>,
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
    pub created_at: String,
    pub updated_at: String,
}

fn trade_status_str(s: &TradeStatus) -> String {
    match s {
        TradeStatus::Watching => "watching",
        TradeStatus::VirtualBought => "virtual_bought",
        TradeStatus::VirtualSold => "virtual_sold",
        TradeStatus::Bought => "bought",
        TradeStatus::Sold => "sold",
    }
    .to_string()
}

impl From<Trade> for TradeResponse {
    fn from(t: Trade) -> Self {
        Self {
            id: t.id,
            address: t.address,
            symbol: t.symbol,
            name: t.name,
            status: trade_status_str(&t.status),
            trade_type: t.trade_type,
            entry_price: t.entry_price,
            entry_date: fmt_ts_opt(t.entry_date),
            position_size: t.position_size,
            exit_price: t.exit_price,
            exit_date: fmt_ts_opt(t.exit_date),
            notes: t.notes,
            stop_loss_pct: t.stop_loss_pct,
            stop_price: t.stop_price,
            trailing_stop: t.trailing_stop,
            peak_price: t.peak_price,
            stop_loss_enabled: t.stop_loss_enabled,
            take_profit_enabled: t.take_profit_enabled,
            take_profit_multiplier: t.take_profit_multiplier,
            peak_decay_enabled: t.peak_decay_enabled,
            peak_decay_pct: t.peak_decay_pct,
            volume_exhaustion_enabled: t.volume_exhaustion_enabled,
            volume_exhaustion_pct: t.volume_exhaustion_pct,
            peak_volume_24h: t.peak_volume_24h,
            close_reason: t.close_reason,
            tx_hash: t.tx_hash,
            narrative: t.narrative,
            pump_graduated: t.pump_graduated,
            created_at: fmt_ts(t.created_at),
            updated_at: fmt_ts(t.updated_at),
        }
    }
}
