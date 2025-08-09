# 数据库模式设计

## 概述

GeekTools 插件市场使用 SQLite 作为主数据库，采用轻量级关系型数据库设计模式。数据库结构支持用户管理、插件管理、评分系统、管理后台等核心功能，并提供完整的审计日志和性能优化。

## 数据库配置

### 基础信息

- **数据库类型**: SQLite 3+
- **字符集**: UTF-8
- **时区**: UTC
- **连接模式**: WAL模式（Write-Ahead Logging）
- **事务隔离级别**: SERIALIZABLE
- **外键约束**: 启用

### 连接配置

```env
DATABASE_URL=sqlite://./data/marketplace.db
DATABASE_MAX_CONNECTIONS=10
SQLITE_SYNCHRONOUS=NORMAL
SQLITE_JOURNAL_MODE=WAL
```

## 核心数据表

### 1. 用户表 (users)

存储用户基本信息和认证数据。

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL CHECK(length(username) >= 3 AND length(username) <= 100),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    display_name TEXT,
    role TEXT DEFAULT 'user' CHECK(role IN ('user', 'developer', 'moderator', 'admin', 'superadmin')),
    is_active INTEGER DEFAULT 1 CHECK(is_active IN (0, 1)),
    is_verified INTEGER DEFAULT 0 CHECK(is_verified IN (0, 1)),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | 用户唯一标识 |
| username | TEXT | UNIQUE, NOT NULL | 用户名，3-100字符 |
| email | TEXT | UNIQUE, NOT NULL | 邮箱地址 |
| password_hash | TEXT | NOT NULL | bcrypt加密后的密码 |
| display_name | TEXT | NULL | 显示名称 |
| role | TEXT | DEFAULT 'user' | 用户角色 |
| is_active | INTEGER | DEFAULT 1 | 账户是否激活 (0/1) |
| is_verified | INTEGER | DEFAULT 0 | 邮箱是否验证 (0/1) |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP | 创建时间 |
| updated_at | DATETIME | DEFAULT CURRENT_TIMESTAMP | 更新时间 |

**用户角色说明**:
- `user`: 普通用户
- `developer`: 插件开发者
- `moderator`: 版主
- `admin`: 管理员
- `superadmin`: 超级管理员

**索引设计**:
```sql
-- 主键索引（自动创建）
-- 唯一索引（自动创建）
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_is_active ON users(is_active);
CREATE INDEX idx_users_created_at ON users(created_at);
```

### 2. 插件表 (plugins)

存储插件的基本信息和元数据。

