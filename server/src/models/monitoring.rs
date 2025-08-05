use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
    pub uptime_seconds: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f64,
    pub cores: u32,
    pub load_average: Vec<f64>, // 1min, 5min, 15min
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_gb: f64,
    pub used_gb: f64,
    pub usage_percent: f64,
    pub available_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub total_gb: f64,
    pub used_gb: f64,
    pub usage_percent: f64,
    pub free_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_recv: u64,
    pub packets_sent: u64,
    pub packets_recv: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String, // healthy, warning, critical
    pub response_time_ms: Option<u64>,
    pub uptime_percent: f64,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesStatusResponse {
    pub services: Vec<ServiceStatus>,
    pub overall_status: String,
    pub healthy_count: u32,
    pub warning_count: u32,
    pub critical_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStatus {
    pub status: String,
    pub response_time_ms: u64,
    pub active_connections: i64,
    pub max_connections: i32,
    pub queries_per_second: f64,
    pub database_size_mb: f64,
    pub version: String,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpStatus {
    pub status: String,
    pub configured: bool,
    pub last_email_sent: Option<chrono::DateTime<chrono::Utc>>,
    pub emails_sent_today: i64,
    pub last_error: Option<String>,
    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SystemLog {
    pub id: i32,
    pub log_level: String,
    pub component: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub user_id: Option<i32>,
    pub ip_address: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SystemLogsQuery {
    #[validate(range(min = 1, max = 1000, message = "Limit must be between 1 and 1000"))]
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub level: Option<String>,
    pub component: Option<String>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLogsResponse {
    pub logs: Vec<SystemLog>,
    pub total_count: i64,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TestEmailRequest {
    #[validate(email(message = "Valid email address is required"))]
    pub recipient: String,
    #[validate(length(min = 1, max = 255, message = "Subject is required"))]
    pub subject: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_type: String,
    pub status: String, // success, warning, error
    pub message: String,
    pub response_time_ms: Option<u64>,
    pub details: Option<serde_json::Value>,
    pub tested_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SystemMetricRecord {
    pub id: i32,
    pub metric_type: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub unit: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceChartData {
    pub timepoints: Vec<chrono::DateTime<chrono::Utc>>,
    pub cpu_usage: Vec<f64>,
    pub memory_usage: Vec<f64>,
    pub disk_usage: Vec<f64>,
    pub active_connections: Vec<i64>,
}

impl SystemLogsQuery {
    pub fn get_limit(&self) -> i32 {
        self.limit.unwrap_or(50).clamp(1, 1000)
    }

    pub fn get_offset(&self) -> i32 {
        self.offset.unwrap_or(0).max(0)
    }

    pub fn build_where_clause(&self) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let mut params = Vec::new();
        let mut param_count = 1;

        if let Some(level) = &self.level {
            conditions.push(format!("log_level = ${}", param_count));
            params.push(level.clone());
            param_count += 1;
        }

        if let Some(component) = &self.component {
            conditions.push(format!("component = ${}", param_count));
            params.push(component.clone());
            param_count += 1;
        }

        if let Some(since) = &self.since {
            conditions.push(format!("created_at >= ${}", param_count));
            params.push(since.to_rfc3339());
            param_count += 1;
        }

        if let Some(until) = &self.until {
            conditions.push(format!("created_at <= ${}", param_count));
            params.push(until.to_rfc3339());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        (where_clause, params)
    }
}

impl TestEmailRequest {
    pub fn get_subject(&self) -> String {
        self.subject
            .as_ref()
            .unwrap_or(&"GeekTools Test Email".to_string())
            .clone()
    }

    pub fn get_body(&self) -> String {
        self.body.as_ref().unwrap_or(&format!(
            "This is a test email sent from GeekTools Plugin Marketplace monitoring system.\n\nSent at: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )).clone()
    }
}

impl SystemMetrics {
    pub fn mock() -> Self {
        let now = chrono::Utc::now();
        Self {
            cpu: CpuMetrics {
                usage_percent: 25.3,
                cores: 4,
                load_average: vec![0.65, 0.72, 0.80],
            },
            memory: MemoryMetrics {
                total_gb: 16.0,
                used_gb: 11.2,
                usage_percent: 70.0,
                available_gb: 4.8,
            },
            disk: DiskMetrics {
                total_gb: 500.0,
                used_gb: 229.0,
                usage_percent: 45.8,
                free_gb: 271.0,
            },
            network: NetworkMetrics {
                bytes_sent: 1048576000,
                bytes_recv: 2097152000,
                packets_sent: 1000000,
                packets_recv: 1500000,
            },
            uptime_seconds: 2847392, // ~33 days
            timestamp: now,
        }
    }
}

impl ServiceStatus {
    pub fn web_server() -> Self {
        Self {
            name: "web_server".to_string(),
            status: "healthy".to_string(),
            response_time_ms: Some(12),
            uptime_percent: 99.9,
            last_check: chrono::Utc::now(),
            details: serde_json::json!({
                "port": 3000,
                "active_connections": 47
            }),
        }
    }

    pub fn database(db_status: &DatabaseStatus) -> Self {
        let status = if db_status.response_time_ms < 100 && db_status.active_connections < db_status.max_connections as i64 * 8 / 10 {
            "healthy"
        } else if db_status.response_time_ms < 1000 {
            "warning"
        } else {
            "critical"
        };

        Self {
            name: "database".to_string(),
            status: status.to_string(),
            response_time_ms: Some(db_status.response_time_ms),
            uptime_percent: 99.8,
            last_check: db_status.last_check,
            details: serde_json::json!({
                "active_connections": db_status.active_connections,
                "max_connections": db_status.max_connections,
                "queries_per_second": db_status.queries_per_second,
                "database_size_mb": db_status.database_size_mb
            }),
        }
    }

    pub fn smtp(smtp_status: &SmtpStatus) -> Self {
        Self {
            name: "smtp".to_string(),
            status: smtp_status.status.clone(),
            response_time_ms: smtp_status.response_time_ms,
            uptime_percent: if smtp_status.configured { 99.5 } else { 0.0 },
            last_check: chrono::Utc::now(),
            details: serde_json::json!({
                "configured": smtp_status.configured,
                "last_email_sent": smtp_status.last_email_sent,
                "emails_sent_today": smtp_status.emails_sent_today,
                "last_error": smtp_status.last_error
            }),
        }
    }
}