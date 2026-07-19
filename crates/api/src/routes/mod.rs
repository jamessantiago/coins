use axum::Router;
use coins_app::state::AppState;

pub mod health;
pub mod risk;
pub mod safety;

pub fn router() -> Router<AppState> {
    health::router()
        .merge(risk::router())
        .merge(safety::router())
}