```sql
CREATE TABLE plugins (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    author TEXT NOT NULL,
    current_version TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    downloads INTEGER DEFAULT 0,
    rating REAL DEFAULT 0.00 CHECK(rating >= 0.00 AND rating <= 5.00),
    status TEXT DEFAULT 'active' CHECK(status IN ('active', 'deprecated', 'banned')),
    min_geektools_version TEXT,
    homepage_url TEXT CHECK(length(homepage_url) <= 500),
    repository_url TEXT CHECK(length(repository_url) <= 500),
    license TEXT CHECK(length(license) <= 100)
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | TEXT | PRIMARY KEY | 插件唯一标识符 |
| name | TEXT | NOT NULL | 插件名称 |
| description | TEXT | NULL | 插件描述 |
| author | TEXT | NOT NULL | 作者名称 |
| current_version | TEXT | NOT NULL | 当前版本号 |
| downloads | INTEGER | DEFAULT 0 | 总下载次数 |
| rating | REAL | DEFAULT 0.00 | 平均评分 (0.00-5.00) |
| status | TEXT | DEFAULT 'active' | 插件状态 |
| min_geektools_version | TEXT | NULL | 最低GeekTools版本要求 |
| homepage_url | TEXT | NULL | 主页URL |
| repository_url | TEXT | NULL | 代码仓库URL |
| license | TEXT | NULL | 许可证类型 |

**插件状态说明**:
- `active`: 正常可用
- `deprecated`: 已弃用
- `banned`: 已禁用

**索引设计**:
```sql
CREATE INDEX idx_plugins_status_downloads ON plugins(status, downloads DESC);
CREATE INDEX idx_plugins_created_at ON plugins(created_at DESC);
CREATE INDEX idx_plugins_rating ON plugins(rating DESC);
CREATE INDEX idx_plugins_updated_at ON plugins(updated_at DESC);
CREATE INDEX idx_plugins_author ON plugins(author);
```

### 3. 插件版本表 (plugin_versions)

存储插件的版本历史和文件信息。

```sql
CREATE TABLE plugin_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL,
    version TEXT NOT NULL,
    changelog TEXT,
    file_path TEXT NOT NULL CHECK(length(file_path) <= 500),
    file_size INTEGER NOT NULL CHECK(file_size > 0 AND file_size <= 104857600),
    file_hash TEXT NOT NULL CHECK(length(file_hash) = 64),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    downloads INTEGER DEFAULT 0,
    is_stable INTEGER DEFAULT 1 CHECK(is_stable IN (0, 1)),
    FOREIGN KEY (plugin_id) REFERENCES plugins(id) ON DELETE CASCADE,
    UNIQUE(plugin_id, version)
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | 版本记录ID |
| plugin_id | TEXT | FOREIGN KEY | 关联插件ID |
| version | TEXT | NOT NULL | 版本号 |
| changelog | TEXT | NULL | 版本更新日志 |
| file_path | TEXT | NOT NULL | 文件存储路径 |
| file_size | INTEGER | NOT NULL | 文件大小（字节） |
| file_hash | TEXT | NOT NULL | 文件SHA256哈希 |
| downloads | INTEGER | DEFAULT 0 | 该版本下载次数 |
| is_stable | INTEGER | DEFAULT 1 | 是否为稳定版本 (0/1) |

**索引设计**:
```sql
CREATE INDEX idx_plugin_versions_plugin_id ON plugin_versions(plugin_id);
CREATE INDEX idx_plugin_versions_created_at ON plugin_versions(created_at DESC);
CREATE INDEX idx_plugin_versions_is_stable ON plugin_versions(is_stable);
```

### 4. 插件脚本表 (plugin_scripts)

存储插件包内的脚本文件信息。

```sql
CREATE TABLE plugin_scripts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL,
    version TEXT NOT NULL,
    script_name TEXT NOT NULL,
    script_file TEXT NOT NULL,
    description TEXT,
    is_executable INTEGER DEFAULT 0 CHECK(is_executable IN (0, 1)),
    FOREIGN KEY (plugin_id) REFERENCES plugins(id) ON DELETE CASCADE,
    FOREIGN KEY (plugin_id, version) REFERENCES plugin_versions(plugin_id, version) ON DELETE CASCADE
);
```

### 5. 插件依赖表 (plugin_dependencies)

存储插件之间的依赖关系。

```sql
CREATE TABLE plugin_dependencies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL,
    dependency_id TEXT NOT NULL,
    min_version TEXT,
    FOREIGN KEY (plugin_id) REFERENCES plugins(id) ON DELETE CASCADE,
    FOREIGN KEY (dependency_id) REFERENCES plugins(id) ON DELETE CASCADE,
    UNIQUE(plugin_id, dependency_id),
    CHECK (plugin_id != dependency_id)
);
```

**索引设计**:
```sql
CREATE INDEX idx_plugin_dependencies_plugin_id ON plugin_dependencies(plugin_id);
CREATE INDEX idx_plugin_dependencies_dependency_id ON plugin_dependencies(dependency_id);
```

### 6. 插件标签表 (plugin_tags)

存储插件的标签信息，支持多标签分类。

