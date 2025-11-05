use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

use crate::database::{check_database_health, models::HealthCheck, DbPool};

/// 健康检查接口
pub async fn health_check(
    State(pool): State<DbPool>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 检查数据库连接
    match check_database_health(&pool).await {
        Ok(_) => {
            let health_status = HealthCheck {
                status: "healthy".to_string(),
                database: "connected".to_string(),
                timestamp: chrono::Utc::now(),
            };

            Ok(Json(json!({
                "status": "success",
                "data": health_status,
                "message": "服务端正常运行 + MySQL 连接成功 ✅"
            })))
        }
        Err(e) => {
            tracing::error!("数据库连接失败: {}", e);
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "error",
                    "message": format!("数据库连接失败: {}", e)
                })),
            ))
        }
    }
}

/// 根路径欢迎接口
pub async fn root() -> Json<Value> {
    Json(json!({
        "message": "欢迎使用 Axum + SQLx MySQL 服务",
        "version": "1.0.0"
    }))
}