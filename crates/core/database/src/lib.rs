pub mod models;
pub mod queries;

pub use models::cex_listing::CexListing;
pub use models::cluster_count::ClusterCount;
pub use models::distilled_token::DistilledToken;
pub use models::known_pool::KnownPool;
pub use models::poll_timestamp::PollTimestamp;
pub use models::pump_bonding_curve::PumpBondingCurve;
pub use models::research_entry::ResearchEntry;
pub use models::risk_settings::{RiskSettings, TradingMode};
pub use models::sse_event::SseEvent;
pub use models::telegram::{TelegramChannel, TelegramMessage};
pub use models::token::Token;
pub use models::trade::{Trade, TradeStatus};
pub use queries::risk_settings::{get_risk, upsert_risk};

use sqlx::SqlitePool;
pub use sqlx::migrate::Migrator;
use sqlx::sqlite::SqlitePoolOptions;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn create_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    MIGRATOR.run(&pool).await?;
    Ok(pool)
}