```sql
CREATE TABLE plugin_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL,
    tag TEXT NOT NULL CHECK(length(tag) <= 100),
    FOREIGN KEY (plugin_id) REFERENCES plugins(id) ON DELETE CASCADE,
    UNIQUE(plugin_id, tag)
);
```

**索引设计**:
```sql
CREATE INDEX idx_plugin_tags_plugin_id ON plugin_tags(plugin_id);
CREATE INDEX idx_plugin_tags_tag ON plugin_tags(tag);
```

### 7. 插件评分表 (plugin_ratings)

存储用户对插件的评分和评论。

```sql
CREATE TABLE plugin_ratings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (plugin_id) REFERENCES plugins(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, plugin_id)
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | 评分记录ID |
| plugin_id | TEXT | FOREIGN KEY | 关联插件ID |
| user_id | INTEGER | FOREIGN KEY | 评分用户ID |
| rating | INTEGER | CHECK(1-5) | 评分 (1-5星) |
| review | TEXT | NULL | 评论内容 |

**索引设计**:
```sql
CREATE INDEX idx_plugin_ratings_plugin_id ON plugin_ratings(plugin_id);
CREATE INDEX idx_plugin_ratings_user_id ON plugin_ratings(user_id);
CREATE INDEX idx_plugin_ratings_rating ON plugin_ratings(rating);
CREATE INDEX idx_plugin_ratings_created_at ON plugin_ratings(created_at DESC);
```

## 管理和审计表

### 8. 用户登录活动表 (user_login_activities)

记录用户登录活动，用于安全审计和分析。

```sql
CREATE TABLE user_login_activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    email TEXT NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    login_time DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    logout_time DATETIME,
    session_duration INTEGER, -- in seconds
    login_method TEXT DEFAULT 'email_verification' CHECK(login_method IN ('email_verification', 'password', 'oauth', 'api_token')),
    is_successful INTEGER DEFAULT 1 CHECK(is_successful IN (0, 1)),
    failure_reason TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | 活动记录ID |
| user_id | INTEGER | FOREIGN KEY | 用户ID |
| email | TEXT | NOT NULL | 登录邮箱 |
| ip_address | TEXT | NULL | 客户端IP地址 |
| user_agent | TEXT | NULL | 用户代理字符串 |
| login_time | DATETIME | NOT NULL | 登录时间 |
| logout_time | DATETIME | NULL | 登出时间 |
| session_duration | INTEGER | NULL | 会话持续时间（秒） |
| login_method | TEXT | DEFAULT | 登录方式 |
| is_successful | INTEGER | DEFAULT 1 | 登录是否成功 (0/1) |
| failure_reason | TEXT | NULL | 失败原因 |

**登录方式说明**:
- `email_verification`: 邮箱验证码登录
- `password`: 密码登录
- `oauth`: OAuth登录
- `api_token`: API Token登录

**索引设计**:
```sql
CREATE INDEX idx_user_login_activities_user_id ON user_login_activities(user_id);
CREATE INDEX idx_user_login_activities_login_time ON user_login_activities(login_time);
CREATE INDEX idx_user_login_activities_email ON user_login_activities(email);
CREATE INDEX idx_user_login_activities_ip_address ON user_login_activities(ip_address);
CREATE INDEX idx_user_login_activities_is_successful ON user_login_activities(is_successful);
```

### 9. 管理员SQL日志表 (admin_sql_logs)

记录管理员执行的SQL操作，用于审计。

