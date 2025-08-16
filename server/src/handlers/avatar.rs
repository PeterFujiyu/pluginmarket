use axum::{
    extract::{Multipart, Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    body::Body,
    Json,
};
use std::path::PathBuf;
use tokio::fs;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::middleware::auth::Claims;
use crate::models::user::{AvatarUploadResponse, UserAvatar};
use crate::services::AppState;

// 允许的图片MIME类型
const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/jpg", 
    "image/png",
    "image/gif",
    "image/webp"
];

// 最大文件大小 (5MB)
const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;

/// 上传用户头像
pub async fn upload_avatar(
    State(state): State<AppState>,
    claims: Claims,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let user_id = claims.user_id;

    // 解析multipart数据
    let mut avatar_data: Option<(String, Vec<u8>)> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "avatar" {
            let filename = field.file_name().unwrap_or("avatar").to_string();
            let content_type = field.content_type().unwrap_or("").to_string();
            
            // 验证文件类型
            if !ALLOWED_MIME_TYPES.contains(&content_type.as_str()) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(AvatarUploadResponse {
                        success: false,
                        message: "不支持的文件类型。请使用 JPG、PNG、GIF 或 WebP 格式".to_string(),
                        avatar_url: None,
                    }),
                );
            }

            // 读取文件数据
            let data = match field.bytes().await {
                Ok(bytes) => bytes.to_vec(),
                Err(_) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(AvatarUploadResponse {
                            success: false,
                            message: "文件读取失败".to_string(),
                            avatar_url: None,
                        }),
                    );
                }
            };

            // 检查文件大小
            if data.len() > MAX_FILE_SIZE {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(AvatarUploadResponse {
                        success: false,
                        message: "文件太大，最大支持 5MB".to_string(),
                        avatar_url: None,
                    }),
                );
            }

            avatar_data = Some((content_type, data));
            break;
        }
    }

    let (mime_type, file_data) = match avatar_data {
        Some(data) => data,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(AvatarUploadResponse {
                    success: false,
                    message: "未找到头像文件".to_string(),
                    avatar_url: None,
                }),
            );
        }
    };

    // 生成唯一文件名
    let file_extension = match mime_type.as_str() {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "jpg",
    };
    
    let filename = format!("avatar_{}_{}.{}", user_id, Uuid::new_v4(), file_extension);
    
    // 确保头像目录存在
    let avatar_dir = PathBuf::from("uploads/avatars");
    if let Err(_) = fs::create_dir_all(&avatar_dir).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AvatarUploadResponse {
                success: false,
                message: "无法创建头像目录".to_string(),
                avatar_url: None,
            }),
        );
    }

    let file_path = avatar_dir.join(&filename);
    
    // 保存文件
    if let Err(_) = fs::write(&file_path, &file_data).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AvatarUploadResponse {
                success: false,
                message: "文件保存失败".to_string(),
                avatar_url: None,
            }),
        );
    }

    // 保存到数据库 (依赖数据库触发器自动处理旧头像)
    let file_path_str = file_path.to_string_lossy().to_string();
    let insert_result = sqlx::query!(
        r#"
        INSERT INTO user_avatars (user_id, file_name, file_path, file_size, mime_type, is_active)
        VALUES ($1, $2, $3, $4, $5, true)
        RETURNING id
        "#,
        user_id,
        filename,
        file_path_str,
        file_data.len() as i32,
        mime_type
    )
    .fetch_one(&state.db_pool)
    .await;

    match insert_result {
        Ok(_) => {
            let avatar_url = format!("/api/v1/avatars/{}", filename);
            (
                StatusCode::OK,
                Json(AvatarUploadResponse {
                    success: true,
                    message: "头像上传成功".to_string(),
                    avatar_url: Some(avatar_url),
                }),
            )
        }
        Err(e) => {
            // 删除已保存的文件
            let _ = fs::remove_file(&file_path).await;
            
            tracing::error!("数据库保存头像信息失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AvatarUploadResponse {
                    success: false,
                    message: "头像信息保存失败".to_string(),
                    avatar_url: None,
                }),
            )
        }
    }
}

/// 获取头像文件
pub async fn get_avatar(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Response {
    // 验证文件名安全性
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return StatusCode::BAD_REQUEST.into_response();
    }

    // 查询数据库确认文件存在
    let avatar_result = sqlx::query_as!(
        UserAvatar,
        "SELECT * FROM user_avatars WHERE file_name = $1 AND is_active = true",
        filename
    )
    .fetch_optional(&state.db_pool)
    .await;

    let avatar = match avatar_result {
        Ok(Some(avatar)) => avatar,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // 检查文件是否存在
    let file_path = PathBuf::from(&avatar.file_path);
    if !file_path.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    // 读取文件并返回
    match tokio::fs::File::open(&file_path).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);
            
            let mut response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, &avatar.mime_type)
                .header(header::CACHE_CONTROL, "public, max-age=86400") // 缓存1天
                .body(body)
                .unwrap();
            
            response
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// 删除用户头像
pub async fn delete_avatar(
    State(state): State<AppState>,
    claims: Claims,
) -> impl IntoResponse {
    let user_id = claims.user_id;

    // 查找当前活跃的头像
    let avatar_result = sqlx::query_as!(
        UserAvatar,
        "SELECT * FROM user_avatars WHERE user_id = $1 AND is_active = true",
        user_id
    )
    .fetch_optional(&state.db_pool)
    .await;

    let avatar = match avatar_result {
        Ok(Some(avatar)) => avatar,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(AvatarUploadResponse {
                    success: false,
                    message: "未找到活跃的头像".to_string(),
                    avatar_url: None,
                }),
            );
        }
        Err(e) => {
            tracing::error!("查询头像失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AvatarUploadResponse {
                    success: false,
                    message: "查询头像失败".to_string(),
                    avatar_url: None,
                }),
            );
        }
    };

    // 从数据库删除头像记录
    let delete_result = sqlx::query!(
        "DELETE FROM user_avatars WHERE id = $1",
        avatar.id
    )
    .execute(&state.db_pool)
    .await;

    match delete_result {
        Ok(_) => {
            // 删除文件
            let file_path = PathBuf::from(&avatar.file_path);
            if file_path.exists() {
                let _ = fs::remove_file(&file_path).await;
            }

            (
                StatusCode::OK,
                Json(AvatarUploadResponse {
                    success: true,
                    message: "头像删除成功".to_string(),
                    avatar_url: None,
                }),
            )
        }
        Err(e) => {
            tracing::error!("删除头像失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AvatarUploadResponse {
                    success: false,
                    message: "删除头像失败".to_string(),
                    avatar_url: None,
                }),
            )
        }
    }
}

/// 获取用户头像信息
pub async fn get_user_avatar_info(
    State(state): State<AppState>,
    claims: Claims,
) -> impl IntoResponse {
    let user_id = claims.user_id;

    let avatar_result = sqlx::query_as!(
        UserAvatar,
        "SELECT * FROM user_avatars WHERE user_id = $1 AND is_active = true",
        user_id
    )
    .fetch_optional(&state.db_pool)
    .await;

    match avatar_result {
        Ok(Some(avatar)) => {
            let avatar_url = format!("/api/v1/avatars/{}", avatar.file_name);
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "avatar_url": avatar_url,
                    "file_size": avatar.file_size,
                    "upload_time": avatar.upload_time
                })),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "message": "未找到头像"
            })),
        ),
        Err(e) => {
            tracing::error!("查询头像信息失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "查询失败"
                })),
            )
        }
    }
}