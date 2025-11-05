use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HealthCheck {
    pub status: String,
    pub database: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}