use utoipa::OpenApi;

use crate::routes::health;
use crate::routes::risk;
use crate::routes::safety;

use coins_app::dto::safety::{SafetyCheckResponse, SafetyCheckTradeRequest};
use coins_app::dto::settings::{AddFundsRequest, RiskSettingsResponse, UpdateRiskSettingsRequest};
use coins_app::safety::CheckResult;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::healthcheck,
        risk::get_risk,
        risk::update_risk,
        risk::reset_drawdown,
        risk::add_funds,
        safety::check_general,
        safety::check_trade,
    ),
    components(
        schemas(
            health::HealthResponse,
            RiskSettingsResponse,
            UpdateRiskSettingsRequest,
            AddFundsRequest,
            SafetyCheckResponse,
            SafetyCheckTradeRequest,
            CheckResult,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "risk", description = "Risk settings management"),
        (name = "safety", description = "Safety check endpoints"),
    ),
)]
pub struct ApiDoc;