```sql
CREATE TABLE admin_sql_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    admin_user_id INTEGER NOT NULL,
    admin_email TEXT NOT NULL,
    sql_query TEXT NOT NULL,
    execution_time_ms INTEGER,
    rows_affected INTEGER,
    is_successful INTEGER DEFAULT 1 CHECK(is_successful IN (0, 1)),
    error_message TEXT,
    ip_address TEXT,
    executed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (admin_user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | 日志记录ID |
| admin_user_id | INTEGER | FOREIGN KEY | 管理员用户ID |
| admin_email | TEXT | NOT NULL | 管理员邮箱 |
| sql_query | TEXT | NOT NULL | 执行的SQL语句 |
| execution_time_ms | INTEGER | NULL | 执行时间（毫秒） |
| rows_affected | INTEGER | NULL | 影响行数 |
| is_successful | INTEGER | DEFAULT 1 | 执行是否成功 (0/1) |
| error_message | TEXT | NULL | 错误信息 |
| ip_address | TEXT | NULL | 执行者IP地址 |

**索引设计**:
```sql
CREATE INDEX idx_admin_sql_logs_admin_user_id ON admin_sql_logs(admin_user_id);
CREATE INDEX idx_admin_sql_logs_executed_at ON admin_sql_logs(executed_at);
CREATE INDEX idx_admin_sql_logs_is_successful ON admin_sql_logs(is_successful);
```

### 10. 用户资料变更表 (user_profile_changes)

记录用户资料的变更历史。

```sql
CREATE TABLE user_profile_changes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    changed_by_user_id INTEGER NOT NULL,
    field_name TEXT NOT NULL CHECK(field_name IN ('email', 'username', 'display_name', 'role', 'is_active', 'is_verified')),
    old_value TEXT,
    new_value TEXT,
    change_reason TEXT,
    ip_address TEXT,
    changed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (changed_by_user_id) REFERENCES users(id)
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | 变更记录ID |
| user_id | INTEGER | FOREIGN KEY | 被修改的用户ID |
| changed_by_user_id | INTEGER | FOREIGN KEY | 执行修改的用户ID |
| field_name | TEXT | NOT NULL | 修改的字段名 |
| old_value | TEXT | NULL | 原值 |
| new_value | TEXT | NULL | 新值 |
| change_reason | TEXT | NULL | 修改原因 |
| ip_address | TEXT | NULL | 修改者IP地址 |

**可变更字段说明**:
- `email`: 邮箱地址
- `username`: 用户名
- `display_name`: 显示名称
- `role`: 用户角色
- `is_active`: 激活状态
- `is_verified`: 验证状态

**索引设计**:
```sql
CREATE INDEX idx_user_profile_changes_user_id ON user_profile_changes(user_id);
CREATE INDEX idx_user_profile_changes_changed_at ON user_profile_changes(changed_at);
CREATE INDEX idx_user_profile_changes_changed_by_user_id ON user_profile_changes(changed_by_user_id);
```

## 数据库触发器

### 1. 自动更新时间戳

创建触发器自动更新 `updated_at` 字段：

```sql
-- 为用户表创建触发器
CREATE TRIGGER update_users_updated_at 
    AFTER UPDATE ON users
    FOR EACH ROW
    BEGIN
        UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

-- 为插件表创建触发器
CREATE TRIGGER update_plugins_updated_at 
    AFTER UPDATE ON plugins
    FOR EACH ROW
    BEGIN
        UPDATE plugins SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

-- 为评分表创建触发器
CREATE TRIGGER update_plugin_ratings_updated_at 
    AFTER UPDATE ON plugin_ratings
    FOR EACH ROW
    BEGIN
        UPDATE plugin_ratings SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;
```

### 2. 计算插件平均评分

创建触发器自动计算和更新插件平均评分：

```sql
-- 插入评分后更新平均分
CREATE TRIGGER update_plugin_rating_on_insert
    AFTER INSERT ON plugin_ratings
    FOR EACH ROW
    BEGIN
        UPDATE plugins 
        SET rating = (
            SELECT ROUND(AVG(rating), 2) 
            FROM plugin_ratings 
            WHERE plugin_id = NEW.plugin_id
        )
        WHERE id = NEW.plugin_id;
    END;

-- 更新评分后更新平均分
CREATE TRIGGER update_plugin_rating_on_update
    AFTER UPDATE ON plugin_ratings
    FOR EACH ROW
    BEGIN
        UPDATE plugins 
        SET rating = (
            SELECT ROUND(AVG(rating), 2) 
            FROM plugin_ratings 
            WHERE plugin_id = NEW.plugin_id
        )
        WHERE id = NEW.plugin_id;
    END;

-- 删除评分后更新平均分
CREATE TRIGGER update_plugin_rating_on_delete
    AFTER DELETE ON plugin_ratings
    FOR EACH ROW
    BEGIN
        UPDATE plugins 
        SET rating = (
            SELECT COALESCE(ROUND(AVG(rating), 2), 0.00) 
            FROM plugin_ratings 
            WHERE plugin_id = OLD.plugin_id
        )
        WHERE id = OLD.plugin_id;
    END;
```

