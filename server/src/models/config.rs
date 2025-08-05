use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub smtp: SmtpConfigResponse,
    pub database: DatabaseConfigResponse,
    pub server: ServerConfigResponse,
    pub storage: StorageConfigResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfigResponse {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String, // Will be masked in actual response
    pub from_address: String,
    pub from_name: String,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfigResponse {
    pub max_connections: u32,
    pub connect_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfigResponse {
    pub host: String,
    pub port: u16,
    pub jwt_access_token_expires_in: i64,
    pub jwt_refresh_token_expires_in: i64,
    pub cors_origins: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfigResponse {
    pub upload_path: String,
    pub max_file_size: u64,
    pub use_cdn: bool,
    pub cdn_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateConfigRequest {
    pub config_type: String,
    pub config_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SmtpConfigUpdate {
    pub enabled: bool,
    #[validate(length(min = 1, max = 255, message = "SMTP host must be between 1 and 255 characters"))]
    pub host: String,
    #[validate(range(min = 1, max = 65535, message = "Port must be between 1 and 65535"))]
    pub port: u16,
    #[validate(email(message = "Username must be a valid email address"))]
    pub username: String,
    #[validate(length(min = 1, max = 255, message = "Password is required"))]
    pub password: String,
    #[validate(email(message = "From address must be a valid email address"))]
    pub from_address: String,
    #[validate(length(min = 1, max = 255, message = "From name is required"))]
    pub from_name: String,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DatabaseConfigUpdate {
    #[validate(range(min = 1, max = 100, message = "Max connections must be between 1 and 100"))]
    pub max_connections: u32,
    #[validate(range(min = 5, max = 300, message = "Connect timeout must be between 5 and 300 seconds"))]
    pub connect_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfigUpdate {
    #[validate(length(min = 1, max = 255, message = "Host is required"))]
    pub host: String,
    #[validate(range(min = 1024, max = 65535, message = "Port must be between 1024 and 65535"))]
    pub port: u16,
    #[validate(length(min = 32, message = "JWT secret must be at least 32 characters"))]
    pub jwt_secret: String,
    #[validate(range(min = 300, max = 86400, message = "Access token expiry must be between 300 and 86400 seconds"))]
    pub jwt_access_token_expires_in: i64,
    #[validate(range(min = 86400, max = 2592000, message = "Refresh token expiry must be between 86400 and 2592000 seconds"))]
    pub jwt_refresh_token_expires_in: i64,
    #[validate(length(min = 1, message = "CORS origins is required"))]
    pub cors_origins: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct StorageConfigUpdate {
    #[validate(length(min = 1, max = 255, message = "Upload path is required"))]
    pub upload_path: String,
    #[validate(range(min = 1048576, max = 1073741824, message = "Max file size must be between 1MB and 1GB"))]
    pub max_file_size: u64,
    pub use_cdn: bool,
    #[validate(url(message = "CDN base URL must be a valid URL"))]
    pub cdn_base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TestConfigRequest {
    #[validate(length(min = 1, message = "Config type is required"))]
    pub config_type: String, // "database", "smtp", "storage"
    pub test_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ConfigTestEmailRequest {
    #[validate(email(message = "Recipient must be a valid email address"))]
    pub recipient: String,
    #[validate(length(min = 1, max = 255, message = "Subject is required"))]
    pub subject: String,
    #[validate(length(min = 1, max = 10000, message = "Body is required"))]
    pub body: Option<String>,
}

impl ConfigTestEmailRequest {
    pub fn get_subject(&self) -> String {
        self.subject.clone()
    }

    pub fn get_body(&self) -> String {
        self.body.as_ref().unwrap_or(&format!(
            "This is a test email sent from GeekTools Plugin Marketplace configuration system.\n\nSent at: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )).clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHistoryEntry {
    pub id: i32,
    pub config_type: String,
    pub old_config: Option<serde_json::Value>,
    pub new_config: serde_json::Value,
    pub changed_by_id: i32,
    pub changed_at: chrono::DateTime<chrono::Utc>,
    pub change_reason: Option<String>,
    pub version: String,
    pub ip_address: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RollbackConfigRequest {
    #[validate(range(min = 1, message = "History ID is required"))]
    pub history_id: i32,
    #[validate(length(min = 1, max = 255, message = "Reason is required"))]
    pub reason: String,
}

impl From<&crate::utils::config::Config> for ConfigResponse {
    fn from(config: &crate::utils::config::Config) -> Self {
        Self {
            smtp: SmtpConfigResponse {
                enabled: config.smtp.enabled,
                host: config.smtp.host.clone(),
                port: config.smtp.port,
                username: config.smtp.username.clone(),
                password: if config.smtp.password.is_empty() { 
                    String::new() 
                } else { 
                    "***".to_string() 
                },
                from_address: config.smtp.from_address.clone(),
                from_name: config.smtp.from_name.clone(),
                use_tls: config.smtp.use_tls,
            },
            database: DatabaseConfigResponse {
                max_connections: config.database.max_connections,
                connect_timeout: config.database.connect_timeout,
            },
            server: ServerConfigResponse {
                host: config.server.host.clone(),
                port: config.server.port,
                jwt_access_token_expires_in: config.jwt.access_token_expires_in,
                jwt_refresh_token_expires_in: config.jwt.refresh_token_expires_in,
                cors_origins: config.cors.allowed_origins.join("\n"),
            },
            storage: StorageConfigResponse {
                upload_path: config.storage.upload_path.clone(),
                max_file_size: config.storage.max_file_size,
                use_cdn: config.storage.use_cdn,
                cdn_base_url: config.storage.cdn_base_url.clone(),
            },
        }
    }
}