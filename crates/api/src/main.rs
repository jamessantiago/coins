mod routes;
mod state;

use std::net::SocketAddr;
use std::process::exit;
use std::time::Duration;

use axum::Router;
use axum::routing::get;
use axum_server::tls_rustls::RustlsConfig;
use coins_config::Config;
use coins_database::create_pool;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use crate::routes::health::healthcheck;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let config = Config::from_env()?;
    let pool = create_pool(&config.core_database).await?;

    {
        let pool = pool.clone();
        let config = config.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300));
            loop {
                interval.tick().await;
                if let Err(e) = coins_scanner::run(&pool, &config).await {
                    tracing::error!(error = %e, "scanner cycle failed");
                }
            }
        });
    }

    {
        let pool = pool.clone();
        let config = config.clone();
        tokio::spawn(async move {
            let interval_secs = config.cex_poll_interval();
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                if let Err(e) = coins_cex::run(&pool, &config).await {
                    tracing::error!(error = %e, "CEX scan cycle failed");
                }
            }
        });
    }

    let host = config.host.clone().unwrap_or("0.0.0.0".into());
    let port = config.port.unwrap_or(3000);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    let (tls_cert, tls_key) = (config.tls_cert_path.clone(), config.tls_key_path.clone());
    let state = AppState { pool, config };

    let app = Router::new()
        .route("/healthcheck", get(healthcheck))
        .layer(CorsLayer::permissive())
        .with_state(state);

    match (&tls_cert, &tls_key) {
        (Some(cert), Some(key)) => {
            let tls_config = RustlsConfig::from_pem_file(cert, key).await?;
            info!("listening on https://{}", addr);
            axum_server::bind_rustls(addr, tls_config)
                .serve(app.into_make_service())
                .await?;
        }
        _ => {
            error!("Generate a key before continuing, see scripts");
            exit(1);
        }
    }

    Ok(())
}
