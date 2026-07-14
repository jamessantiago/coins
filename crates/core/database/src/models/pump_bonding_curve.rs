use chrono::NaiveDateTime;

#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct PumpBondingCurve {
    pub bonding_curve: String,
    pub mint: String,
    pub first_seen: NaiveDateTime,
}
