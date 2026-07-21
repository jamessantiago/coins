mod client;

use chrono::Utc;
use coins_config::Config;
use coins_database::models::risk_settings::TradingMode;
use coins_database::models::trade::{Trade, TradeStatus};
use coins_database::queries::{poll_timestamp, risk_settings, sse_event, trade};
use coins_trading::virtual_sell;
use sqlx::SqlitePool;

const SERVICE_NAME: &str = "portfolio";

pub async fn run(pool: &SqlitePool, config: &Config) -> anyhow::Result<()> {
    let now = Utc::now().naive_utc();
    tracing::info!("portfolio cycle starting");

    let mut settings = risk_settings::get_risk(pool).await?;
    let open_trades = trade::list_open(pool).await?;

    if open_trades.is_empty() {
        poll_timestamp::upsert(pool, SERVICE_NAME, now, 0).await?;
        tracing::info!("portfolio cycle finished: no open trades");
        return Ok(());
    }

    let search_url = config.dexscreener_search_url();
    let mut processed = 0i32;
    let mut sold_ids: Vec<i64> = Vec::new();

    for t in &open_trades {
        let pairs = client::search_pairs(&search_url, &t.address).await;
        let best = pairs.into_iter().max_by(|a, b| {
            a.liquidity_usd
                .unwrap_or(0.0)
                .partial_cmp(&b.liquidity_usd.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let Some(current_price) = best.as_ref().and_then(|p| p.price_usd) else {
            tracing::warn!("no price data for {} ({})", t.symbol, t.address);
            continue;
        };
        let current_volume = best.as_ref().and_then(|p| p.volume_24h);

        // --- Volume exhaustion (does not depend on peak_price) ---
        if let Some(reason) = check_volume_exhaustion(t, current_volume) {
            execute_sell(pool, t, current_price, reason).await?;
            sold_ids.push(t.id);
            processed += 1;
            continue;
        }

        // --- Update peaks ---
        update_peaks(pool, t, current_price, current_volume).await?;

        // --- Auto-sell conditions ---
        if let Some(reason) = evaluate_auto_sell(t, current_price) {
            execute_sell(pool, t, current_price, reason).await?;
            sold_ids.push(t.id);
            processed += 1;
        }
    }

    // --- Update portfolio values ---
    let remaining = trade::list_open(pool).await?;
    update_portfolio_value(pool, &mut settings, &remaining).await?;

    // --- Global drawdown check ---
    let drawdown = compute_drawdown(&settings);
    if drawdown >= settings.max_drawdown_pct {
        tracing::warn!(
            drawdown = %format_args!("{:.1}%", drawdown),
            "max drawdown triggered — selling all open positions",
        );

        let all_open = if sold_ids.is_empty() {
            remaining
        } else {
            trade::list_open(pool).await?
        };

        for t in &all_open {
            let pairs = crate::client::search_pairs(&search_url, &t.address).await;
            let price = pairs
                .into_iter()
                .max_by(|a, b| {
                    a.liquidity_usd
                        .unwrap_or(0.0)
                        .partial_cmp(&b.liquidity_usd.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .and_then(|p| p.price_usd);
            execute_sell(pool, t, price.unwrap_or(0.0), "max_drawdown").await?;
            processed += 1;
        }
        sold_ids.clear();
    } else if drawdown >= settings.drawdown_reduce_pct && !remaining.is_empty() {
        // reduce — sell the worst performer
        let worst = find_worst_performer(&search_url, &remaining).await;
        if let Some((t, price, _)) = worst {
            execute_sell(pool, &t, price, "drawdown_reduce").await?;
            processed += 1;
        }
    }

    // --- Emit portfolio update ---
    let (portfolio_value, peak_value) = match settings.trading_mode {
        TradingMode::Virtual => (
            settings.virtual_portfolio_value,
            settings.virtual_peak_value,
        ),
        TradingMode::Real => (settings.real_portfolio_value, settings.real_peak_value),
    };
    let current_drawdown = compute_drawdown(&settings);
    let open_count = trade::list_open(pool).await?.len() as i32;

    let event_data = serde_json::json!({
        "portfolio_value": format_args!("{:.4}", portfolio_value).to_string(),
        "peak_value": format_args!("{:.4}", peak_value).to_string(),
        "drawdown_pct": format_args!("{:.2}", current_drawdown).to_string(),
        "open_positions": open_count,
        "sold_this_cycle": sold_ids.len(),
    });
    let _ = sse_event::create(pool, "portfolio_update", &event_data.to_string()).await;

    poll_timestamp::upsert(pool, SERVICE_NAME, now, processed).await?;

    tracing::info!(
        "portfolio cycle finished: {} processed, {} sold, {} open, drawdown {:.1}%",
        processed,
        sold_ids.len(),
        open_count,
        current_drawdown,
    );

    Ok(())
}

fn evaluate_auto_sell(trade: &Trade, current_price: f64) -> Option<&'static str> {
    let entry = match trade.entry_price {
        Some(e) if e > 0.0 => e,
        _ => return None,
    };

    let peak = trade.peak_price.unwrap_or(entry);

    // Hard stop price (absolute floor)
    if let Some(stop) = trade.stop_price
        && current_price < stop
    {
        return Some("stop_price");
    }

    // Stop loss / trailing stop (both use stop_loss_pct)
    if let Some(sl_pct) = trade.stop_loss_pct {
        if sl_pct <= 0.0 {
            // no stop
        } else if trade.trailing_stop {
            let ts_price = peak * (1.0 - sl_pct / 100.0);
            if current_price < ts_price {
                return Some("trailing_stop");
            }
        } else if trade.stop_loss_enabled {
            let sl_price = entry * (1.0 - sl_pct / 100.0);
            if current_price < sl_price {
                return Some("stop_loss");
            }
        }
    }

    // Take profit
    if trade.take_profit_enabled
        && let Some(tp_mult) = trade.take_profit_multiplier
        && tp_mult > 0.0
    {
        let tp_price = entry * tp_mult;
        if current_price > tp_price {
            return Some("take_profit");
        }
    }

    // Peak decay
    if trade.peak_decay_enabled
        && let Some(decay_pct) = trade.peak_decay_pct
        && decay_pct > 0.0
    {
        let decay_price = peak * (1.0 - decay_pct / 100.0);
        if current_price < decay_price {
            return Some("peak_decay");
        }
    }

    None
}

fn check_volume_exhaustion(trade: &Trade, current_volume: Option<f64>) -> Option<&'static str> {
    if !trade.volume_exhaustion_enabled {
        return None;
    }
    let exhaustion_pct = trade.volume_exhaustion_pct?;
    let peak_volume = trade.peak_volume_24h?;
    let current_vol = current_volume?;

    if peak_volume <= 0.0 {
        return None;
    }

    let threshold = peak_volume * (1.0 - exhaustion_pct / 100.0);
    if current_vol < threshold {
        Some("volume_exhaustion")
    } else {
        None
    }
}

async fn update_peaks(
    pool: &SqlitePool,
    trade: &Trade,
    current_price: f64,
    current_volume: Option<f64>,
) -> anyhow::Result<()> {
    // Update peak_price
    let current_peak = trade.peak_price.unwrap_or(0.0);
    if current_price > current_peak {
        trade::update_peak_price(pool, trade.id, current_price).await?;
        tracing::debug!(
            trade_id = trade.id,
            symbol = %trade.symbol,
            new_peak = current_price,
            "peak_price updated",
        );
    }

    // Update peak_volume_24h
    if let Some(vol) = current_volume {
        let current_peak_vol = trade.peak_volume_24h.unwrap_or(0.0);
        if vol > current_peak_vol {
            trade::update_peak_volume(pool, trade.id, vol).await?;
        }
    }

    Ok(())
}

async fn execute_sell(
    pool: &SqlitePool,
    trade: &Trade,
    exit_price: f64,
    close_reason: &'static str,
) -> anyhow::Result<()> {
    let status = &trade.status;

    if *status == TradeStatus::VirtualBought {
        tracing::info!(
            trade_id = trade.id,
            symbol = %trade.symbol,
            exit_price,
            reason = close_reason,
            "auto-selling virtual trade",
        );
        virtual_sell(
            pool,
            coins_trading::VirtualSellRequest {
                trade_id: trade.id,
                exit_price,
                close_reason: Some(close_reason.into()),
            },
        )
        .await?;

        let event_data = serde_json::json!({
            "trade_id": trade.id,
            "symbol": &trade.symbol,
            "exit_price": exit_price,
            "reason": close_reason,
        });
        let _ = sse_event::create(pool, "auto_sell", &event_data.to_string()).await;
    }

    Ok(())
}

async fn update_portfolio_value(
    pool: &SqlitePool,
    settings: &mut coins_database::models::risk_settings::RiskSettings,
    open_trades: &[Trade],
) -> anyhow::Result<()> {
    let now = Utc::now().naive_utc();
    let trading_mode = settings.trading_mode.clone();

    match trading_mode {
        TradingMode::Virtual => {
            let mut total_market_value = 0.0;
            for t in open_trades {
                if t.trade_type != "virtual" {
                    continue;
                }
                if let Some(amt) = current_market_value(t) {
                    total_market_value += amt;
                }
            }

            let total_portfolio = settings.virtual_wallet_balance + total_market_value;
            settings.virtual_portfolio_value = total_market_value;
            if total_portfolio > settings.virtual_peak_value {
                settings.virtual_peak_value = total_portfolio;
            }
            settings.updated_at = now;
            risk_settings::upsert_risk(pool, settings).await?;
        }
        TradingMode::Real => {
            // Real trades without entry_price can't compute market value accurately,
            // but we still update peaks on individual trades.
            let total_value: f64 = open_trades
                .iter()
                .filter(|t| t.trade_type == "real")
                .filter_map(current_market_value)
                .sum();

            settings.real_portfolio_value = settings.real_portfolio_value.max(total_value);
            if settings.real_portfolio_value > settings.real_peak_value {
                settings.real_peak_value = settings.real_portfolio_value;
            }
            settings.updated_at = now;
            risk_settings::upsert_risk(pool, settings).await?;
        }
    }

    Ok(())
}

fn current_market_value(trade: &Trade) -> Option<f64> {
    let entry_price = trade.entry_price?;
    let position_size = trade.position_size?;
    let peak = trade.peak_price?;
    if entry_price <= 0.0 {
        return Some(position_size);
    }
    Some((position_size / entry_price) * peak)
}

fn compute_drawdown(settings: &coins_database::models::risk_settings::RiskSettings) -> f64 {
    let (peak, current) = match settings.trading_mode {
        TradingMode::Virtual => (
            settings.virtual_peak_value,
            settings.virtual_portfolio_value,
        ),
        TradingMode::Real => (settings.real_peak_value, settings.real_portfolio_value),
    };
    if peak <= 0.0 {
        return 0.0;
    }
    ((peak - current) / peak * 100.0).max(0.0)
}

async fn find_worst_performer(
    search_url: &str,
    open_trades: &[Trade],
) -> Option<(Trade, f64, f64)> {
    let mut worst: Option<(Trade, f64, f64)> = None;

    for t in open_trades {
        let pairs = crate::client::search_pairs(search_url, &t.address).await;
        let best = pairs.into_iter().max_by(|a, b| {
            a.liquidity_usd
                .unwrap_or(0.0)
                .partial_cmp(&b.liquidity_usd.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let current_price = match best.as_ref().and_then(|p| p.price_usd) {
            Some(p) => p,
            None => continue,
        };

        let entry = match t.entry_price {
            Some(e) if e > 0.0 => e,
            _ => continue,
        };

        let pnl_pct = ((current_price - entry) / entry) * 100.0;

        match &worst {
            Some((_, _, worst_pnl)) if pnl_pct >= *worst_pnl => {}
            _ => worst = Some((t.clone(), current_price, pnl_pct)),
        }
    }

    worst
}
