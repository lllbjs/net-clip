use axum::{
    extract::{State, Json},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use serde_json::json;
use std::net::SocketAddr;

use crate::{
    database::{models::{ApiResponse, CreateUser, LoginUser, LoginResponse, TokenClaims}, UserRepository, SessionRepository, DbPool},
    config::Config,
};

/// 用户注册
pub async fn register(
    State(pool): State<DbPool>,
    Json(user_data): Json<CreateUser>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // 检查用户名是否已存在
    if UserRepository::find_by_username(&pool, &user_data.username.as_str()).await.is_ok() {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error("用户名已存在"))));
    }

    // 创建用户
    match UserRepository::create_user(&pool, &user_data).await {
        Ok(user) => {
            let response = ApiResponse::success(user, "用户注册成功");
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            tracing::error!("用户注册失败: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("用户注册失败"))))
        }
    }
}

/// 用户登录
pub async fn login(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    addr: Option<axum::extract::ConnectInfo<SocketAddr>>,
    headers: HeaderMap,
    Json(login_data): Json<LoginUser>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    let ip = addr.map(|ci| ci.0.ip().to_string()).unwrap_or_else(|| "unknown".to_string());
    let device_info = headers.get("user-agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|s| s.to_string());

    // 验证用户密码
    let user = match UserRepository::verify_password(&pool, &login_data.username, &login_data.password).await {
        Ok(user) => user,
        Err(_) => {
            return Err((StatusCode::UNAUTHORIZED, Json(ApiResponse::error("用户名或密码错误"))));
        }
    };

    // 检查用户状态
    if user.status == 0 {
        return Err((StatusCode::FORBIDDEN, Json(ApiResponse::error("用户已被禁用"))));
    }

    // 生成 JWT token
    let now = Utc::now();
    let expires_at = now + Duration::seconds(config.jwt_expires_in);
    let refresh_expires_at = now + Duration::seconds(config.jwt_refresh_expires_in);

    let claims = TokenClaims {
        sub: user.id,
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    ).map_err(|e| {
        tracing::error!("JWT token 生成失败: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("登录失败")))
    })?;

    let refresh_claims = TokenClaims {
        sub: user.id,
        exp: refresh_expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let refresh_token = encode(
        &Header::new(Algorithm::HS256),
        &refresh_claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    ).map_err(|e| {
        tracing::error!("Refresh token 生成失败: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("登录失败")))
    })?;

    // 创建会话
    match SessionRepository::create_session(
        &pool,
        user.id,
        &token,
        &refresh_token,
        expires_at,
        refresh_expires_at,
        &ip,
        device_info,
    ).await {
        Ok(_) => {
            // 更新用户登录信息
            if let Err(e) = UserRepository::update_login_info(&pool, user.id, &ip).await {
                tracing::error!("更新用户登录信息失败: {}", e);
            }

            let login_response = LoginResponse {
                user,
                access_token: token,
                refresh_token,
                expires_in: config.jwt_expires_in,
            };

            let response = ApiResponse::success(login_response, "登录成功");
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            tracing::error!("创建会话失败: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("登录失败"))))
        }
    }
}

/// 刷新 token
pub async fn refresh_token(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    let auth_header = headers.get("authorization")
        .and_then(|header| header.to_str().ok());

    let refresh_token = if let Some(header) = auth_header {
        if header.starts_with("Bearer ") {
            &header[7..]
        } else {
            return Err((StatusCode::UNAUTHORIZED, Json(ApiResponse::error("无效的 token 格式"))));
        }
    } else {
        return Err((StatusCode::UNAUTHORIZED, Json(ApiResponse::error("缺少认证头"))));
    };

    // 验证 refresh token
    let session = match SessionRepository::find_by_token(&pool, refresh_token).await {
        Ok(session) => session,
        Err(_) => {
            return Err((StatusCode::UNAUTHORIZED, Json(ApiResponse::error("无效的 refresh token"))));
        }
    };

    // 生成新的 access token
    let now = Utc::now();
    let expires_at = now + Duration::seconds(config.jwt_expires_in);

    let claims = TokenClaims {
        sub: session.user_id,
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let new_token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    ).map_err(|e| {
        tracing::error!("JWT token 生成失败: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("token 刷新失败")))
    })?;

    let response = json!({
        "status": "success",
        "data": {
            "access_token": new_token,
            "expires_in": config.jwt_expires_in,
            "token_type": "Bearer"
        },
        "message": "token 刷新成功"
    });

    Ok((StatusCode::OK, Json(response)))
}

/// 退出登录
pub async fn logout(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let auth_header = headers.get("authorization")
        .and_then(|header| header.to_str().ok());

    let token = if let Some(header) = auth_header {
        if header.starts_with("Bearer ") {
            &header[7..]
        } else {
            return (StatusCode::UNAUTHORIZED, Json(ApiResponse::<()>::error("无效的 token 格式")));
        }
    } else {
        return (StatusCode::UNAUTHORIZED, Json(ApiResponse::<()>::error("缺少认证头")));
    };

    match SessionRepository::delete_session(&pool, token).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success((), "退出登录成功"))),
        Err(e) => {
            tracing::error!("删除会话失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("退出登录失败")))
        }
    }
}

/// 获取当前用户信息
pub async fn get_me(
    user_id: i64,
    State(pool): State<DbPool>,
) -> impl IntoResponse {
    match UserRepository::find_by_id(&pool, user_id).await {
        Ok(user) => {
            // 不返回密码等敏感信息
            let user_response = json!({
                "id": user.id,
                "username": user.username,
                "email": user.email,
                "status": user.status,
                "last_login_at": user.last_login_at,
                "login_count": user.login_count,
                "created_at": user.created_at
            });

            let response = ApiResponse::success(user_response, "获取用户信息成功");
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("获取用户信息失败: {}", e);
            (StatusCode::NOT_FOUND, Json(ApiResponse::error("用户不存在")))
        }
    }
}