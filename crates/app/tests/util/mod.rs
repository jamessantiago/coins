use coins_database::create_pool;
use sqlx::SqlitePool;

pub async fn test_pool() -> SqlitePool {
    create_pool("sqlite::memory:").await.unwrap()
}
