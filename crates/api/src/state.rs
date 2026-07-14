use coins_config::Config;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    #[allow(dead_code)]
    pub config: Config,
}
