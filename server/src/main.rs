mod models;
mod handlers;
mod services;
mod utils;
mod middleware;

use axum::{
    extract::DefaultBodyLimit,
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{get, post},
    Router,
};
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use handlers::{auth, plugins, search, health, admin, config, backup, monitoring};
use services::AppState;
use utils::config::Config;

#[derive(Parser)]
#[command(name = "geektools-marketplace-server")]
#[command(about = "GeekTools Plugin Marketplace Server")]
struct Cli {
    #[arg(short, long, default_value = "config/config.yaml")]
    config: String,
    
    #[arg(short, long)]
    port: Option<u16>,
    
    #[arg(short, long)]
    database_url: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Load configuration
    let config = Config::from_file(&cli.config)?;
    
    // Override config with CLI arguments
    let port = cli.port.unwrap_or(config.server.port);
    let database_url = cli.database_url.unwrap_or(config.database.url.clone());

    info!("Starting GeekTools Marketplace Server on port {}", port);
    
    // Log configuration details
    log_configuration(&config, port, &database_url);

    // Connect to database
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Create application state
    let state = AppState::new(pool, config).await?;

    // Build application router
    let app = create_app(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Server listening on http://0.0.0.0:{}", port);
    
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_app(state: AppState) -> Router {
    // CORS configuration - Allow specific origins with credentials
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
        .allow_origin("http://127.0.0.1:8080".parse::<HeaderValue>().unwrap())
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_origin("http://127.0.0.1:3000".parse::<HeaderValue>().unwrap())
        .allow_origin("null".parse::<HeaderValue>().unwrap()) // For file:// protocol
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true);

    // API routes
    let api_routes = Router::new()
        // Auth routes
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh_token))
        .route("/auth/send-code", post(auth::send_verification_code))
        .route("/auth/verify-code", post(auth::verify_code_and_login))
        
        // Plugin routes
        .route("/plugins", get(plugins::list_plugins))
        .route("/plugins", post(plugins::upload_plugin))
        .route("/plugins/upload", post(plugins::upload_plugin_temp)) // Temporary endpoint without auth
        .route("/plugins/:id", get(plugins::get_plugin))
        .route("/plugins/:id/download", get(plugins::download_plugin))
        .route("/plugins/:id/stats", get(plugins::get_plugin_stats))
        .route("/plugins/:id/ratings", get(plugins::get_plugin_ratings))
        .route("/plugins/:id/ratings", post(plugins::create_rating))
        
        // Search routes
        .route("/search", post(search::advanced_search))
        .route("/search/suggestions", get(search::search_suggestions))
        
        // Health check
        .route("/health", get(health::health_check))
        .route("/metrics", get(health::metrics))
        
        // Admin routes
        .route("/admin/dashboard", get(admin::get_dashboard_stats))
        .route("/admin/users", get(admin::get_users_for_management))
        .route("/admin/users/update-email", post(admin::update_user_email))
        .route("/admin/users/ban", post(admin::ban_user))
        .route("/admin/users/unban", post(admin::unban_user))
        .route("/admin/plugins", get(admin::get_plugins_for_management))
        .route("/admin/plugins/delete", post(admin::delete_plugin))
        .route("/admin/sql/execute", post(admin::execute_sql))
        .route("/admin/login-activities", get(admin::get_user_login_activities))
        .route("/admin/recent-logins", get(admin::get_recent_logins))
        
        // Configuration Management routes
        .route("/admin/config", get(config::get_config))
        .route("/admin/config/update", post(config::update_config))
        .route("/admin/config/test", post(config::test_config))
        .route("/admin/config/test/email", post(config::test_email))
        .route("/admin/config/rollback", post(config::rollback_config))
        .route("/admin/config/history", get(config::get_config_history))
        .route("/admin/config/snapshot", post(config::create_config_snapshot))
        .route("/admin/config/compare", get(config::compare_config_versions))
        
        // Backup Management routes
        .route("/admin/backup/stats", get(backup::get_backup_stats))
        .route("/admin/backup/list", get(backup::list_backups))
        .route("/admin/backup/create", post(backup::create_backup))
        .route("/admin/backup/restore", post(backup::restore_backup))
        .route("/admin/backup/delete", post(backup::delete_backup))
        .route("/admin/backup/:id/download", get(backup::download_backup))
        .route("/admin/backup/status/:operation_id", get(backup::get_operation_status))
        .route("/admin/backup/operations", get(backup::list_operations))
        .route("/admin/backup/schedules", get(backup::list_schedules))
        .route("/admin/backup/schedule", post(backup::create_schedule))
        .route("/admin/backup/schedule/update", post(backup::update_schedule))
        .route("/admin/backup/schedule/:id/delete", post(backup::delete_schedule))
        .route("/admin/backup/schedule/:id/toggle", post(backup::toggle_schedule))
        
        // System Monitoring routes
        .route("/admin/monitor/overview", get(monitoring::get_monitoring_overview))
        .route("/admin/monitor/system", get(monitoring::get_system_metrics))
        .route("/admin/monitor/services", get(monitoring::get_services_status))
        .route("/admin/monitor/database", get(monitoring::get_database_status))
        .route("/admin/monitor/smtp", get(monitoring::get_smtp_status))
        .route("/admin/monitor/logs", get(monitoring::get_system_logs))
        .route("/admin/monitor/test/database", post(monitoring::test_database_connection))
        .route("/admin/monitor/test/smtp", post(monitoring::test_smtp_connection))
        .route("/admin/monitor/performance", get(monitoring::get_performance_chart_data))
        .route("/admin/monitor/metrics/record", post(monitoring::record_system_metrics))
        .route("/admin/monitor/cleanup", post(monitoring::cleanup_old_data))
        .route("/admin/monitor/log", post(monitoring::log_system_event))
        
        .with_state(state);

    // Main application
    Router::new()
        .nest("/api/v1", api_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(cors)
                .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB max file size
        )
}

