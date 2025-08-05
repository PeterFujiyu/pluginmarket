pub mod auth;
pub mod plugin;
pub mod storage;
pub mod admin;
pub mod smtp;
pub mod config;
pub mod backup;
pub mod monitoring;

use sqlx::PgPool;
use std::sync::Arc;

use crate::utils::config::Config;
use auth::AuthService;
use plugin::PluginService;
use storage::StorageService;
use admin::AdminService;
use smtp::SmtpService;
use config::ConfigService;
use backup::BackupService;
use monitoring::MonitoringService;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: Arc<Config>,
    pub auth_service: Arc<AuthService>,
    pub plugin_service: Arc<PluginService>,
    pub storage_service: Arc<StorageService>,
    pub admin_service: Arc<AdminService>,
    pub smtp_service: Arc<SmtpService>,
    pub config_service: Arc<ConfigService>,
    pub backup_service: Arc<BackupService>,
    pub monitoring_service: Arc<MonitoringService>,
}

impl AppState {
    pub async fn new(db_pool: PgPool, config: Config) -> anyhow::Result<Self> {
        let config = Arc::new(config);
        let storage_service = Arc::new(StorageService::new(config.clone())?);
        let auth_service = Arc::new(AuthService::new(db_pool.clone(), config.clone()));
        let plugin_service = Arc::new(PluginService::new(
            db_pool.clone(),
            storage_service.clone(),
            config.clone(),
        ));
        let admin_service = Arc::new(AdminService::new(db_pool.clone(), config.clone()));
        let smtp_service = Arc::new(SmtpService::new(config.smtp.clone()));
        let config_service = Arc::new(ConfigService::new(
            db_pool.clone(),
            (*config).clone(),
            smtp_service.clone(),
        ));
        
        // Load persisted configuration from database
        config_service.load_persisted_config().await?;
        
        let backup_service = Arc::new(BackupService::new(
            db_pool.clone(),
            config.clone(),
        ));
        let monitoring_service = Arc::new(MonitoringService::new(
            db_pool.clone(),
            config.clone(),
            smtp_service.clone(),
        ));

        Ok(Self {
            db_pool,
            config,
            auth_service,
            plugin_service,
            storage_service,
            admin_service,
            smtp_service,
            config_service,
            backup_service,
            monitoring_service,
        })
    }
}