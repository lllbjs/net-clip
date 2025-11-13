use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{
    database::{
        models::{ApiResponse, CreateClip, UpdateClip},
        ClipRepository, DbPool
    },
};

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 创建 Clip
pub async fn create_clip(
    State(pool): State<DbPool>,
    Json(clip_data): Json<CreateClip>,
    user_id: i64,
) -> impl IntoResponse {
    match ClipRepository::create_clip(&pool, user_id, &clip_data).await {
        Ok(clip) => {
            let response = ApiResponse::success(clip, "Clip 创建成功");
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            tracing::error!("创建 Clip 失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("创建 Clip 失败")))
        }
    }
}

/// 获取用户的 Clips
pub async fn get_user_clips(
    State(pool): State<DbPool>,
    Query(pagination): Query<Pagination>,
    user_id: i64,
) -> impl IntoResponse {
    let page = pagination.page.unwrap_or(1);
    let page_size = pagination.page_size.unwrap_or(20);

    if page < 1 || page_size < 1 || page_size > 100 {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::error("分页参数无效")));
    }

    match ClipRepository::find_by_user_id(&pool, user_id, page, page_size).await {
        Ok(clips) => {
            let response = ApiResponse::success(clips, "获取 Clips 成功");
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("获取用户 Clips 失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("获取 Clips 失败")))
        }
    }
}

/// 根据 ID 获取 Clip
pub async fn get_clip_by_id(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match ClipRepository::find_by_id(&pool, id).await {
        Ok(clip) => {
            // 增加查看次数
            if let Err(e) = ClipRepository::increment_view_count(&pool, id).await {
                tracing::error!("增加查看次数失败: {}", e);
            }

            let response = ApiResponse::success(clip, "获取 Clip 成功");
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("获取 Clip 失败: {}", e);
            (StatusCode::NOT_FOUND, Json(ApiResponse::error("Clip 不存在")))
        }
    }
}

/// 根据短链接获取 Clip
pub async fn get_clip_by_short_url(
    State(pool): State<DbPool>,
    Path(short_url): Path<String>,
) -> impl IntoResponse {
    match ClipRepository::find_by_short_url(&pool, &short_url).await {
        Ok(clip) => {
            // 增加查看次数
            if let Err(e) = ClipRepository::increment_view_count(&pool, clip.id).await {
                tracing::error!("增加查看次数失败: {}", e);
            }

            let response = ApiResponse::success(clip, "获取 Clip 成功");
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("获取 Clip 失败: {}", e);
            (StatusCode::NOT_FOUND, Json(ApiResponse::error("Clip 不存在或已过期")))
        }
    }
}

/// 更新 Clip
pub async fn update_clip(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Json(clip_data): Json<UpdateClip>,
    user_id: i64,
) -> impl IntoResponse {
    match ClipRepository::update_clip(&pool, id, user_id, &clip_data).await {
        Ok(clip) => {
            let response = ApiResponse::success(clip, "Clip 更新成功");
            (StatusCode::OK, Json(response))
        }
        Err(sqlx::Error::RowNotFound) => {
            (StatusCode::NOT_FOUND, Json(ApiResponse::error("Clip 不存在或无权访问")))
        }
        Err(e) => {
            tracing::error!("更新 Clip 失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("更新 Clip 失败")))
        }
    }
}

/// 删除 Clip
pub async fn delete_clip(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    user_id: i64,
) -> impl IntoResponse {
    match ClipRepository::delete_clip(&pool, id, user_id).await {
        Ok(_) => {
            let response = ApiResponse::success((), "Clip 删除成功");
            (StatusCode::OK, Json(response))
        }
        Err(sqlx::Error::RowNotFound) => {
            (StatusCode::NOT_FOUND, Json(ApiResponse::error("Clip 不存在或无权访问")))
        }
        Err(e) => {
            tracing::error!("删除 Clip 失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("删除 Clip 失败")))
        }
    }
}