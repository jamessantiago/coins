use anyhow::Result;
use sqlx::SqlitePool;

use crate::models::pump_bonding_curve::PumpBondingCurve;

pub async fn bulk_create(pool: &SqlitePool, curves: &[PumpBondingCurve]) -> Result<()> {
    let mut tx = pool.begin().await?;
    for c in curves {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO pump_bonding_curves (bonding_curve, mint, first_seen)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(&c.bonding_curve)
        .bind(&c.mint)
        .bind(c.first_seen)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn list_bonding_curves(pool: &SqlitePool) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>("SELECT bonding_curve FROM pump_bonding_curves")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}
