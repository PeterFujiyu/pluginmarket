use anyhow::{anyhow, Result};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::{
    models::{
        ConfigResponse, UpdateConfigRequest, SmtpConfigUpdate, DatabaseConfigUpdate,
        ServerConfigUpdate, StorageConfigUpdate, TestConfigRequest, ConfigTestEmailRequest,
        ConfigHistoryEntry, RollbackConfigRequest,
    },
    utils::config::{Config, SmtpConfig},
    services::smtp::SmtpService,
};

pub struct ConfigService {
    pool: PgPool,
    config: Arc<RwLock<Config>>,
    smtp_service: Arc<SmtpService>,
}

impl ConfigService {
    pub fn new(pool: PgPool, config: Config, smtp_service: Arc<SmtpService>) -> Self {
        Self {
            pool,
            config: Arc::new(RwLock::new(config)),
            smtp_service,
        }
    }

    pub async fn load_persisted_config(&self) -> Result<()> {
        // Try to load configuration from database
        match self.load_config_from_database().await {
            Ok(Some(persisted_config)) => {
                info!("Loading persisted configuration from database");
                // Apply configuration with error handling to prevent startup failure
                if let Err(e) = self.apply_full_config(persisted_config).await {
                    warn!("Failed to apply persisted configuration: {}, using default config", e);
                    // Continue with default configuration to prevent startup failure
                }
            }
            Ok(None) => {
                info!("No persisted configuration found, using default config from file");
            }
            Err(e) => {
                warn!("Failed to load persisted configuration: {}, using default config", e);
            }
        }
        Ok(())
    }

