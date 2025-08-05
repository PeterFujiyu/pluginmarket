-- Fix config_history table schema

-- Drop existing table if needed and recreate with correct structure
DROP TABLE IF EXISTS config_history CASCADE;

CREATE TABLE config_history (
    id SERIAL PRIMARY KEY,
    config_type VARCHAR(50) NOT NULL,
    old_config JSONB,
    new_config JSONB NOT NULL,
    changed_by_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    change_reason TEXT,
    version VARCHAR(50) NOT NULL,
    ip_address VARCHAR(45),
    changed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT config_history_config_type_check 
        CHECK (config_type IN ('smtp', 'database', 'server', 'storage', 'snapshot'))
);

-- Recreate indexes
CREATE INDEX idx_config_history_config_type ON config_history(config_type);
CREATE INDEX idx_config_history_changed_at ON config_history(changed_at DESC);
CREATE INDEX idx_config_history_changed_by_id ON config_history(changed_by_id);
CREATE INDEX idx_config_history_version ON config_history(version);