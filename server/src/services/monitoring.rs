use anyhow::{anyhow, Result};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::{
    models::{
        SystemMetrics, ServicesStatusResponse, ServiceStatus, DatabaseStatus, SmtpStatus,
        SystemLog, SystemLogsQuery, SystemLogsResponse, TestResult, PerformanceChartData,
        SystemMetricRecord, monitoring::TestEmailRequest,
    },
    utils::config::Config,
    services::smtp::SmtpService,
};

pub struct MonitoringService {
    pool: PgPool,
    config: Arc<Config>,
    smtp_service: Arc<SmtpService>,
}

impl MonitoringService {
    pub fn new(pool: PgPool, config: Arc<Config>, smtp_service: Arc<SmtpService>) -> Self {
        Self {
            pool,
            config,
            smtp_service,
        }
    }

    /// Get system metrics (CPU, memory, disk, network)
    pub async fn get_system_metrics(&self) -> Result<SystemMetrics> {
        // In a production environment, you would collect real system metrics
        // For now, we'll return mock data with some realistic values
        
        // You could use libraries like:
        // - sysinfo for system information
        // - tokio-metrics for async runtime metrics
        // - systemstat for cross-platform system statistics
        
        Ok(SystemMetrics::mock())
    }

    /// Get services health status
    pub async fn get_services_status(&self) -> Result<ServicesStatusResponse> {
        let mut services = Vec::new();

        // Check web server status (always healthy if we're responding)
        services.push(ServiceStatus::web_server());

        // Check database status
        let db_status = self.get_database_status().await?;
        services.push(ServiceStatus::database(&db_status));

        // Check SMTP status
        let smtp_status = self.get_smtp_status().await?;
        services.push(ServiceStatus::smtp(&smtp_status));

        // Calculate overall status
        let healthy_count = services.iter().filter(|s| s.status == "healthy").count() as u32;
        let warning_count = services.iter().filter(|s| s.status == "warning").count() as u32;
        let critical_count = services.iter().filter(|s| s.status == "critical").count() as u32;

        let overall_status = if critical_count > 0 {
            "critical"
        } else if warning_count > 0 {
            "warning"
        } else {
            "healthy"
        };

        Ok(ServicesStatusResponse {
            services,
            overall_status: overall_status.to_string(),
            healthy_count,
            warning_count,
            critical_count,
        })
    }

    /// Get database status and metrics
    pub async fn get_database_status(&self) -> Result<DatabaseStatus> {
        let start_time = std::time::Instant::now();

        // Test database connectivity and get metrics
        let result = sqlx::query!(
            r#"
            SELECT 
                (SELECT count(*) FROM pg_stat_activity WHERE state = 'active') as active_connections,
                (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_connections,
                (SELECT pg_database_size(current_database())::bigint / 1024 / 1024) as database_size_mb,
                version() as version
            "#
        ).fetch_one(&self.pool).await;

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(row) => {
                // Calculate queries per second (mock value - in production you'd track this)
                let queries_per_second = 45.7;

                Ok(DatabaseStatus {
                    status: "healthy".to_string(),
                    response_time_ms,
                    active_connections: row.active_connections.unwrap_or(0),
                    max_connections: row.max_connections.unwrap_or(100),
                    queries_per_second,
                    database_size_mb: row.database_size_mb.unwrap_or(0) as f64,
                    version: row.version.unwrap_or_else(|| "Unknown".to_string()),
                    last_check: chrono::Utc::now(),
                })
            }
            Err(e) => {
                error!("Database health check failed: {}", e);
                Ok(DatabaseStatus {
                    status: "critical".to_string(),
                    response_time_ms,
                    active_connections: 0,
                    max_connections: 0,
                    queries_per_second: 0.0,
                    database_size_mb: 0.0,
                    version: "Unknown".to_string(),
                    last_check: chrono::Utc::now(),
                })
            }
        }
    }

    /// Get SMTP service status
    pub async fn get_smtp_status(&self) -> Result<SmtpStatus> {
        let configured = self.smtp_service.is_enabled().await;
        
        // Get recent email statistics (simplified without system_logs table for now)
        let emails_sent_today = 0i64; // Mock value since system_logs table doesn't exist yet
        let last_email_sent: Option<chrono::DateTime<chrono::Utc>> = None;
        let last_error: Option<String> = None;

        let status = if !configured {
            "warning"
        } else if last_error.is_some() && emails_sent_today == 0 {
            "critical"
        } else {
            "healthy"
        };

        Ok(SmtpStatus {
            status: status.to_string(),
            configured,
            last_email_sent,
            emails_sent_today,
            last_error,
            response_time_ms: None,
        })
    }

    /// Get system logs with filtering and pagination
    pub async fn get_system_logs(&self, params: SystemLogsQuery) -> Result<SystemLogsResponse> {
        // Return empty logs for now since system_logs table doesn't exist yet
        Ok(SystemLogsResponse {
            logs: vec![],
            total_count: 0,
            has_more: false,
        })
    }