### 3. 记录用户资料变更

创建触发器自动记录用户资料变更：

```sql
CREATE TRIGGER log_user_profile_changes
    AFTER UPDATE ON users
    FOR EACH ROW
    WHEN OLD.email != NEW.email 
      OR OLD.username != NEW.username 
      OR OLD.display_name != NEW.display_name 
      OR OLD.role != NEW.role
      OR OLD.is_active != NEW.is_active
      OR OLD.is_verified != NEW.is_verified
    BEGIN
        -- 记录邮箱变更
        INSERT INTO user_profile_changes (user_id, changed_by_user_id, field_name, old_value, new_value, change_reason)
        SELECT NEW.id, NEW.id, 'email', OLD.email, NEW.email, 'User profile update'
        WHERE OLD.email != NEW.email;
        
        -- 记录用户名变更
        INSERT INTO user_profile_changes (user_id, changed_by_user_id, field_name, old_value, new_value, change_reason)
        SELECT NEW.id, NEW.id, 'username', OLD.username, NEW.username, 'User profile update'
        WHERE OLD.username != NEW.username;
        
        -- 记录显示名称变更
        INSERT INTO user_profile_changes (user_id, changed_by_user_id, field_name, old_value, new_value, change_reason)
        SELECT NEW.id, NEW.id, 'display_name', OLD.display_name, NEW.display_name, 'User profile update'
        WHERE OLD.display_name != NEW.display_name OR (OLD.display_name IS NULL AND NEW.display_name IS NOT NULL) OR (OLD.display_name IS NOT NULL AND NEW.display_name IS NULL);
        
        -- 记录角色变更
        INSERT INTO user_profile_changes (user_id, changed_by_user_id, field_name, old_value, new_value, change_reason)
        SELECT NEW.id, NEW.id, 'role', OLD.role, NEW.role, 'Role change'
        WHERE OLD.role != NEW.role;
        
        -- 记录激活状态变更
        INSERT INTO user_profile_changes (user_id, changed_by_user_id, field_name, old_value, new_value, change_reason)
        SELECT NEW.id, NEW.id, 'is_active', CAST(OLD.is_active AS TEXT), CAST(NEW.is_active AS TEXT), 'Status change'
        WHERE OLD.is_active != NEW.is_active;
        
        -- 记录验证状态变更
        INSERT INTO user_profile_changes (user_id, changed_by_user_id, field_name, old_value, new_value, change_reason)
        SELECT NEW.id, NEW.id, 'is_verified', CAST(OLD.is_verified AS TEXT), CAST(NEW.is_verified AS TEXT), 'Verification change'
        WHERE OLD.is_verified != NEW.is_verified;
    END;
```

## 视图设计

### 1. 插件统计视图

创建插件统计信息的视图：

```sql
CREATE VIEW plugin_statistics AS
SELECT 
    p.id,
    p.name,
    p.author,
    p.current_version,
    p.downloads,
    p.rating,
    p.status,
    p.created_at,
    COUNT(DISTINCT pr.id) as total_ratings,
    COUNT(DISTINCT pv.id) as total_versions,
    MAX(pv.created_at) as last_version_date,
    GROUP_CONCAT(DISTINCT pt.tag) as tags
FROM plugins p
LEFT JOIN plugin_ratings pr ON p.id = pr.plugin_id
LEFT JOIN plugin_versions pv ON p.id = pv.plugin_id
LEFT JOIN plugin_tags pt ON p.id = pt.plugin_id
GROUP BY p.id, p.name, p.author, p.current_version, p.downloads, p.rating, p.status, p.created_at;
```

