use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use validator::Validate;

use crate::{
    handlers::{success_response, success_response_with_message, AppError, Result},
    middleware::auth::get_user_from_token,
    models::{
        UpdateConfigRequest, TestConfigRequest, ConfigTestEmailRequest, RollbackConfigRequest,
        AdminPaginationQuery,
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

/// Get current configuration
pub async fn get_config(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let config = state
        .config_service
        .get_current_config()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get configuration: {}", e)))?;

    Ok(success_response(config))
}

/// Update configuration
pub async fn update_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<UpdateConfigRequest>,
) -> Result<Json<serde_json::Value>> {
    let (admin_id, _admin_email) = require_admin(&headers, &state).await?;

    payload.validate()?;

    let ip_address = get_client_ip(&headers);

    let version = state
        .config_service
        .update_config(admin_id, payload, ip_address)
        .await
        .map_err(|e| AppError::BadRequest(format!("Configuration update failed: {}", e)))?;

    let response = serde_json::json!({
        "version": version,
        "applied_at": chrono::Utc::now(),
    });

    Ok(success_response_with_message(
        response,
        "Configuration updated successfully",
    ))
}

/// Test configuration
pub async fn test_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<TestConfigRequest>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let result = state
        .config_service
        .test_config(payload)
        .await
        .map_err(|e| AppError::BadRequest(format!("Configuration test failed: {}", e)))?;

    Ok(success_response(result))
}

/// Send test email
pub async fn test_email(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<ConfigTestEmailRequest>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    payload.validate()?;

    let body = payload.body.unwrap_or_else(|| {
        "This is a test email from GeekTools Plugin Marketplace configuration system.".to_string()
    });

    match state
        .smtp_service
        .send_test_email(&payload.recipient, &payload.subject, &body)
        .await
    {
        Ok(_) => {
            let response = serde_json::json!({
                "recipient": payload.recipient,
                "subject": payload.subject,
                "sent_at": chrono::Utc::now(),
            });

            Ok(success_response_with_message(
                response,
                "Test email sent successfully",
            ))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "recipient": payload.recipient,
                "subject": payload.subject,
                "error": e.to_string(),
                "tested_at": chrono::Utc::now(),
            });

            Err(AppError::BadRequest(format!("Test email failed: {}", e)))
        }
    }
}

/// Get configuration history
pub async fn get_config_history(
    headers: HeaderMap,
    Query(params): Query<serde_json::Value>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let limit: Option<i64> = params
        .get("limit")
        .and_then(|v| v.as_i64())
        .map(|v| v as i64);

    let history = state
        .config_service
        .get_config_history(limit)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get configuration history: {}", e)))?;

    Ok(success_response(history))
}

/// Rollback configuration to a previous version
pub async fn rollback_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<RollbackConfigRequest>,
) -> Result<Json<serde_json::Value>> {
    let (admin_id, _admin_email) = require_admin(&headers, &state).await?;

    payload.validate()?;

    let ip_address = get_client_ip(&headers);

    let version = state
        .config_service
        .rollback_config(admin_id, payload, ip_address)
        .await
        .map_err(|e| AppError::BadRequest(format!("Configuration rollback failed: {}", e)))?;

    let response = serde_json::json!({
        "version": version,
        "rolled_back_at": chrono::Utc::now(),
    });

    Ok(success_response_with_message(
        response,
        "Configuration rolled back successfully",
    ))
}

/// Create configuration snapshot
pub async fn create_config_snapshot(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let (admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let description = payload
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let ip_address = get_client_ip(&headers);

    let version = state
        .config_service
        .create_config_snapshot(admin_id, description, ip_address)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create configuration snapshot: {}", e)))?;

    let response = serde_json::json!({
        "version": version,
        "created_at": chrono::Utc::now(),
    });

    Ok(success_response_with_message(
        response,
        "Configuration snapshot created successfully",
    ))
}

/// Compare configuration versions
pub async fn compare_config_versions(
    headers: HeaderMap,
    Query(params): Query<serde_json::Value>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let version1_id = params
        .get("version1")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or_else(|| AppError::BadRequest("version1 parameter is required".to_string()))?;

    let version2_id = params
        .get("version2")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or_else(|| AppError::BadRequest("version2 parameter is required".to_string()))?;

    // Get both configuration versions
    let history1 = sqlx::query!(
        "SELECT * FROM config_history WHERE id = $1",
        version1_id
    )
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Configuration version 1 not found".to_string()))?;

    let history2 = sqlx::query!(
        "SELECT * FROM config_history WHERE id = $1",
        version2_id
    )
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Configuration version 2 not found".to_string()))?;

    let response = serde_json::json!({
        "version1": {
            "id": history1.id,
            "config_type": history1.config_type,
            "config": history1.new_config,
            "changed_by_id": history1.changed_by_id,
            "changed_at": history1.changed_at,
            "version": history1.version,
        },
        "version2": {
            "id": history2.id,
            "config_type": history2.config_type,
            "config": history2.new_config,
            "changed_by_id": history2.changed_by_id,
            "changed_at": history2.changed_at,
            "version": history2.version,
        },
        "comparison": {
            "same_type": history1.config_type == history2.config_type,
            "config_diff": generate_config_diff(&history1.new_config, &history2.new_config),
        }
    });

    Ok(success_response(response))
}

// Helper function to generate configuration differences
fn generate_config_diff(
    config1: &serde_json::Value,
    config2: &serde_json::Value,
) -> serde_json::Value {
    use serde_json::Value;
    
    let mut changes = Vec::new();
    
    // Simple diff implementation
    if let (Value::Object(obj1), Value::Object(obj2)) = (config1, config2) {
        for (key, value1) in obj1 {
            if let Some(value2) = obj2.get(key) {
                if value1 != value2 {
                    changes.push(serde_json::json!({
                        "field": key,
                        "type": "modified",
                        "old_value": value1,
                        "new_value": value2,
                    }));
                }
            } else {
                changes.push(serde_json::json!({
                    "field": key,
                    "type": "removed",
                    "old_value": value1,
                }));
            }
        }
        
        for (key, value2) in obj2 {
            if !obj1.contains_key(key) {
                changes.push(serde_json::json!({
                    "field": key,
                    "type": "added",
                    "new_value": value2,
                }));
            }
        }
    }
    
    serde_json::json!({
        "changes": changes,
        "total_changes": changes.len(),
    })
}