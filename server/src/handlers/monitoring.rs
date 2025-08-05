use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use validator::Validate;

use crate::{
    handlers::{success_response, success_response_with_message, AppError, Result},
    middleware::auth::get_user_from_token,
    models::{monitoring::TestEmailRequest as MonitoringTestEmailRequest, SystemLogsQuery},
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

/// Get system metrics (CPU, memory, disk, network)
pub async fn get_system_metrics(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let metrics = state
        .monitoring_service
        .get_system_metrics()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get system metrics: {}", e)))?;

    Ok(success_response(metrics))
}

/// Get services health status
pub async fn get_services_status(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let status = state
        .monitoring_service
        .get_services_status()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get services status: {}", e)))?;

    Ok(success_response(status))
}

/// Get database status and metrics
pub async fn get_database_status(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let status = state
        .monitoring_service
        .get_database_status()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get database status: {}", e)))?;

    Ok(success_response(status))
}

/// Get SMTP service status
pub async fn get_smtp_status(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let status = state
        .monitoring_service
        .get_smtp_status()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get SMTP status: {}", e)))?;

    Ok(success_response(status))
}

/// Get system logs with filtering and pagination
pub async fn get_system_logs(
    headers: HeaderMap,
    Query(params): Query<SystemLogsQuery>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    params.validate()?;

    let logs = state
        .monitoring_service
        .get_system_logs(params)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get system logs: {}", e)))?;

    Ok(success_response(logs))
}

/// Test database connection
pub async fn test_database_connection(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let result = state
        .monitoring_service
        .test_database_connection()
        .await
        .map_err(|e| AppError::Internal(format!("Database test failed: {}", e)))?;

    Ok(success_response(result))
}

/// Test SMTP connection by sending a test email
pub async fn test_smtp_connection(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<MonitoringTestEmailRequest>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    payload.validate()?;

    let result = state
        .monitoring_service
        .test_smtp_connection(payload)
        .await
        .map_err(|e| AppError::Internal(format!("SMTP test failed: {}", e)))?;

    Ok(success_response(result))
}

/// Get performance chart data
pub async fn get_performance_chart_data(
    headers: HeaderMap,
    Query(params): Query<serde_json::Value>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let hours = params
        .get("hours")
        .and_then(|v| v.as_i64())
        .map(|h| h as i32);

    let chart_data = state
        .monitoring_service
        .get_performance_chart_data(hours)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get performance data: {}", e)))?;

    Ok(success_response(chart_data))
}

/// Record current system metrics
pub async fn record_system_metrics(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    // Get current metrics
    let metrics = state
        .monitoring_service
        .get_system_metrics()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get system metrics: {}", e)))?;

    // Record them to database
    state
        .monitoring_service
        .record_system_metrics(&metrics)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to record system metrics: {}", e)))?;

    Ok(success_response_with_message(
        serde_json::json!({
            "recorded_at": metrics.timestamp,
            "metrics_count": 9 // CPU, memory, disk metrics
        }),
        "System metrics recorded successfully",
    ))
}

/// Clean up old monitoring data
pub async fn cleanup_old_data(
    headers: HeaderMap,
    Query(params): Query<serde_json::Value>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let days_to_keep = params
        .get("days")
        .and_then(|v| v.as_i64())
        .map(|d| d as i32)
        .unwrap_or(30)
        .clamp(1, 365);

    state
        .monitoring_service
        .cleanup_old_data(days_to_keep)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to cleanup old data: {}", e)))?;

    Ok(success_response_with_message(
        serde_json::json!({
            "days_kept": days_to_keep,
            "cleaned_up_at": chrono::Utc::now()
        }),
        "Old monitoring data cleaned up successfully",
    ))
}

/// Log a custom system event
pub async fn log_system_event(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let (admin_id, _admin_email) = require_admin(&headers, &state).await?;

    let level = payload
        .get("level")
        .and_then(|v| v.as_str())
        .unwrap_or("INFO");
    
    let component = payload
        .get("component")
        .and_then(|v| v.as_str())
        .unwrap_or("admin");
    
    let message = payload
        .get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("message is required".to_string()))?;
    
    let details = payload.get("details").cloned();
    let ip_address = get_client_ip(&headers);

    state
        .monitoring_service
        .log_system_event(level, component, message, details, Some(admin_id), ip_address)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to log system event: {}", e)))?;

    Ok(success_response_with_message(
        serde_json::json!({
            "logged_at": chrono::Utc::now(),
            "level": level,
            "component": component
        }),
        "System event logged successfully",
    ))
}

/// Get monitoring dashboard overview
pub async fn get_monitoring_overview(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (_admin_id, _admin_email) = require_admin(&headers, &state).await?;

    // Get all the overview data in parallel
    let (system_metrics, services_status, db_status, smtp_status) = tokio::try_join!(
        state.monitoring_service.get_system_metrics(),
        state.monitoring_service.get_services_status(),
        state.monitoring_service.get_database_status(),
        state.monitoring_service.get_smtp_status()
    ).map_err(|e| AppError::Internal(format!("Failed to get monitoring overview: {}", e)))?;

    // Get recent error count
    let recent_errors = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM system_logs 
        WHERE log_level IN ('ERROR', 'WARN') 
        AND created_at >= NOW() - INTERVAL '24 hours'
        "#
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);

    let overview = serde_json::json!({
        "system_metrics": system_metrics,
        "services_status": services_status,
        "database_status": db_status,
        "smtp_status": smtp_status,
        "recent_issues": {
            "error_count_24h": recent_errors,
            "last_updated": chrono::Utc::now()
        }
    });

    Ok(success_response(overview))
}