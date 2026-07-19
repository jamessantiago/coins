use chrono::Utc;
use sqlx::SqlitePool;

use coins_database::models::trade::{Trade, TradeStatus};
use coins_database::queries::risk_settings::{get_risk, upsert_risk};
use coins_database::queries::trade::{create, get_by_id, update};

#[derive(Debug, Clone)]
pub struct VirtualBuyRequest {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub position_size: f64,
    pub entry_price: f64,
    pub narrative: String,
}

#[derive(Debug, Clone)]
pub struct VirtualSellRequest {
    pub trade_id: i64,
    pub exit_price: f64,
    pub close_reason: Option<String>,
}

pub async fn virtual_buy(pool: &SqlitePool, req: VirtualBuyRequest) -> anyhow::Result<Trade> {
    let mut settings = get_risk(pool).await?;

    if req.position_size > settings.virtual_wallet_balance {
        anyhow::bail!(
            "insufficient virtual balance: have {:.2}, need {:.2}",
            settings.virtual_wallet_balance,
            req.position_size,
        );
    }

    let now = Utc::now().naive_utc();

    let trade = Trade {
        id: 0,
        address: req.address,
        symbol: req.symbol,
        name: req.name,
        status: TradeStatus::VirtualBought,
        trade_type: "virtual".into(),
        entry_price: Some(req.entry_price),
        entry_date: Some(now),
        position_size: Some(req.position_size),
        exit_price: None,
        exit_date: None,
        notes: String::new(),
        stop_loss_pct: Some(settings.default_stop_pct),
        stop_price: None,
        trailing_stop: false,
        peak_price: Some(req.entry_price),
        stop_loss_enabled: true,
        take_profit_enabled: false,
        take_profit_multiplier: None,
        peak_decay_enabled: false,
        peak_decay_pct: None,
        volume_exhaustion_enabled: false,
        volume_exhaustion_pct: None,
        peak_volume_24h: None,
        close_reason: None,
        tx_hash: String::new(),
        narrative: req.narrative,
        pump_graduated: false,
        created_at: now,
        updated_at: now,
    };

    let trade = create(pool, &trade).await?;

    settings.virtual_wallet_balance -= req.position_size;
    settings.virtual_portfolio_value += req.position_size;
    if settings.virtual_portfolio_value > settings.virtual_peak_value {
        settings.virtual_peak_value = settings.virtual_portfolio_value;
    }
    settings.updated_at = now;
    upsert_risk(pool, &settings).await?;

    Ok(trade)
}

pub async fn virtual_sell(pool: &SqlitePool, req: VirtualSellRequest) -> anyhow::Result<Trade> {
    let mut trade = get_by_id(pool, req.trade_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("trade {} not found", req.trade_id))?;

    anyhow::ensure!(
        trade.status == TradeStatus::VirtualBought,
        "trade {} is not VirtualBought (status: {:?})",
        req.trade_id,
        trade.status,
    );

    let now = Utc::now().naive_utc();
    let entry_price = trade.entry_price.unwrap_or(req.exit_price);
    let position_size = trade.position_size.unwrap_or(0.0);

    let proceeds = if entry_price > 0.0 {
        (position_size / entry_price) * req.exit_price
    } else {
        position_size
    };

    trade.status = TradeStatus::VirtualSold;
    trade.exit_price = Some(req.exit_price);
    trade.exit_date = Some(now);
    trade.close_reason = req.close_reason;
    trade.updated_at = now;

    update(pool, &trade).await?;

    let mut settings = get_risk(pool).await?;
    settings.virtual_wallet_balance += proceeds;
    settings.virtual_portfolio_value -= position_size;
    if settings.virtual_portfolio_value > settings.virtual_peak_value {
        settings.virtual_peak_value = settings.virtual_portfolio_value;
    }
    settings.updated_at = now;
    upsert_risk(pool, &settings).await?;

    Ok(trade)
}
