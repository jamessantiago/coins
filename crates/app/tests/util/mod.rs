use sqlx::SqlitePool;
use coins_database::create_pool;

pub async fn test_pool() -> SqlitePool {
    create_pool("sqlite::memory:").await.unwrap()
}
