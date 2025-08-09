use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub storage: StorageConfig,
    pub cors: CorsConfig,
    #[serde(default)]
    pub smtp: SmtpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expires_in: i64,
    pub refresh_token_expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub upload_path: String,
    pub max_file_size: u64,
    pub use_cdn: bool,
    pub cdn_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_smtp_host")]
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_smtp_from_address")]
    pub from_address: String,
    #[serde(default = "default_smtp_from_name")]
    pub from_name: String,
    #[serde(default = "default_true")]
    pub use_tls: bool,
}

fn default_smtp_host() -> String {
    "smtp.gmail.com".to_string()
}

fn default_smtp_port() -> u16 {
    587
}

fn default_smtp_from_address() -> String {
    "noreply@geektools.dev".to_string()
}

fn default_smtp_from_name() -> String {
    "GeekTools Plugin Marketplace".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: default_smtp_host(),
            port: default_smtp_port(),
            username: String::new(),
            password: String::new(),
            from_address: default_smtp_from_address(),
            from_name: default_smtp_from_name(),
            use_tls: true,
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        // Load from environment variables first
        dotenvy::dotenv().ok();

        // Try to load from file
        let config = if std::path::Path::new(path).exists() {
            let contents = std::fs::read_to_string(path)?;
            serde_yaml::from_str::<Config>(&contents)?
        } else {
            Self::default()
        };

        // Override with environment variables
        Ok(Self {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or(config.server.host),
                port: env::var("SERVER_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(config.server.port),
                workers: config.server.workers,
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").unwrap_or(config.database.url),
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(config.database.max_connections),
                connect_timeout: config.database.connect_timeout,
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET").unwrap_or(config.jwt.secret),
                access_token_expires_in: env::var("JWT_ACCESS_TOKEN_EXPIRES_IN")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(config.jwt.access_token_expires_in),
                refresh_token_expires_in: env::var("JWT_REFRESH_TOKEN_EXPIRES_IN")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(config.jwt.refresh_token_expires_in),
            },
            storage: StorageConfig {
                upload_path: env::var("STORAGE_UPLOAD_PATH").unwrap_or(config.storage.upload_path),
                max_file_size: env::var("STORAGE_MAX_FILE_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(config.storage.max_file_size),
                use_cdn: env::var("STORAGE_USE_CDN")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(config.storage.use_cdn),
                cdn_base_url: env::var("STORAGE_CDN_BASE_URL").unwrap_or(config.storage.cdn_base_url),
            },
            cors: config.cors,
            smtp: SmtpConfig {
                enabled: env::var("SMTP_ENABLED")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(config.smtp.enabled),
                host: env::var("SMTP_HOST").unwrap_or(config.smtp.host),
                port: env::var("SMTP_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(config.smtp.port),
                username: env::var("SMTP_USERNAME").unwrap_or(config.smtp.username),
                password: env::var("SMTP_PASSWORD").unwrap_or(config.smtp.password),
                from_address: env::var("SMTP_FROM_ADDRESS").unwrap_or(config.smtp.from_address),
                from_name: env::var("SMTP_FROM_NAME").unwrap_or(config.smtp.from_name),
                use_tls: env::var("SMTP_USE_TLS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(config.smtp.use_tls),
            },
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                workers: None,
            },
            database: DatabaseConfig {
                url: "sqlite://marketplace.db".to_string(),
                max_connections: 10,
                connect_timeout: 30,
            },
            jwt: JwtConfig {
                secret: "your-secret-key-change-this-in-production".to_string(),
                access_token_expires_in: 3600,  // 1 hour
                refresh_token_expires_in: 86400 * 7, // 7 days
            },
            storage: StorageConfig {
                upload_path: "./uploads".to_string(),
                max_file_size: 100 * 1024 * 1024, // 100MB
                use_cdn: false,
                cdn_base_url: "https://cdn.geektools.dev".to_string(),
            },
            cors: CorsConfig {
                allowed_origins: vec![
                    "http://localhost:3000".to_string(),
                    "http://localhost:3001".to_string(),
                    "http://localhost:8080".to_string(),
                ],
                allowed_methods: vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                ],
                allowed_headers: vec![
                    "Authorization".to_string(),
                    "Content-Type".to_string(),
                    "Accept".to_string(),
                ],
            },
            smtp: SmtpConfig {
                enabled: false,
                host: "smtp.gmail.com".to_string(),
                port: 587,
                username: "".to_string(),
                password: "".to_string(),
                from_address: "noreply@geektools.dev".to_string(),
                from_name: "GeekTools Plugin Marketplace".to_string(),
                use_tls: true,
            },
        }
    }
}