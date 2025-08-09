-- SQLite doesn't support ENUM types, we'll use TEXT with CHECK constraints

-- Users table
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    display_name TEXT,
    is_active BOOLEAN DEFAULT 1,
    is_verified BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Plugins table
CREATE TABLE plugins (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    author TEXT NOT NULL,
    current_version TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    downloads INTEGER DEFAULT 0,
    rating REAL DEFAULT 0.00,
    status TEXT DEFAULT 'active' CHECK (status IN ('active', 'deprecated', 'banned')),
    min_geektools_version TEXT,
    homepage_url TEXT,
    repository_url TEXT,
    license TEXT
);

-- Plugin versions table
CREATE TABLE plugin_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    changelog TEXT,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    file_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    downloads INTEGER DEFAULT 0,
    is_stable BOOLEAN DEFAULT 1,
    UNIQUE(plugin_id, version)
);

-- Plugin scripts table
CREATE TABLE plugin_scripts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    script_name TEXT NOT NULL,
    script_file TEXT NOT NULL,
    description TEXT,
    is_executable BOOLEAN DEFAULT 0
);

-- Plugin dependencies table
CREATE TABLE plugin_dependencies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    dependency_id TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    min_version TEXT
);

-- Plugin tags table
CREATE TABLE plugin_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    UNIQUE(plugin_id, tag)
);

-- Plugin ratings table
CREATE TABLE plugin_ratings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, plugin_id)
);

-- Create indexes for better performance
CREATE INDEX idx_plugins_status_downloads ON plugins(status, downloads DESC);
CREATE INDEX idx_plugins_created_at ON plugins(created_at DESC);
CREATE INDEX idx_plugins_rating ON plugins(rating DESC);
CREATE INDEX idx_plugins_updated_at ON plugins(updated_at DESC);
CREATE INDEX idx_plugin_versions_plugin_id ON plugin_versions(plugin_id);
CREATE INDEX idx_plugin_versions_created_at ON plugin_versions(created_at DESC);
CREATE INDEX idx_plugin_tags_plugin_id ON plugin_tags(plugin_id);
CREATE INDEX idx_plugin_tags_tag ON plugin_tags(tag);
CREATE INDEX idx_plugin_ratings_plugin_id ON plugin_ratings(plugin_id);
CREATE INDEX idx_plugin_ratings_user_id ON plugin_ratings(user_id);

-- SQLite triggers for updated_at (SQLite doesn't support functions like PostgreSQL)
CREATE TRIGGER update_users_updated_at 
    AFTER UPDATE ON users
    FOR EACH ROW
    BEGIN
        UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER update_plugins_updated_at 
    AFTER UPDATE ON plugins
    FOR EACH ROW
    BEGIN
        UPDATE plugins SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER update_plugin_ratings_updated_at 
    AFTER UPDATE ON plugin_ratings
    FOR EACH ROW
    BEGIN
        UPDATE plugin_ratings SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;