### 2. 用户活动统计视图

创建用户活动统计视图：

```sql
CREATE VIEW user_activity_stats AS
SELECT 
    u.id,
    u.username,
    u.email,
    u.role,
    u.is_active,
    u.created_at,
    COUNT(DISTINCT p.id) as plugins_uploaded,
    COUNT(DISTINCT pr.id) as ratings_given,
    COALESCE(SUM(p.downloads), 0) as total_downloads_received,
    MAX(ula.login_time) as last_login_at,
    COUNT(DISTINCT ula.id) as total_logins
FROM users u
LEFT JOIN plugins p ON u.username = p.author
LEFT JOIN plugin_ratings pr ON u.id = pr.user_id
LEFT JOIN user_login_activities ula ON u.id = ula.user_id AND ula.is_successful = 1
GROUP BY u.id, u.username, u.email, u.role, u.is_active, u.created_at;
```

### 3. 热门插件视图

创建热门插件排行视图：

```sql
CREATE VIEW popular_plugins AS
SELECT 
    p.*,
    COALESCE(weekly_downloads.count, 0) as weekly_downloads,
    COALESCE(monthly_downloads.count, 0) as monthly_downloads,
    (p.downloads * 0.4 + p.rating * 0.3 + COALESCE(weekly_downloads.count, 0) * 0.3) as popularity_score
FROM plugins p
LEFT JOIN (
    SELECT plugin_id, COUNT(*) as count
    FROM plugin_versions pv
    WHERE pv.created_at >= date('now', '-7 days')
    GROUP BY plugin_id
) weekly_downloads ON p.id = weekly_downloads.plugin_id
LEFT JOIN (
    SELECT plugin_id, COUNT(*) as count  
    FROM plugin_versions pv
    WHERE pv.created_at >= date('now', '-30 days')
    GROUP BY plugin_id
) monthly_downloads ON p.id = monthly_downloads.plugin_id
WHERE p.status = 'active'
ORDER BY popularity_score DESC;
```

## 数据库安全

### 1. 启用外键约束

```sql
-- 启用外键约束（每次连接时执行）
PRAGMA foreign_keys = ON;

-- 检查外键约束状态
PRAGMA foreign_key_check;
```

### 2. 数据库连接安全

```rust
// Rust SQLx SQLite连接配置
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use std::str::FromStr;

let options = SqliteConnectOptions::from_str("sqlite://./data/marketplace.db")?
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal)  // WAL模式
    .synchronous(SqliteSynchronous::Normal) // 同步模式
    .foreign_keys(true)                     // 启用外键
    .pragma("cache_size", "-64000")         // 64MB缓存
    .pragma("temp_store", "memory")         // 临时存储在内存
    .pragma("mmap_size", "268435456");      // 256MB内存映射

let pool = SqlitePool::connect_with(options).await?;
```

### 3. 数据加密

对于敏感数据的加密处理：

```sql
-- SQLite不支持内置加密，建议在应用层处理敏感数据加密
-- 或使用SQLCipher扩展提供数据库级别的加密
```

## 备份和恢复

### 1. SQLite备份策略

