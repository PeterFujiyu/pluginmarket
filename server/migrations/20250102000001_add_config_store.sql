-- Add config_store table for persistent configuration storage
CREATE TABLE IF NOT EXISTS config_store (
    key VARCHAR(255) PRIMARY KEY,
    value JSONB NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create index for better performance
CREATE INDEX IF NOT EXISTS idx_config_store_updated_at ON config_store(updated_at);

-- Insert default configuration if not exists
INSERT INTO config_store (key, value, updated_at) 
VALUES ('current_config', '{
    "smtp": {
        "enabled": false,
        "host": "smtp.gmail.com",
        "port": 587,
        "username": "",
        "password": "",
        "from_address": "noreply@geektools.dev",
        "from_name": "GeekTools Plugin Marketplace",
        "use_tls": true
    },
    "database": {
        "max_connections": 10,
        "connect_timeout": 30
    },
    "server": {
        "host": "0.0.0.0",
        "port": 3000,
        "jwt_secret": "change-this-to-a-secure-secret-in-production",
        "jwt_access_token_expires_in": 3600,
        "jwt_refresh_token_expires_in": 604800,
        "cors_origins": ["http://localhost:8080", "http://localhost:3000"]
    },
    "storage": {
        "upload_path": "./uploads",
        "max_file_size": 104857600,
        "use_cdn": false,
        "cdn_base_url": "https://cdn.geektools.dev"
    }
}'::jsonb, NOW())
ON CONFLICT (key) DO NOTHING;