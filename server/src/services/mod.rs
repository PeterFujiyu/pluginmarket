pub mod auth;
pub mod plugin;
pub mod storage;
pub mod admin;
pub mod smtp;

use sqlx::SqlitePool;
use std::sync::Arc;

use crate::utils::config::Config;
use auth::AuthService;
use plugin::PluginService;
use storage::StorageService;
use admin::AdminService;
use smtp::SmtpService;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub config: Arc<Config>,
    pub auth_service: Arc<AuthService>,
    pub plugin_service: Arc<PluginService>,
    pub storage_service: Arc<StorageService>,
    pub admin_service: Arc<AdminService>,
    pub smtp_service: Arc<SmtpService>,
}

impl AppState {
    pub async fn new(db_pool: SqlitePool, config: Config) -> anyhow::Result<Self> {
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

        Ok(Self {
            db_pool,
            config,
            auth_service,
            plugin_service,
            storage_service,
            admin_service,
            smtp_service,
        })
    }
}