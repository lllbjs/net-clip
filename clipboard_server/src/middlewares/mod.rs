use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::json;

use crate::{
    database::{models::TokenClaims, SessionRepository, DbPool},
    config::Config,
};

/// 认证中间件
pub async fn auth(
    State((pool, config)): State<(DbPool, Config)>,
    headers: axum::http::HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, axum::Json<serde_json::Value>)> {
    let token = extract_token_from_header(&headers)
        .ok_or_else(|| (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "status": "error",
                "message": "缺少认证令牌"
            }))
        ))?;

    // 验证 token 是否在会话表中
    if SessionRepository::find_by_token(&pool, &token).await.is_err() {
        return Err((
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "status": "error",
                "message": "令牌已过期或无效"
            }))
        ));
    }

    // 验证 JWT token
    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
        .map_err(|_| (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
            "status": "error",
            "message": "无效的令牌"
        }))
        ))?
        .claims;

    // 将用户ID添加到请求扩展中
    request.extensions_mut().insert(claims.sub);

    Ok(next.run(request).await)
}

/// 从请求头中提取 token
fn extract_token_from_header(headers: &axum::http::HeaderMap) -> Option<String> {
    let auth_header = headers.get("authorization")?.to_str().ok()?;

    if auth_header.starts_with("Bearer ") {
        Some(auth_header[7..].to_string())
    } else {
        None
    }
}