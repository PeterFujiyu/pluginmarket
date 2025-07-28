-- noinspection SqlNoDataSourceInspectionForFile

-- Initialize database for GeekTools Plugin Marketplace
-- This script ensures the database is properly set up

-- Create database if it doesn't exist (handled by POSTGRES_DB env var)
-- Just ensure we're connected to the right database

-- Set timezone
SET timezone = 'UTC';

-- Create extensions if needed
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Log initialization
DO $$
BEGIN
    RAISE NOTICE 'GeekTools Plugin Marketplace database initialized successfully';
END $$;