use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::time::Duration;

use crate::config::Config;

pub mod models;

pub type DbPool = MySqlPool;

/// 初始化 MySQL 连接池
pub async fn init_pool(config: &Config) -> Result<DbPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&config.database_url)
        .await
}

/// 健康检查 - 测试数据库连接
pub async fn check_database_health(pool: &DbPool) -> Result<(), sqlx::Error> {
    let _: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await?;
    Ok(())
}