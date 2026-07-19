use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router, extract::State};
use serde::Serialize;

use coins_app::state::AppState;

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub database: &'static str,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/healthcheck", get(healthcheck))
}

#[utoipa::path(
    get,
    path = "/healthcheck",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Database is unavailable", body = HealthResponse),
    ),
    tag = "health",
)]
pub async fn healthcheck(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(HealthResponse {
                status: "ok",
                database: "connected",
            }),
        ),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "error",
                database: "disconnected",
            }),
        ),
    }
}
