use serde::{Deserialize, Serialize};

use crate::safety::CheckResult;

#[derive(Serialize, utoipa::ToSchema)]
pub struct SafetyCheckResponse {
    pub allowed: bool,
    pub checks: Vec<CheckResult>,
}

impl From<Vec<CheckResult>> for SafetyCheckResponse {
    fn from(checks: Vec<CheckResult>) -> Self {
        let allowed = checks.iter().all(|c| c.passed);
        Self { allowed, checks }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SafetyCheckTradeRequest {
    pub trade_type: String,
    pub position_size: f64,
}