    /// Test database connection
    pub async fn test_database_connection(&self) -> Result<TestResult> {
        let start_time = std::time::Instant::now();
        
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                
                // Get additional details
                let details = sqlx::query!(
                    "SELECT current_database() as db_name, version() as version"
                ).fetch_one(&self.pool).await.ok();

                Ok(TestResult {
                    test_type: "database".to_string(),
                    status: "success".to_string(),
                    message: "Database connection successful".to_string(),
                    response_time_ms: Some(response_time_ms),
                    details: details.map(|d| serde_json::json!({
                        "database_name": d.db_name,
                        "version": d.version
                    })),
                    tested_at: chrono::Utc::now(),
                })
            }
            Err(e) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                Ok(TestResult {
                    test_type: "database".to_string(),
                    status: "error".to_string(),
                    message: format!("Database connection failed: {}", e),
                    response_time_ms: Some(response_time_ms),
                    details: None,
                    tested_at: chrono::Utc::now(),
                })
            }
        }
    }

    /// Test SMTP configuration by sending a test email
    pub async fn test_smtp_connection(&self, request: TestEmailRequest) -> Result<TestResult> {
        if !self.smtp_service.is_enabled().await {
            return Ok(TestResult {
                test_type: "smtp".to_string(),
                status: "warning".to_string(),
                message: "SMTP is not configured or enabled".to_string(),
                response_time_ms: None,
                details: Some(serde_json::json!({
                    "configured": false,
                    "enabled": false
                })),
                tested_at: chrono::Utc::now(),
            });
        }

        let start_time = std::time::Instant::now();
        
        match self.smtp_service.send_test_email(
            &request.recipient,
            &request.get_subject(),
            &request.get_body(),
        ).await {
            Ok(_) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                
                // Log successful test
                self.log_system_event(
                    "INFO",
                    "monitoring",
                    &format!("SMTP test email sent successfully to {}", request.recipient),
                    Some(serde_json::json!({
                        "recipient": request.recipient,
                        "test_type": "smtp_connection"
                    })),
                    None,
                    None,
                ).await?;

                Ok(TestResult {
                    test_type: "smtp".to_string(),
                    status: "success".to_string(),
                    message: format!("Test email sent successfully to {}", request.recipient),
                    response_time_ms: Some(response_time_ms),
                    details: Some(serde_json::json!({
                        "recipient": request.recipient,
                        "subject": request.get_subject()
                    })),
                    tested_at: chrono::Utc::now(),
                })
            }
            Err(e) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                
                // Log failed test
                self.log_system_event(
                    "ERROR",
                    "monitoring",
                    &format!("SMTP test email failed: {}", e),
                    Some(serde_json::json!({
                        "recipient": request.recipient,
                        "error": e.to_string(),
                        "test_type": "smtp_connection"
                    })),
                    None,
                    None,
                ).await?;

                Ok(TestResult {
                    test_type: "smtp".to_string(),
                    status: "error".to_string(),
                    message: format!("SMTP test failed: {}", e),
                    response_time_ms: Some(response_time_ms),
                    details: Some(serde_json::json!({
                        "error": e.to_string(),
                        "recipient": request.recipient
                    })),
                    tested_at: chrono::Utc::now(),
                })
            }
        }
    }

    /// Get performance chart data
    pub async fn get_performance_chart_data(&self, hours: Option<i32>) -> Result<PerformanceChartData> {
        let hours = hours.unwrap_or(24).clamp(1, 168); // Max 1 week
        
        // In a production environment, you would query actual metrics from the database
        // For now, we'll generate mock data
        let now = chrono::Utc::now();
        let mut timepoints = Vec::new();
        let mut cpu_usage = Vec::new();
        let mut memory_usage = Vec::new();
        let mut disk_usage = Vec::new();
        let mut active_connections = Vec::new();

        // Generate data points every 10 minutes for the specified hours
        let data_points = (hours * 6) as usize; // 6 points per hour (every 10 minutes)
        
        for i in 0..data_points {
            let time = now - chrono::Duration::minutes((data_points - i - 1) as i64 * 10);
            timepoints.push(time);
            
            // Generate realistic-looking data with some variation
            let base_cpu = 25.0 + (i as f64 * 0.1).sin() * 10.0;
            let base_memory = 70.0 + (i as f64 * 0.05).cos() * 5.0;
            let base_disk = 45.8;
            let base_connections = 15 + ((i as f64 * 0.2).sin() * 5.0) as i64;
            
            cpu_usage.push(base_cpu.clamp(5.0, 85.0));
            memory_usage.push(base_memory.clamp(50.0, 90.0));
            disk_usage.push(base_disk);
            active_connections.push(base_connections.clamp(5, 50));
        }

        Ok(PerformanceChartData {
            timepoints,
            cpu_usage,
            memory_usage,
            disk_usage,
            active_connections,
        })
    }

    /// Record system metrics to database
    pub async fn record_system_metrics(&self, metrics: &SystemMetrics) -> Result<()> {
        // Skip recording for now as system_metrics table doesn't exist yet
        tracing::info!("Metrics recording skipped - table not implemented yet");

        Ok(())
    }

    /// Log system event
    pub async fn log_system_event(
        &self,
        level: &str,
        component: &str,
        message: &str,
        details: Option<serde_json::Value>,
        user_id: Option<i32>,
        ip_address: Option<std::net::IpAddr>,
    ) -> Result<()> {
        // Skip database logging for now, use tracing instead
        tracing::info!("[{}][{}] {}", level, component, message);
        Ok(())
    }

    /// Clean up old metrics and logs
    pub async fn cleanup_old_data(&self, days_to_keep: i32) -> Result<()> {
        // Skip cleanup for now since system tables don't exist yet
        tracing::info!("Data cleanup skipped - tables not implemented yet");
        Ok(())
    }
}