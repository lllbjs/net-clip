use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::time::Duration;
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::config::Config;

pub mod models;
pub use models::*;

pub type DbPool = MySqlPool;

/// 初始化 MySQL 连接池
pub async fn init_pool(config: &Config) -> Result<DbPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&config.database_url.as_str())
        .await
}

/// 健康检查 - 测试数据库连接
pub async fn check_database_health(pool: &DbPool) -> Result<(), sqlx::Error> {
    let _: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await?;
    Ok(())
}

/// 用户相关操作
pub struct UserRepository;

impl UserRepository {
    /// 创建用户
    pub async fn create_user(pool: &DbPool, user_data: &CreateUser) -> Result<User, sqlx::Error> {
        let salt = Uuid::new_v4().to_string();
        let password_hash = hash(&format!("{}{}", user_data.password, salt), DEFAULT_COST)
            .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))?;

        let result = sqlx::query!(
            r#"
            INSERT INTO clip_users (username, email, password_hash, salt)
            VALUES (?, ?, ?, ?)
            "#,
            user_data.username,
            user_data.email,
            password_hash,
            salt
        )
            .execute(pool)
            .await?;

        Self::find_by_id(pool, result.last_insert_id() as i64).await
    }

    /// 根据ID查找用户
    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<User, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, username, email, password_hash, salt, status,
                   last_login_at, last_login_ip, login_count, created_at, updated_at
            FROM clip_users
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// 根据用户名查找用户
    pub async fn find_by_username(pool: &DbPool, username: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, username, email, password_hash, salt, status,
                   last_login_at, last_login_ip, login_count, created_at, updated_at
            FROM clip_users
            WHERE username = ? AND deleted_at IS NULL
            "#,
        )
            .bind(username)
            .fetch_one(pool)
            .await
    }

    /// 验证用户密码
    pub async fn verify_password(pool: &DbPool, username: &str, password: &str) -> Result<User, sqlx::Error> {
        let user = Self::find_by_username(pool, username).await?;

        let is_valid = verify(&format!("{}{}", password, user.salt), &user.password_hash)
            .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))?;

        if is_valid {
            Ok(user)
        } else {
            Err(sqlx::Error::RowNotFound)
        }
    }

    /// 更新用户登录信息
    pub async fn update_login_info(pool: &DbPool, user_id: i64, ip: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE clip_users
            SET last_login_at = NOW(), last_login_ip = ?, login_count = login_count + 1
            WHERE id = ?
            "#,
        )
            .bind(ip)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// 会话相关操作
pub struct SessionRepository;

impl SessionRepository {
    /// 创建会话
    pub async fn create_session(
        pool: &DbPool,
        user_id: i64,
        token: &str,
        refresh_token: &str,
        expires_at: DateTime<Utc>,
        refresh_expires_at: DateTime<Utc>,
        ip: &str,
        device_info: Option<String>,
    ) -> Result<UserSession, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO clip_user_sessions (user_id, token, refresh_token, expires_at, refresh_expires_at, ip_address, device_info)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            user_id,
            token,
            refresh_token,
            expires_at,
            refresh_expires_at,
            ip,
            device_info
        )
            .execute(pool)
            .await?;

        Self::find_by_id(pool, result.last_insert_id() as i64).await
    }

    /// 根据ID查找会话
    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<UserSession, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, user_id, token, refresh_token, expires_at, refresh_expires_at, device_info, ip_address, created_at
            FROM clip_user_sessions
            WHERE id = ?
            "#,
        )
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// 根据token查找会话
    pub async fn find_by_token(pool: &DbPool, token: &str) -> Result<UserSession, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, user_id, token, refresh_token, expires_at, refresh_expires_at, device_info, ip_address, created_at
            FROM clip_user_sessions
            WHERE token = ? AND expires_at > NOW()
            "#,
        )
            .bind(token)
            .fetch_one(pool)
            .await
    }

    /// 删除会话
    pub async fn delete_session(pool: &DbPool, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM clip_user_sessions WHERE token = ?
            "#,
        )
            .bind(token)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// Clip 内容相关操作
pub struct ClipRepository;

impl ClipRepository {
    /// 创建 Clip
    pub async fn create_clip(pool: &DbPool, user_id: i64, clip_data: &CreateClip) -> Result<ClipContent, sqlx::Error> {
        let short_url = Some(Uuid::new_v4().to_string()[..8].to_string());
        let tags_json = clip_data.tags.as_ref().map(|tags| serde_json::to_value(tags).unwrap());

        let result = sqlx::query!(
            r#"
            INSERT INTO clip_contents (user_id, title, content, content_type, language, is_encrypted, access_type, short_url, tags)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            user_id,
            clip_data.title,
            clip_data.content,
            clip_data.content_type.as_deref().unwrap_or("text"),
            clip_data.language,
            clip_data.is_encrypted.unwrap_or(false) as i8,
            clip_data.access_type.as_deref().unwrap_or("private"),
            short_url,
            tags_json
        )
            .execute(pool)
            .await?;

        Self::find_by_id(pool, result.last_insert_id() as i64).await
    }

    /// 根据ID查找 Clip
    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<ClipContent, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, user_id, title, content, content_type, language, is_encrypted, encryption_key,
                   access_type, view_count, expires_at, short_url, tags, created_at, updated_at
            FROM clip_contents
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// 根据短链接查找 Clip
    pub async fn find_by_short_url(pool: &DbPool, short_url: &String) -> Result<ClipContent, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, user_id, title, content, content_type, language, is_encrypted, encryption_key,
                   access_type, view_count, expires_at, short_url, tags, created_at, updated_at
            FROM clip_contents
            WHERE short_url = ? AND deleted_at IS NULL AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
            .bind(short_url)
            .fetch_one(pool)
            .await
    }

    /// 获取用户的 Clips
    pub async fn find_by_user_id(pool: &DbPool, user_id: i64, page: i64, page_size: i64) -> Result<Vec<ClipContent>, sqlx::Error> {
        let offset = (page - 1) * page_size;

        sqlx::query_as(
            r#"
            SELECT id, user_id, title, content, content_type, language, is_encrypted, encryption_key,
                   access_type, view_count, expires_at, short_url, tags, created_at, updated_at
            FROM clip_contents
            WHERE user_id = ? AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
            .bind(user_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    /// 更新 Clip
    pub async fn update_clip(pool: &DbPool, id: i64, user_id: i64, clip_data: &UpdateClip) -> Result<ClipContent, sqlx::Error> {
        let tags_json = clip_data.tags.as_ref().map(|tags| serde_json::to_value(tags).unwrap());

        sqlx::query(
            r#"
            UPDATE clip_contents
            SET title = COALESCE(?, title),
                content = COALESCE(?, content),
                content_type = COALESCE(?, content_type),
                language = ?,
                access_type = COALESCE(?, access_type),
                expires_at = ?,
                tags = COALESCE(?, tags)
            WHERE id = ? AND user_id = ?
            "#,
        )
            .bind(&clip_data.title)
            .bind(&clip_data.content)
            .bind(&clip_data.content_type)
            .bind(&clip_data.language)
            .bind(&clip_data.access_type)
            .bind(&clip_data.expires_at)
            .bind(&tags_json)
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Self::find_by_id(pool, id).await
    }

    /// 删除 Clip（软删除）
    pub async fn delete_clip(pool: &DbPool, id: i64, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE clip_contents SET deleted_at = NOW() WHERE id = ? AND user_id = ?
            "#,
        )
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// 增加查看次数
    pub async fn increment_view_count(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE clip_contents SET view_count = view_count + 1 WHERE id = ?
            "#,
        )
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}