fn log_configuration(config: &Config, port: u16, database_url: &str) {
    info!("=== Server Configuration ===");
    info!("Server host: {}", config.server.host);
    info!("Server port: {}", port);
    if let Some(workers) = config.server.workers {
        info!("Server workers: {}", workers);
    }
    
    info!("=== Database Configuration ===");
    // Mask password in database URL for security
    let masked_db_url = mask_database_password(database_url);
    info!("Database URL: {}", masked_db_url);
    info!("Database max connections: {}", config.database.max_connections);
    info!("Database connect timeout: {}s", config.database.connect_timeout);
    
    info!("=== JWT Configuration ===");
    info!("JWT access token expires in: {}s", config.jwt.access_token_expires_in);
    info!("JWT refresh token expires in: {}s", config.jwt.refresh_token_expires_in);
    info!("JWT secret configured: {}", !config.jwt.secret.is_empty());
    
    info!("=== Storage Configuration ===");
    info!("Storage upload path: {}", config.storage.upload_path);
    info!("Storage max file size: {} bytes ({}MB)", config.storage.max_file_size, config.storage.max_file_size / 1024 / 1024);
    info!("Storage use CDN: {}", config.storage.use_cdn);
    if config.storage.use_cdn {
        info!("Storage CDN base URL: {}", config.storage.cdn_base_url);
    }
    
    info!("=== SMTP Configuration ===");
    info!("SMTP enabled: {}", config.smtp.enabled);
    if config.smtp.enabled {
        info!("SMTP host: {}", config.smtp.host);
        info!("SMTP port: {}", config.smtp.port);
        info!("SMTP username: {}", config.smtp.username);
        info!("SMTP password configured: {}", !config.smtp.password.is_empty());
        info!("SMTP from address: {}", config.smtp.from_address);
        info!("SMTP from name: {}", config.smtp.from_name);
        info!("SMTP use TLS: {}", config.smtp.use_tls);
        
        // Log SMTP status
        if config.smtp.username.is_empty() || config.smtp.password.is_empty() {
            tracing::warn!("SMTP is enabled but credentials are missing - email sending will be disabled");
        } else {
            info!("SMTP fully configured and ready for email sending");
        }
    } else {
        info!("SMTP is disabled - verification codes will be displayed in logs");
    }
    
    info!("=== CORS Configuration ===");
    info!("CORS allowed origins: {:?}", config.cors.allowed_origins);
    info!("CORS allowed methods: {:?}", config.cors.allowed_methods);
    info!("CORS allowed headers: {:?}", config.cors.allowed_headers);
    
    info!("=== Configuration Loading Complete ===");
}

fn mask_database_password(database_url: &str) -> String {
    // Mask password for security in logs
    if let Ok(url) = url::Url::parse(database_url) {
        if url.password().is_some() {
            let mut masked_url = url.clone();
            masked_url.set_password(Some("***")).ok();
            masked_url.to_string()
        } else {
            database_url.to_string()
        }
    } else {
        // If parsing fails, try to manually mask password part
        if let Some(password_start) = database_url.find("://") {
            if let Some(at_pos) = database_url[password_start + 3..].find('@') {
                let before_creds = &database_url[..password_start + 3];
                let after_creds = &database_url[password_start + 3 + at_pos..];
                if let Some(colon_pos) = database_url[password_start + 3..password_start + 3 + at_pos].find(':') {
                    let username = &database_url[password_start + 3..password_start + 3 + colon_pos];
                    format!("{}{}:***{}", before_creds, username, after_creds)
                } else {
                    database_url.to_string()
                }
            } else {
                database_url.to_string()
            }
        } else {
            database_url.to_string()
        }
    }
}