```bash
#!/bin/bash
# SQLite数据库备份脚本

BACKUP_DIR="/var/backups/marketplace"
DATE=$(date +%Y%m%d_%H%M%S)
DB_FILE="./data/marketplace.db"

# 创建备份目录
mkdir -p $BACKUP_DIR

# 使用SQLite的.backup命令进行在线备份
sqlite3 $DB_FILE ".backup $BACKUP_DIR/marketplace_backup_$DATE.db"

# 压缩备份文件
gzip "$BACKUP_DIR/marketplace_backup_$DATE.db"

# 验证备份文件完整性
sqlite3 "$BACKUP_DIR/marketplace_backup_$DATE.db.gz" "PRAGMA integrity_check;" || echo "备份文件可能损坏"

# 删除7天前的备份
find $BACKUP_DIR -name "marketplace_backup_*.db.gz" -mtime +7 -delete

# 备份到远程存储（可选）
# aws s3 cp "$BACKUP_DIR/marketplace_backup_$DATE.db.gz" s3://backup-bucket/
```

### 2. 恢复操作

```bash
#!/bin/bash
# SQLite数据库恢复脚本

BACKUP_FILE=$1
DB_FILE="./data/marketplace.db"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file>"
    exit 1
fi

# 停止应用服务
systemctl stop marketplace

# 备份当前数据库
cp "$DB_FILE" "${DB_FILE}.recovery_backup"

# 解压并恢复数据
if [[ $BACKUP_FILE == *.gz ]]; then
    gunzip -c "$BACKUP_FILE" > "$DB_FILE"
else
    cp "$BACKUP_FILE" "$DB_FILE"
fi

# 检查数据库完整性
sqlite3 "$DB_FILE" "PRAGMA integrity_check;"
if [ $? -ne 0 ]; then
    echo "恢复的数据库文件损坏，回滚到原文件"
    mv "${DB_FILE}.recovery_backup" "$DB_FILE"
    exit 1
fi

# 优化数据库
sqlite3 "$DB_FILE" "VACUUM; ANALYZE;"

# 启动应用服务
systemctl start marketplace

echo "数据库恢复完成"
```

## 性能优化

### 1. 查询优化索引

```sql
-- 创建复合索引优化常用查询
CREATE INDEX idx_plugins_status_rating_downloads ON plugins(status, rating DESC, downloads DESC);
CREATE INDEX idx_plugin_ratings_plugin_rating ON plugin_ratings(plugin_id, rating);
CREATE INDEX idx_user_login_activities_composite ON user_login_activities(user_id, login_time DESC, is_successful);

-- 创建部分索引（SQLite 3.8.0+）
CREATE INDEX idx_active_plugins ON plugins(created_at DESC) WHERE status = 'active';
CREATE INDEX idx_failed_logins ON user_login_activities(ip_address, login_time) WHERE is_successful = 0;

-- 创建表达式索引
CREATE INDEX idx_plugins_lower_name ON plugins(LOWER(name));
CREATE INDEX idx_users_lower_email ON users(LOWER(email));
```

### 2. 数据库配置优化

```sql
-- SQLite性能优化PRAGMA设置
PRAGMA journal_mode = WAL;           -- WAL模式，提高并发性能
PRAGMA synchronous = NORMAL;         -- 平衡安全性和性能
PRAGMA cache_size = -64000;          -- 64MB缓存
PRAGMA temp_store = memory;          -- 临时数据存储在内存
PRAGMA mmap_size = 268435456;        -- 256MB内存映射
PRAGMA optimize;                     -- 自动优化

-- 定期执行的维护命令
PRAGMA incremental_vacuum;           -- 增量清理
ANALYZE;                            -- 更新查询优化器统计信息
```

### 3. 连接池配置

```rust
// Rust SQLx SQLite连接池配置
use sqlx::sqlite::SqlitePoolOptions;
use std::time::Duration;

let pool = SqlitePoolOptions::new()
    .max_connections(10)              // SQLite建议连接数不要太多
    .min_connections(1)               // 最小连接数
    .acquire_timeout(Duration::from_secs(30))  // 获取连接超时
    .idle_timeout(Duration::from_secs(600))    // 空闲连接超时
    .max_lifetime(Duration::from_secs(1800))   // 连接最大生命周期
    .test_before_acquire(true)        // 获取前测试连接
    .connect_with(options)
    .await?;
```

