use anyhow::Result;
use sqlx::SqlitePool;

use crate::models::risk_settings::RiskSettings;

/// Retrieve the singleton RiskSettings row (pk=1), creating it with defaults if it
/// doesn't exist yet.
pub async fn get_risk(pool: &SqlitePool) -> Result<RiskSettings> {
    let result = sqlx::query_as::<_, RiskSettings>(
        r#"
        SELECT
            peak_value,
            real_peak_value,
            real_portfolio_value,
            virtual_peak_value,
            virtual_portfolio_value,
            virtual_wallet_balance,
            trading_mode,
            max_drawdown_pct,
            drawdown_reduce_pct,
            drawdown_pause_pct,
            max_positions,
            max_narrative_pct,
            default_stop_pct,
            default_position_pct,
            updated_at
        FROM risk_settings
        WHERE id = 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    match result {
        Some(settings) => Ok(settings),
        None => {
            let defaults = RiskSettings::default();
            upsert_risk(pool, &defaults).await?;
            Ok(defaults)
        }
    }
}

/// Upsert the singleton RiskSettings row (pk=1).
pub async fn upsert_risk(pool: &SqlitePool, settings: &RiskSettings) -> Result<()> {
    let query = sqlx::query(
        r#"
        INSERT INTO risk_settings (
            id, peak_value, real_peak_value, real_portfolio_value,
            virtual_peak_value, virtual_portfolio_value, virtual_wallet_balance,
            trading_mode,
            max_drawdown_pct, drawdown_reduce_pct, drawdown_pause_pct,
            max_positions, max_narrative_pct,
            default_stop_pct, default_position_pct,
            updated_at
        ) VALUES (
            1,
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
        )
        ON CONFLICT(id) DO UPDATE SET
            peak_value             = excluded.peak_value,
            real_peak_value        = excluded.real_peak_value,
            real_portfolio_value   = excluded.real_portfolio_value,
            virtual_peak_value     = excluded.virtual_peak_value,
            virtual_portfolio_value = excluded.virtual_portfolio_value,
            virtual_wallet_balance = excluded.virtual_wallet_balance,
            trading_mode           = excluded.trading_mode,
            max_drawdown_pct       = excluded.max_drawdown_pct,
            drawdown_reduce_pct    = excluded.drawdown_reduce_pct,
            drawdown_pause_pct     = excluded.drawdown_pause_pct,
            max_positions          = excluded.max_positions,
            max_narrative_pct      = excluded.max_narrative_pct,
            default_stop_pct       = excluded.default_stop_pct,
            default_position_pct   = excluded.default_position_pct,
            updated_at             = excluded.updated_at
        "#,
    );

    settings.bind_to(query).execute(pool).await?;
    Ok(())
}
