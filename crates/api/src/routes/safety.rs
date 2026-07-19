use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};

use coins_app::dto::safety::{SafetyCheckResponse, SafetyCheckTradeRequest};
use coins_app::safety;
use coins_app::state::AppState;

use crate::error::AppError;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/safety/check", get(check_general))
        .route("/safety/check-trade", post(check_trade))
}

#[utoipa::path(
    get,
    path = "/safety/check",
    responses(
        (status = 200, description = "General safety check results", body = SafetyCheckResponse),
    ),
    tag = "safety",
)]
pub async fn check_general(
    State(state): State<AppState>,
) -> Result<Json<SafetyCheckResponse>, AppError> {
    let checks = safety::run_general(&state.pool)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(SafetyCheckResponse::from(checks)))
}

#[utoipa::path(
    post,
    path = "/safety/check-trade",
    request_body = SafetyCheckTradeRequest,
    responses(
        (status = 200, description = "Trade safety check results", body = SafetyCheckResponse),
    ),
    tag = "safety",
)]
pub async fn check_trade(
    State(state): State<AppState>,
    Json(body): Json<SafetyCheckTradeRequest>,
) -> Result<Json<SafetyCheckResponse>, AppError> {
    let checks = safety::run_trade_check(&state.pool, &body.trade_type, body.position_size)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(SafetyCheckResponse::from(checks)))
}
