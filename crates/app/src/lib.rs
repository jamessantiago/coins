pub mod dto;
pub mod risk;
pub mod state;

use std::time::Duration;
use coins_config::Config;
use sqlx::SqlitePool;
use state::AppState;

pub struct App {
    pub state: AppState,
}

impl App {
    pub async fn init(config: &Config) -> anyhow::Result<Self> {
        let pool = coins_database::create_pool(&config.core_database).await?;
        spawn_jobs(&pool, config).await;
        Ok(Self {
            state: AppState { pool, config: config.clone() },
        })
    }
}

async fn spawn_jobs(pool: &SqlitePool, config: &Config) {
    let pools = (
        pool.clone(),
        pool.clone(),
        pool.clone(),
        pool.clone(),
    );
    let configs = (
        config.clone(),
        config.clone(),
        config.clone(),
        config.clone(),
    );

    tokio::spawn(async move {
        let pool = pools.0;
        let config = configs.0;
        let mut interval = tokio::time::interval(Duration::from_secs(config.scanner_poll_interval()));
        loop {
            interval.tick().await;
            if let Err(e) = coins_scanner::run(&pool, &config).await {
                tracing::error!(error = %e, "scanner cycle failed");
            }
        }
    });

    tokio::spawn(async move {
        let pool = pools.1;
        let config = configs.1;
        let mut interval = tokio::time::interval(Duration::from_secs(config.cex_poll_interval()));
        loop {
            interval.tick().await;
            if let Err(e) = coins_cex::run(&pool, &config).await {
                tracing::error!(error = %e, "CEX scan cycle failed");
            }
        }
    });

    tokio::spawn(async move {
        let pool = pools.2;
        let config = configs.2;
        let mut interval = tokio::time::interval(Duration::from_secs(config.telegram_poll_interval()));
        loop {
            interval.tick().await;
            if let Err(e) = coins_telegram::run(&pool, &config).await {
                tracing::error!(error = %e, "telegram monitor cycle failed");
            }
        }
    });

    tokio::spawn(async move {
        let pool = pools.3;
        let config = configs.3;
        let mut interval = tokio::time::interval(Duration::from_secs(config.distiller_poll_interval()));
        loop {
            interval.tick().await;
            if let Err(e) = coins_distiller::run(&pool, &config).await {
                tracing::error!(error = %e, "distiller cycle failed");
            }
        }
    });
}