    async fn load_config_from_database(&self) -> Result<Option<serde_json::Value>> {
        let result = sqlx::query!(
            "SELECT value FROM config_store WHERE key = 'current_config'"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| row.value))
    }

    async fn apply_full_config(&self, config_json: serde_json::Value) -> Result<()> {
        let mut config = self.config.write().await;
        
        // Update SMTP configuration
        if let Some(smtp_config) = config_json.get("smtp") {
            if let Ok(smtp_update) = serde_json::from_value::<SmtpConfigUpdate>(smtp_config.clone()) {
                config.smtp.enabled = smtp_update.enabled;
                config.smtp.host = smtp_update.host;
                config.smtp.port = smtp_update.port;
                config.smtp.username = smtp_update.username;
                config.smtp.password = smtp_update.password;
                config.smtp.from_address = smtp_update.from_address;
                config.smtp.from_name = smtp_update.from_name;
                config.smtp.use_tls = smtp_update.use_tls;
                
                // Update SMTP service with error handling
                if let Err(e) = self.smtp_service.update_config(&config.smtp).await {
                    warn!("Failed to update SMTP service configuration: {}", e);
                    // Continue without failing the entire configuration load
                }
            }
        }

        // Update database configuration
        if let Some(db_config) = config_json.get("database") {
            if let Ok(db_update) = serde_json::from_value::<DatabaseConfigUpdate>(db_config.clone()) {
                config.database.max_connections = db_update.max_connections;
                config.database.connect_timeout = db_update.connect_timeout;
            }
        }

        // Update server configuration
        if let Some(server_config) = config_json.get("server") {
            if let Ok(server_update) = serde_json::from_value::<ServerConfigUpdate>(server_config.clone()) {
                config.server.host = server_update.host;
                config.server.port = server_update.port;
                config.jwt.secret = server_update.jwt_secret;
                config.jwt.access_token_expires_in = server_update.jwt_access_token_expires_in;
                config.jwt.refresh_token_expires_in = server_update.jwt_refresh_token_expires_in;
                config.cors.allowed_origins = server_update.cors_origins
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }

        // Update storage configuration
        if let Some(storage_config) = config_json.get("storage") {
            if let Ok(storage_update) = serde_json::from_value::<StorageConfigUpdate>(storage_config.clone()) {
                config.storage.upload_path = storage_update.upload_path;
                config.storage.max_file_size = storage_update.max_file_size;
                config.storage.use_cdn = storage_update.use_cdn;
                if let Some(cdn_url) = storage_update.cdn_base_url {
                    config.storage.cdn_base_url = cdn_url;
                }
            }
        }

        info!("Applied persisted configuration successfully");
        Ok(())
    }

    pub async fn get_current_config(&self) -> Result<ConfigResponse> {
        let config = self.config.read().await;
        Ok(ConfigResponse::from(&*config))
    }

    pub async fn update_config(
        &self,
        admin_id: i32,
        request: UpdateConfigRequest,
        ip_address: Option<std::net::IpAddr>,
    ) -> Result<String> {
        let config_type = request.config_type.as_str();
        
        // Get current config for backup
        let old_config = {
            let config = self.config.read().await;
            self.serialize_config_section(&config, config_type)?
        };

        // Update config based on type
        let version = match config_type {
            "smtp" => {
                let smtp_update: SmtpConfigUpdate = serde_json::from_value(request.config_data)
                    .map_err(|e| anyhow!("Invalid SMTP config: {}", e))?;
                self.update_smtp_config(smtp_update).await?
            }
            "database" => {
                let db_update: DatabaseConfigUpdate = serde_json::from_value(request.config_data)
                    .map_err(|e| anyhow!("Invalid database config: {}", e))?;
                self.update_database_config(db_update).await?
            }
            "server" => {
                let server_update: ServerConfigUpdate = serde_json::from_value(request.config_data)
                    .map_err(|e| anyhow!("Invalid server config: {}", e))?;
                self.update_server_config(server_update).await?
            }
            "storage" => {
                let storage_update: StorageConfigUpdate = serde_json::from_value(request.config_data)
                    .map_err(|e| anyhow!("Invalid storage config: {}", e))?;
                self.update_storage_config(storage_update).await?
            }
            _ => return Err(anyhow!("Unsupported config type: {}", config_type)),
        };

        // Get new config for history
        let new_config = {
            let config = self.config.read().await;
            self.serialize_config_section(&config, config_type)?
        };

        // Save to history
        self.save_config_history(
            config_type,
            Some(old_config),
            new_config,
            admin_id,
            None,
            &version,
            ip_address,
        ).await?;

        // Save config to file/env (in production, this would update environment or config store)
        self.persist_config().await?;

        Ok(version)
    }

    async fn update_smtp_config(&self, update: SmtpConfigUpdate) -> Result<String> {
        let mut config = self.config.write().await;
        config.smtp.enabled = update.enabled;
        config.smtp.host = update.host;
        config.smtp.port = update.port;
        config.smtp.username = update.username;
        config.smtp.password = update.password;
        config.smtp.from_address = update.from_address;
        config.smtp.from_name = update.from_name;
        config.smtp.use_tls = update.use_tls;

        // Update SMTP service configuration with hot validation (for explicit updates)
        if let Err(e) = self.smtp_service.update_config_with_validation(&config.smtp).await {
            warn!("SMTP configuration update failed: {}", e);
            // Allow configuration to be saved even if SMTP validation fails
            // This enables hot updates without blocking the configuration change
        }
        
        Ok(self.generate_version())
    }

    async fn update_database_config(&self, update: DatabaseConfigUpdate) -> Result<String> {
        // Test database connection with new configuration first
        match self.test_database_connection_with_params(update.max_connections, update.connect_timeout).await {
            Ok(test_result) => {
                if test_result["status"] == "error" {
                    warn!("Database configuration test failed: {}", test_result["message"]);
                    // Continue with update but warn user
                }
                info!("Database configuration test passed, applying changes");
            }
            Err(e) => {
                warn!("Database configuration validation failed: {}", e);
                // Continue with update but log warning
            }
        }

        let mut config = self.config.write().await;
        config.database.max_connections = update.max_connections;
        config.database.connect_timeout = update.connect_timeout;
        
        info!("Database configuration updated successfully. Note: Connection pool changes take effect on next restart.");
        Ok(self.generate_version())
    }

    async fn update_server_config(&self, update: ServerConfigUpdate) -> Result<String> {
        let mut config = self.config.write().await;
        config.server.host = update.host;
        config.server.port = update.port;
        config.jwt.secret = update.jwt_secret;
        config.jwt.access_token_expires_in = update.jwt_access_token_expires_in;
        config.jwt.refresh_token_expires_in = update.jwt_refresh_token_expires_in;
        
        // Parse CORS origins
        config.cors.allowed_origins = update.cors_origins
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        warn!("Server configuration updated. Restart required for changes to take effect.");
        Ok(self.generate_version())
    }

    async fn update_storage_config(&self, update: StorageConfigUpdate) -> Result<String> {
        let mut config = self.config.write().await;
        config.storage.upload_path = update.upload_path;
        config.storage.max_file_size = update.max_file_size;
        config.storage.use_cdn = update.use_cdn;
        
        if let Some(cdn_url) = update.cdn_base_url {
            config.storage.cdn_base_url = cdn_url;
        }
        
        Ok(self.generate_version())
    }

    pub async fn test_config(&self, request: TestConfigRequest) -> Result<serde_json::Value> {
        match request.config_type.as_str() {
            "smtp" => self.test_smtp_connection().await,
            "database" => self.test_database_connection().await,
            "storage" => self.test_storage_access().await,
            _ => Err(anyhow!("Unsupported test type: {}", request.config_type)),
        }
    }

    async fn test_smtp_config(&self, test_data: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let test_email = if let Some(data) = test_data {
            serde_json::from_value::<ConfigTestEmailRequest>(data)
                .map_err(|e| anyhow!("Invalid test email request: {}", e))?
        } else {
            // Use default test email
            ConfigTestEmailRequest {
                recipient: "test@example.com".to_string(),
                subject: "SMTP Test".to_string(),
                body: Some("This is a test email from GeekTools configuration system.".to_string()),
            }
        };

        // Test SMTP connection and sending
        match self.smtp_service.send_test_email(&test_email.recipient, &test_email.subject, &test_email.body.unwrap_or_else(|| "Test email".to_string())).await {
            Ok(_) => Ok(json!({
                "status": "success",
                "message": format!("Test email sent successfully to {}", test_email.recipient),
                "tested_at": chrono::Utc::now(),
            })),
            Err(e) => Ok(json!({
                "status": "error",
                "message": format!("SMTP test failed: {}", e),
                "tested_at": chrono::Utc::now(),
            })),
        }
    }

    async fn test_database_config(&self) -> Result<serde_json::Value> {
        // Test database connectivity
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => {
                // Get additional database metrics
                let result = sqlx::query!(
                    "SELECT 
                        (SELECT count(*) FROM pg_stat_activity WHERE state = 'active') as active_connections,
                        (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_connections,
                        current_database() as database_name,
                        version() as version"
                ).fetch_one(&self.pool).await?;

                Ok(json!({
                    "status": "success",
                    "message": "Database connection successful",
                    "details": {
                        "database_name": result.database_name,
                        "version": result.version,
                        "active_connections": result.active_connections,
                        "max_connections": result.max_connections,
                    },
                    "tested_at": chrono::Utc::now(),
                }))
            }
            Err(e) => Ok(json!({
                "status": "error",
                "message": format!("Database test failed: {}", e),
                "tested_at": chrono::Utc::now(),
            })),
        }
    }

    async fn test_storage_config(&self) -> Result<serde_json::Value> {
        let config = self.config.read().await;
        let upload_path = &config.storage.upload_path;

        // Test if upload directory exists and is writable
        let path = std::path::Path::new(upload_path);
        
        if !path.exists() {
            if let Err(e) = tokio::fs::create_dir_all(path).await {
                return Ok(json!({
                    "status": "error",
                    "message": format!("Failed to create upload directory: {}", e),
                    "tested_at": chrono::Utc::now(),
                }));
            }
        }

        // Test write permissions by creating a test file
        let test_file = path.join("test_write_permissions.tmp");
        match tokio::fs::write(&test_file, "test").await {
            Ok(_) => {
                // Clean up test file
                let _ = tokio::fs::remove_file(&test_file).await;
                
                // Get directory size information
                let metadata = tokio::fs::metadata(path).await?;
                
                Ok(json!({
                    "status": "success",
                    "message": "Storage configuration is valid",
                    "details": {
                        "upload_path": upload_path,
                        "max_file_size_mb": config.storage.max_file_size / 1024 / 1024,
                        "use_cdn": config.storage.use_cdn,
                        "cdn_base_url": config.storage.cdn_base_url,
                        "directory_exists": true,
                        "writable": true,
                    },
                    "tested_at": chrono::Utc::now(),
                }))
            }
            Err(e) => Ok(json!({
                "status": "error",
                "message": format!("Upload directory is not writable: {}", e),
                "tested_at": chrono::Utc::now(),
            })),
        }
    }

    pub async fn get_config_history(&self, limit: Option<i64>) -> Result<Vec<ConfigHistoryEntry>> {
        let limit = limit.unwrap_or(50).min(100);
        
        let history = sqlx::query_as!(
            ConfigHistoryEntry,
            r#"
            SELECT 
                id,
                config_type,
                old_config,
                new_config,
                changed_by_id,
                changed_at,
                change_reason,
                version,
                ip_address
            FROM config_history 
            ORDER BY changed_at DESC 
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    pub async fn rollback_config(
        &self,
        admin_id: i32,
        request: RollbackConfigRequest,
        ip_address: Option<std::net::IpAddr>,
    ) -> Result<String> {
        // Get the history entry
        let history = sqlx::query_as!(
            ConfigHistoryEntry,
            "SELECT * FROM config_history WHERE id = $1",
            request.history_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Configuration history entry not found"))?;

        // Get current config for backup
        let config_type = &history.config_type;
        let current_config = {
            let config = self.config.read().await;
            self.serialize_config_section(&config, config_type)?
        };

        // Apply the rollback
        let rollback_config = history.old_config
            .ok_or_else(|| anyhow!("No previous configuration to rollback to"))?;

        self.apply_config_section(config_type, rollback_config.clone()).await?;

        // Save rollback to history
        let version = self.generate_version();
        self.save_config_history(
            config_type,
            Some(current_config),
            rollback_config,
            admin_id,
            Some(format!("Rollback: {}", request.reason)),
            &version,
            ip_address,
        ).await?;

        Ok(version)
    }

    pub async fn create_config_snapshot(
        &self,
        admin_id: i32,
        description: Option<String>,
        ip_address: Option<std::net::IpAddr>,
    ) -> Result<String> {
        let config = self.config.read().await;
        let full_config = json!({
            "smtp": self.serialize_config_section(&config, "smtp")?,
            "database": self.serialize_config_section(&config, "database")?,
            "server": self.serialize_config_section(&config, "server")?,
            "storage": self.serialize_config_section(&config, "storage")?,
        });

        let version = self.generate_version();
        self.save_config_history(
            "snapshot",
            None,
            full_config,
            admin_id,
            description,
            &version,
            ip_address,
        ).await?;

        Ok(version)
    }

    // Helper methods
    fn serialize_config_section(&self, config: &Config, section: &str) -> Result<serde_json::Value> {
        match section {
            "smtp" => Ok(json!({
                "enabled": config.smtp.enabled,
                "host": config.smtp.host,
                "port": config.smtp.port,
                "username": config.smtp.username,
                "password": config.smtp.password,
                "from_address": config.smtp.from_address,
                "from_name": config.smtp.from_name,
                "use_tls": config.smtp.use_tls,
            })),
            "database" => Ok(json!({
                "max_connections": config.database.max_connections,
                "connect_timeout": config.database.connect_timeout,
            })),
            "server" => Ok(json!({
                "host": config.server.host,
                "port": config.server.port,
                "jwt_secret": config.jwt.secret,
                "jwt_access_token_expires_in": config.jwt.access_token_expires_in,
                "jwt_refresh_token_expires_in": config.jwt.refresh_token_expires_in,
                "cors_origins": config.cors.allowed_origins.join("\n"),
            })),
            "storage" => Ok(json!({
                "upload_path": config.storage.upload_path,
                "max_file_size": config.storage.max_file_size,
                "use_cdn": config.storage.use_cdn,
                "cdn_base_url": config.storage.cdn_base_url,
            })),
            _ => Err(anyhow!("Unknown config section: {}", section)),
        }
    }

    async fn apply_config_section(&self, section: &str, config_data: serde_json::Value) -> Result<()> {
        match section {
            "smtp" => {
                let smtp_update: SmtpConfigUpdate = serde_json::from_value(config_data)?;
                self.update_smtp_config(smtp_update).await?;
            }
            "database" => {
                let db_update: DatabaseConfigUpdate = serde_json::from_value(config_data)?;
                self.update_database_config(db_update).await?;
            }
            "server" => {
                let server_update: ServerConfigUpdate = serde_json::from_value(config_data)?;
                self.update_server_config(server_update).await?;
            }
            "storage" => {
                let storage_update: StorageConfigUpdate = serde_json::from_value(config_data)?;
                self.update_storage_config(storage_update).await?;
            }
            _ => return Err(anyhow!("Unknown config section: {}", section)),
        }
        Ok(())
    }

    async fn save_config_history(
        &self,
        config_type: &str,
        old_config: Option<serde_json::Value>,
        new_config: serde_json::Value,
        admin_id: i32,
        change_reason: Option<String>,
        version: &str,
        ip_address: Option<std::net::IpAddr>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO config_history (
                config_type, old_config, new_config, changed_by_id, 
                change_reason, version, ip_address
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            config_type,
            old_config,
            new_config,
            admin_id,
            change_reason,
            version,
            ip_address.map(|ip| ip.to_string())
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn persist_config(&self) -> Result<()> {
        // Save the current configuration to the config_store table for persistence across restarts
        let config = self.config.read().await;
        
        // Serialize the entire configuration to JSON
        let full_config = json!({
            "smtp": {
                "enabled": config.smtp.enabled,
                "host": config.smtp.host,
                "port": config.smtp.port,
                "username": config.smtp.username,
                "password": config.smtp.password,
                "from_address": config.smtp.from_address,
                "from_name": config.smtp.from_name,
                "use_tls": config.smtp.use_tls,
            },
            "database": {
                "max_connections": config.database.max_connections,
                "connect_timeout": config.database.connect_timeout,
            },
            "server": {
                "host": config.server.host,
                "port": config.server.port,
                "jwt_secret": config.jwt.secret,
                "jwt_access_token_expires_in": config.jwt.access_token_expires_in,
                "jwt_refresh_token_expires_in": config.jwt.refresh_token_expires_in,
                "cors_origins": config.cors.allowed_origins.clone(),
            },
            "storage": {
                "upload_path": config.storage.upload_path,
                "max_file_size": config.storage.max_file_size,
                "use_cdn": config.storage.use_cdn,
                "cdn_base_url": config.storage.cdn_base_url,
            }
        });

        // Insert or update configuration in the config_store table
        sqlx::query!(
            r#"
            INSERT INTO config_store (key, value, updated_at) 
            VALUES ('current_config', $1, NOW())
            ON CONFLICT (key) 
            DO UPDATE SET value = $1, updated_at = NOW()
            "#,
            full_config
        )
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to persist configuration: {}", e))?;

        info!("Configuration persisted to database successfully");
        Ok(())
    }

    fn generate_version(&self) -> String {
        let now = chrono::Utc::now();
        format!("v{}", now.format("%Y%m%d_%H%M%S"))
    }

    pub async fn get_config_clone(&self) -> Config {
        self.config.read().await.clone()
    }


    /// Test database connection with current configuration
    async fn test_database_connection(&self) -> Result<serde_json::Value> {
        let start_time = std::time::Instant::now();
        
        // Test basic database connectivity
        match sqlx::query("SELECT 1 as test_value").execute(&self.pool).await {
            Ok(_) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                
                // Get additional database info
                let db_info = sqlx::query!(
                    r#"
                    SELECT 
                        current_database() as db_name,
                        version() as version,
                        (SELECT count(*) FROM pg_stat_activity WHERE state = 'active') as active_connections,
                        (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_connections
                    "#
                ).fetch_one(&self.pool).await?;

                Ok(serde_json::json!({
                    "status": "success",
                    "message": "Database connection successful",
                    "response_time_ms": response_time_ms,
                    "details": {
                        "database_name": db_info.db_name,
                        "version": db_info.version,
                        "active_connections": db_info.active_connections,
                        "max_connections": db_info.max_connections
                    },
                    "tested_at": chrono::Utc::now()
                }))
            }
            Err(e) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                Ok(serde_json::json!({
                    "status": "error", 
                    "message": format!("Database connection failed: {}", e),
                    "response_time_ms": response_time_ms,
                    "error": e.to_string(),
                    "tested_at": chrono::Utc::now()
                }))
            }
        }
    }

    /// Test SMTP connection with current configuration
    async fn test_smtp_connection(&self) -> Result<serde_json::Value> {
        let config = self.config.read().await;
        let start_time = std::time::Instant::now();

        if !config.smtp.enabled {
            return Ok(serde_json::json!({
                "status": "warning",
                "message": "SMTP is not enabled in configuration",
                "details": {
                    "configured": false,
                    "enabled": false
                },
                "tested_at": chrono::Utc::now()
            }));
        }

        if config.smtp.username.is_empty() || config.smtp.password.is_empty() {
            return Ok(serde_json::json!({
                "status": "error",
                "message": "SMTP credentials are not configured",
                "details": {
                    "configured": false,
                    "username_set": !config.smtp.username.is_empty(),
                    "password_set": !config.smtp.password.is_empty()
                },
                "tested_at": chrono::Utc::now()
            }));
        }

        // Test SMTP connection by attempting to connect
        match self.test_smtp_connection_internal(&config.smtp).await {
            Ok(_) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                Ok(serde_json::json!({
                    "status": "success",
                    "message": "SMTP connection successful",
                    "response_time_ms": response_time_ms,
                    "details": {
                        "host": config.smtp.host,
                        "port": config.smtp.port,
                        "username": config.smtp.username,
                        "use_tls": config.smtp.use_tls
                    },
                    "tested_at": chrono::Utc::now()
                }))
            }
            Err(e) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                Ok(serde_json::json!({
                    "status": "error",
                    "message": format!("SMTP connection failed: {}", e),
                    "response_time_ms": response_time_ms,
                    "error": e.to_string(),
                    "details": {
                        "host": config.smtp.host,
                        "port": config.smtp.port,
                        "username": config.smtp.username
                    },
                    "tested_at": chrono::Utc::now()
                }))
            }
        }
    }

    /// Internal SMTP connection test (without sending email)
    async fn test_smtp_connection_internal(&self, smtp_config: &SmtpConfig) -> Result<()> {
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::SmtpTransport;

        let creds = Credentials::new(smtp_config.username.clone(), smtp_config.password.clone());
        
        let transport = if smtp_config.use_tls {
            if smtp_config.port == 465 {
                SmtpTransport::relay(&smtp_config.host)?
                    .port(smtp_config.port)
                    .credentials(creds)
                    .timeout(Some(std::time::Duration::from_secs(3)))
                    .tls(lettre::transport::smtp::client::Tls::Wrapper(
                        lettre::transport::smtp::client::TlsParameters::new(smtp_config.host.clone())?
                    ))
                    .build()
            } else {
                SmtpTransport::starttls_relay(&smtp_config.host)?
                    .port(smtp_config.port)
                    .credentials(creds)
                    .timeout(Some(std::time::Duration::from_secs(3)))
                    .build()
            }
        } else {
            SmtpTransport::relay(&smtp_config.host)?
                .port(smtp_config.port)
                .credentials(creds)
                .timeout(Some(std::time::Duration::from_secs(10)))
                .build()
        };

        // Test connection by connecting and immediately disconnecting
        transport.test_connection()?;
        Ok(())
    }

    /// Test storage access
    async fn test_storage_access(&self) -> Result<serde_json::Value> {
        let config = self.config.read().await;
        let start_time = std::time::Instant::now();
        
        // Test if upload directory exists and is writable
        let upload_path = std::path::Path::new(&config.storage.upload_path);
        
        match tokio::fs::metadata(&upload_path).await {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    return Ok(serde_json::json!({
                        "status": "error",
                        "message": "Upload path exists but is not a directory",
                        "details": {
                            "path": config.storage.upload_path,
                            "is_file": metadata.is_file()
                        },
                        "tested_at": chrono::Utc::now()
                    }));
                }

                // Test write permission by creating a temporary test file
                let test_file_path = upload_path.join(".storage_test");
                match tokio::fs::write(&test_file_path, "test").await {
                    Ok(_) => {
                        // Clean up test file
                        let _ = tokio::fs::remove_file(&test_file_path).await;
                        
                        let response_time_ms = start_time.elapsed().as_millis() as u64;
                        Ok(serde_json::json!({
                            "status": "success",
                            "message": "Storage access test successful",
                            "response_time_ms": response_time_ms,
                            "details": {
                                "upload_path": config.storage.upload_path,
                                "max_file_size_mb": config.storage.max_file_size / 1024 / 1024,
                                "use_cdn": config.storage.use_cdn,
                                "writable": true
                            },
                            "tested_at": chrono::Utc::now()
                        }))
                    }
                    Err(e) => {
                        Ok(serde_json::json!({
                            "status": "error", 
                            "message": format!("Storage directory is not writable: {}", e),
                            "details": {
                                "upload_path": config.storage.upload_path,
                                "error": e.to_string(),
                                "writable": false
                            },
                            "tested_at": chrono::Utc::now()
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(serde_json::json!({
                    "status": "error",
                    "message": format!("Upload directory does not exist: {}", e),
                    "details": {
                        "upload_path": config.storage.upload_path,
                        "error": e.to_string(),
                        "exists": false
                    },
                    "tested_at": chrono::Utc::now()
                }))
            }
        }
    }

    /// Test database connection with specific parameters (for hot validation)
    async fn test_database_connection_with_params(&self, max_connections: u32, connect_timeout: u64) -> Result<serde_json::Value> {
        let start_time = std::time::Instant::now();
        
        // Test basic database connectivity with current pool
        match sqlx::query("SELECT 1 as test_value").execute(&self.pool).await {
            Ok(_) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                
                // Get current database info
                let db_info = sqlx::query!(
                    r#"
                    SELECT 
                        current_database() as db_name,
                        version() as version,
                        (SELECT count(*) FROM pg_stat_activity WHERE state = 'active') as active_connections,
                        (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as db_max_connections
                    "#
                ).fetch_one(&self.pool).await?;

                // Validate new parameters against database limits
                let db_max_conn = db_info.db_max_connections.unwrap_or(100) as u32;
                if max_connections > db_max_conn {
                    return Ok(serde_json::json!({
                        "status": "warning",
                        "message": format!("Requested max_connections ({}) exceeds database limit ({})", max_connections, db_max_conn),
                        "response_time_ms": response_time_ms,
                        "details": {
                            "database_name": db_info.db_name,
                            "version": db_info.version,
                            "current_active_connections": db_info.active_connections,
                            "database_max_connections": db_max_conn,
                            "requested_max_connections": max_connections,
                            "requested_connect_timeout": connect_timeout
                        },
                        "tested_at": chrono::Utc::now()
                    }));
                }

                Ok(serde_json::json!({
                    "status": "success",
                    "message": "Database configuration validation successful",
                    "response_time_ms": response_time_ms,
                    "details": {
                        "database_name": db_info.db_name,
                        "version": db_info.version,
                        "current_active_connections": db_info.active_connections,
                        "database_max_connections": db_max_conn,
                        "requested_max_connections": max_connections,
                        "requested_connect_timeout": connect_timeout
                    },
                    "tested_at": chrono::Utc::now()
                }))
            }
            Err(e) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                Ok(serde_json::json!({
                    "status": "error", 
                    "message": format!("Database connection test failed: {}", e),
                    "response_time_ms": response_time_ms,
                    "error": e.to_string(),
                    "details": {
                        "requested_max_connections": max_connections,
                        "requested_connect_timeout": connect_timeout
                    },
                    "tested_at": chrono::Utc::now()
                }))
            }
        }
    }
}