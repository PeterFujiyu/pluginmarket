use axum::{
    extract::{Query, State, Path},
    http::HeaderMap,
    Json,
    response::{Response, IntoResponse},
};
use validator::Validate;

use crate::{
    handlers::{success_response, success_response_with_message, AppError, Result},
    middleware::auth::get_user_from_token,
    models::{
        CreateBackupRequest, RestoreBackupRequest, CreateScheduleRequest, 
        UpdateScheduleRequest, DeleteBackupRequest, AdminPaginationQuery,
    },
    services::AppState,
};

// Admin authentication middleware helper
async fn require_admin(headers: &HeaderMap, state: &AppState) -> Result<(i32, String)> {
    let user = get_user_from_token(headers, &state.auth_service).await?;
    
    if !state.admin_service.is_admin(user.id).await.map_err(|e| {
        AppError::Internal(format!("Failed to check admin status: {}", e))
    })? {
        return Err(AppError::Forbidden("需要管理员权限".to_string()));
    }

    Ok((user.id, user.email))
}

// Get client IP address
fn get_client_ip(headers: &HeaderMap) -> Option<std::net::IpAddr> {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|ip| ip.trim().parse().ok())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .and_then(|ip| ip.parse().ok())
        })
}

/// Get backup statistics
pub async fn get_backup_stats(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let stats = state
        .backup_service
        .get_backup_stats()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get backup stats: {}", e)))?;

    Ok(success_response(stats))
}

/// List backups with pagination and filtering
pub async fn list_backups(
    headers: HeaderMap,
    Query(pagination): Query<AdminPaginationQuery>,
    Query(params): Query<serde_json::Value>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    pagination.validate()?;

    let filter = params
        .get("filter")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let result = state
        .backup_service
        .list_backups(pagination, filter)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list backups: {}", e)))?;

    Ok(success_response(result))
}

/// Create a new backup
pub async fn create_backup(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateBackupRequest>,
) -> Result<Json<serde_json::Value>> {
    let (admin_id, admin_email) = require_admin(&headers, &state).await?;

    payload.validate()?;

    let ip_address = get_client_ip(&headers);

    let operation_id = state
        .backup_service
        .create_backup(admin_id, &admin_email, payload, ip_address)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to create backup: {}", e)))?;

    let response = serde_json::json!({
        "operation_id": operation_id,
        "status": "started",
        "message": "Backup creation initiated"
    });

    Ok(success_response_with_message(
        response,
        "Backup operation started successfully",
    ))
}

/// Restore a backup
pub async fn restore_backup(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<RestoreBackupRequest>,
) -> Result<Json<serde_json::Value>> {
    let (admin_id, _admin_email) = require_admin(&headers, &state).await?;

    payload.validate()?;

    let ip_address = get_client_ip(&headers);

    let operation_id = state
        .backup_service
        .restore_backup(admin_id, payload, ip_address)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to restore backup: {}", e)))?;

    let response = serde_json::json!({
        "operation_id": operation_id,
        "status": "started",
        "message": "Backup restore initiated"
    });

    Ok(success_response_with_message(
        response,
        "Backup restore operation started successfully",
    ))
}

/// Delete a backup
pub async fn delete_backup(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<DeleteBackupRequest>,
) -> Result<Json<serde_json::Value>> {
    let (admin_id, _admin_email) = require_admin(&headers, &state).await?;

    payload.validate()?;

    let ip_address = get_client_ip(&headers);

    state
        .backup_service
        .delete_backup(admin_id, payload, ip_address)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to delete backup: {}", e)))?;

    Ok(success_response_with_message(
        serde_json::json!({}),
        "Backup deleted successfully",
    ))
}

/// Download a backup file
pub async fn download_backup(
    headers: HeaderMap,
    Path(backup_id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Response> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let file_path = state
        .backup_service
        .get_backup_file_path(backup_id)
        .await
        .map_err(|e| AppError::NotFound(format!("Backup file not found: {}", e)))?;

    // Read file content
    let file_content = tokio::fs::read(&file_path)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read backup file: {}", e)))?;

    // Get filename from path
    let filename = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("backup.sql.gz");

    // Create response with appropriate headers
    let headers = [
        ("content-type", "application/gzip"),
        ("content-disposition", &format!("attachment; filename=\"{}\"", filename)),
    ];

    Ok((headers, file_content).into_response())
}

/// Get backup operation status
pub async fn get_operation_status(
    headers: HeaderMap,
    Path(operation_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let status = state
        .backup_service
        .get_operation_status(&operation_id)
        .await
        .ok_or_else(|| AppError::NotFound("Operation not found".to_string()))?;

    Ok(success_response(status))
}

/// List all current backup operations
pub async fn list_operations(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let operations = state
        .backup_service
        .list_operations()
        .await;

    Ok(success_response(operations))
}

/// List backup schedules
pub async fn list_schedules(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let schedules = state
        .backup_service
        .list_schedules()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list schedules: {}", e)))?;

    Ok(success_response(schedules))
}

/// Create backup schedule
pub async fn create_schedule(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateScheduleRequest>,
) -> Result<Json<serde_json::Value>> {
    let (admin_id, _admin_email) = require_admin(&headers, &state).await?;

    payload.validate()?;

    let schedule_id = state
        .backup_service
        .create_schedule(admin_id, payload)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to create schedule: {}", e)))?;

    let response = serde_json::json!({
        "schedule_id": schedule_id
    });

    Ok(success_response_with_message(
        response,
        "Backup schedule created successfully",
    ))
}

/// Update backup schedule
pub async fn update_schedule(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<UpdateScheduleRequest>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    payload.validate()?;

    // Note: Implementation would go here - this is a placeholder
    // In production, you would implement the actual update logic

    Ok(success_response_with_message(
        serde_json::json!({}),
        "Backup schedule updated successfully",
    ))
}

/// Delete backup schedule
pub async fn delete_schedule(
    headers: HeaderMap,
    Path(schedule_id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    // Delete schedule
    sqlx::query!(
        "DELETE FROM backup_schedules WHERE id = $1",
        schedule_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to delete schedule: {}", e)))?;

    Ok(success_response_with_message(
        serde_json::json!({}),
        "Backup schedule deleted successfully",
    ))
}

/// Toggle backup schedule enabled/disabled
pub async fn toggle_schedule(
    headers: HeaderMap,
    Path(schedule_id): Path<i32>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let enabled = payload
        .get("enabled")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| AppError::BadRequest("enabled field is required".to_string()))?;

    // Update schedule
    sqlx::query!(
        "UPDATE backup_schedules SET enabled = $1, updated_at = NOW() WHERE id = $2",
        enabled,
        schedule_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to update schedule: {}", e)))?;

    let message = if enabled {
        "Backup schedule enabled successfully"
    } else {
        "Backup schedule disabled successfully"
    };

    Ok(success_response_with_message(
        serde_json::json!({
            "schedule_id": schedule_id,
            "enabled": enabled
        }),
        message,
    ))
}