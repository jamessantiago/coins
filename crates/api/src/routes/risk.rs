use axum::{
    Json, Router,
    extract::State,
    routing::{get, post, put},
};
use coins_app::dto::settings::{AddFundsRequest, RiskSettingsResponse, UpdateRiskSettingsRequest};
use coins_app::risk;
use coins_app::state::AppState;

use crate::error::AppError;
use crate::extractor::ValidatedJson;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/risk", get(get_risk))
        .route("/risk", put(update_risk))
        .route("/risk/reset-drawdown", post(reset_drawdown))
        .route("/risk/add-funds", post(add_funds))
}

#[utoipa::path(
    get,
    path = "/risk",
    responses(
        (status = 200, description = "Current risk settings", body = RiskSettingsResponse),
    ),
    tag = "risk",
)]
pub async fn get_risk(
    State(state): State<AppState>,
) -> Result<Json<RiskSettingsResponse>, AppError> {
    let settings = risk::get_current(&state.pool)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(RiskSettingsResponse::from(settings)))
}

#[utoipa::path(
    put,
    path = "/risk",
    request_body = UpdateRiskSettingsRequest,
    responses(
        (status = 200, description = "Updated risk settings", body = RiskSettingsResponse),
    ),
    tag = "risk",
)]
pub async fn update_risk(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<UpdateRiskSettingsRequest>,
) -> Result<Json<RiskSettingsResponse>, AppError> {
    let settings = body.into_model();
    risk::upsert(&state.pool, &settings)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(RiskSettingsResponse::from(settings)))
}

#[utoipa::path(
    post,
    path = "/risk/reset-drawdown",
    responses(
        (status = 200, description = "Drawdown reset", body = RiskSettingsResponse),
    ),
    tag = "risk",
)]
pub async fn reset_drawdown(
    State(state): State<AppState>,
) -> Result<Json<RiskSettingsResponse>, AppError> {
    let settings = risk::reset_drawdown(&state.pool)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(RiskSettingsResponse::from(settings)))
}

#[utoipa::path(
    post,
    path = "/risk/add-funds",
    request_body = AddFundsRequest,
    responses(
        (status = 200, description = "Funds added", body = RiskSettingsResponse),
    ),
    tag = "risk",
)]
pub async fn add_funds(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<AddFundsRequest>,
) -> Result<Json<RiskSettingsResponse>, AppError> {
    let settings = risk::add_funds(&state.pool, body.amount)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(RiskSettingsResponse::from(settings)))
}