## 监控和维护

### 1. 数据库状态监控

```sql
-- 监控数据库大小
SELECT 
    name,
    (page_count * page_size) / 1024 / 1024 as size_mb
FROM pragma_database_list() d, pragma_page_count(d.name) p, pragma_page_size(d.name) s;

-- 监控表大小和统计信息
SELECT 
    name as table_name,
    rootpage,
    sql
FROM sqlite_master 
WHERE type = 'table'
ORDER BY name;

-- 监控索引使用情况
SELECT 
    name as index_name,
    tbl_name as table_name,
    sql
FROM sqlite_master 
WHERE type = 'index' AND sql IS NOT NULL
ORDER BY tbl_name, name;

-- 检查数据库完整性
PRAGMA integrity_check;

-- 检查外键完整性
PRAGMA foreign_key_check;

-- 查看数据库统计信息
PRAGMA table_info(users);
PRAGMA index_list(users);
PRAGMA index_info(idx_users_email);
```

### 2. 定期维护任务

```sql
-- SQLite维护脚本
-- 1. 更新查询优化器统计信息
ANALYZE;

-- 2. 优化数据库文件（重建数据库，回收空间）
VACUUM;

-- 3. 增量清理未使用空间
PRAGMA incremental_vacuum;

-- 4. 清理过期数据
DELETE FROM user_login_activities 
WHERE created_at < date('now', '-90 days');

-- 5. 清理临时文件记录
DELETE FROM plugin_versions 
WHERE file_path LIKE '%/temp/%' 
  AND created_at < date('now', '-1 day');

-- 6. 重新计算插件统计
UPDATE plugins SET downloads = (
    SELECT COALESCE(SUM(downloads), 0)
    FROM plugin_versions 
    WHERE plugin_id = plugins.id
);

-- 7. 优化查询执行计划
PRAGMA optimize;
```

### 3. 性能监控查询

```sql
-- 检查慢查询（需要启用统计扩展）
-- SQLite没有内置的慢查询日志，建议在应用层实现

-- 检查表的行数统计
SELECT 
    'users' as table_name, COUNT(*) as row_count FROM users
UNION ALL
SELECT 
    'plugins' as table_name, COUNT(*) as row_count FROM plugins
UNION ALL
SELECT 
    'plugin_ratings' as table_name, COUNT(*) as row_count FROM plugin_ratings
UNION ALL
SELECT 
    'user_login_activities' as table_name, COUNT(*) as row_count FROM user_login_activities;

-- 检查数据库配置
PRAGMA compile_options;
PRAGMA database_list;
PRAGMA journal_mode;
PRAGMA synchronous;
PRAGMA cache_size;
PRAGMA temp_store;
```

## 初始化脚本

```sql
-- 数据库初始化脚本
-- 1. 启用外键约束
PRAGMA foreign_keys = ON;

-- 2. 设置性能参数
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;
PRAGMA temp_store = memory;
PRAGMA mmap_size = 268435456;

-- 3. 创建所有表（按依赖顺序）
-- 见上述表创建语句

-- 4. 创建所有索引
-- 见上述索引创建语句

-- 5. 创建所有触发器
-- 见上述触发器创建语句

-- 6. 创建所有视图
-- 见上述视图创建语句

-- 7. 插入初始数据
INSERT INTO users (username, email, password_hash, display_name, role, is_active, is_verified)
VALUES ('admin', 'admin@geektools.com', '$2b$12$...', 'Administrator', 'superadmin', 1, 1);

-- 8. 优化数据库
ANALYZE;
PRAGMA optimize;
```

这个SQLite数据库设计文档提供了完整的数据库架构说明，包括表结构、索引设计、触发器、视图、安全策略、性能优化和维护策略。所有设计都基于SQLite的特性和限制，确保系统的稳定性、安全性和可扩展性。与PostgreSQL相比，SQLite的设计更加轻量级，适合中小型应用和嵌入式场景。