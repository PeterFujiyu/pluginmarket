-- Add configuration management tables

-- Configuration history table
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

-- Create indexes for better performance
CREATE INDEX idx_config_history_config_type ON config_history(config_type);
CREATE INDEX idx_config_history_changed_at ON config_history(changed_at DESC);
CREATE INDEX idx_config_history_changed_by_id ON config_history(changed_by_id);
CREATE INDEX idx_config_history_version ON config_history(version);

-- Backup metadata table
CREATE TABLE backup_metadata (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    backup_type VARCHAR(20) NOT NULL DEFAULT 'full',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    file_path TEXT,
    file_size BIGINT,
    created_by_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT,
    compressed BOOLEAN DEFAULT true,
    
    CONSTRAINT backup_metadata_type_check 
        CHECK (backup_type IN ('full', 'data', 'schema')),
    CONSTRAINT backup_metadata_status_check 
        CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled'))
);

-- Create indexes for backup metadata
CREATE INDEX idx_backup_metadata_status ON backup_metadata(status);
CREATE INDEX idx_backup_metadata_created_at ON backup_metadata(created_at DESC);
CREATE INDEX idx_backup_metadata_created_by_id ON backup_metadata(created_by_id);
CREATE INDEX idx_backup_metadata_backup_type ON backup_metadata(backup_type);

-- Backup schedules table
CREATE TABLE backup_schedules (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    frequency VARCHAR(20) NOT NULL,
    schedule_time TIME NOT NULL,
    schedule_day INTEGER, -- For weekly schedules (0=Sunday, 6=Saturday)
    schedule_date INTEGER, -- For monthly schedules (1-31)
    retention_count INTEGER NOT NULL DEFAULT 7,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_run_at TIMESTAMP WITH TIME ZONE,
    next_run_at TIMESTAMP WITH TIME ZONE,
    last_backup_id INTEGER REFERENCES backup_metadata(id),
    created_by_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT backup_schedules_frequency_check 
        CHECK (frequency IN ('daily', 'weekly', 'monthly')),
    CONSTRAINT backup_schedules_day_check 
        CHECK (schedule_day IS NULL OR (schedule_day >= 0 AND schedule_day <= 6)),
    CONSTRAINT backup_schedules_date_check 
        CHECK (schedule_date IS NULL OR (schedule_date >= 1 AND schedule_date <= 31))
);

-- Create indexes for backup schedules
CREATE INDEX idx_backup_schedules_enabled ON backup_schedules(enabled);
CREATE INDEX idx_backup_schedules_next_run_at ON backup_schedules(next_run_at);
CREATE INDEX idx_backup_schedules_frequency ON backup_schedules(frequency);

-- System monitoring logs table
CREATE TABLE system_logs (
    id SERIAL PRIMARY KEY,
    log_level VARCHAR(10) NOT NULL,
    component VARCHAR(50) NOT NULL,
    message TEXT NOT NULL,
    details JSONB,
    user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    ip_address VARCHAR(45),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT system_logs_level_check 
        CHECK (log_level IN ('DEBUG', 'INFO', 'WARN', 'ERROR', 'FATAL'))
);

-- Create indexes for system logs
CREATE INDEX idx_system_logs_level ON system_logs(log_level);
CREATE INDEX idx_system_logs_component ON system_logs(component);
CREATE INDEX idx_system_logs_created_at ON system_logs(created_at DESC);
CREATE INDEX idx_system_logs_user_id ON system_logs(user_id);

-- System metrics table for storing performance data
CREATE TABLE system_metrics (
    id SERIAL PRIMARY KEY,
    metric_type VARCHAR(50) NOT NULL,
    metric_name VARCHAR(100) NOT NULL,
    metric_value NUMERIC NOT NULL,
    unit VARCHAR(20),
    tags JSONB,
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    UNIQUE(metric_type, metric_name, recorded_at)
);

-- Create indexes for system metrics
CREATE INDEX idx_system_metrics_type_name ON system_metrics(metric_type, metric_name);
CREATE INDEX idx_system_metrics_recorded_at ON system_metrics(recorded_at DESC);
CREATE INDEX idx_system_metrics_tags ON system_metrics USING GIN(tags);

-- Add trigger to update backup_schedules.updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_backup_schedules_updated_at 
    BEFORE UPDATE ON backup_schedules 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert initial system log entry
INSERT INTO system_logs (log_level, component, message, details) 
VALUES ('INFO', 'migration', 'Configuration management tables created', 
        '{"migration": "20250101000001_add_config_management", "tables": ["config_history", "backup_metadata", "backup_schedules", "system_logs", "system_metrics"]}'::jsonb);