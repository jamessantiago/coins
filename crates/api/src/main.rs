mod routes;
mod state;

use axum::routing::get;
use axum::Router;
use coins_config::Config;
use coins_database::create_pool;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::routes::health::healthcheck;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let config = Config::from_env()?;
    let pool = create_pool(&config.core_database).await?;

    let host = config.host.clone().unwrap_or("0.0.0.0".into());
    let port = config.port.unwrap_or(3000);

    let state = AppState { pool, config };

    let app = Router::new()
        .route("/healthcheck", get(healthcheck))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("{}:{}", host, port);

    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
