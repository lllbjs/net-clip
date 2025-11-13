use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// 健康检查模型
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: String,
    pub database: String,
    pub timestamp: DateTime<Utc>,
}

// 用户相关模型
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub salt: String,
    pub status: i8,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub login_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSession {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Clip 相关模型
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClipContent {
    pub id: i64,
    pub user_id: i64,
    pub title: Option<String>,
    pub content: String,
    pub content_type: String,
    pub language: Option<String>,
    pub is_encrypted: i8,
    pub encryption_key: Option<String>,
    pub access_type: String,
    pub view_count: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub short_url: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateClip {
    pub title: Option<String>,
    pub content: String,
    pub content_type: Option<String>,
    pub language: Option<String>,
    pub is_encrypted: Option<bool>,
    pub access_type: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateClip {
    pub title: Option<String>,
    pub content: Option<String>,
    pub content_type: Option<String>,
    pub language: Option<String>,
    pub access_type: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

// JWT Token
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: i64,
    pub exp: usize,
    pub iat: usize,
}

// API 响应模型
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: Option<T>,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: &str) -> Self {
        Self {
            status: "success".to_string(),
            data: Some(data),
            message: message.to_string(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: User,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}