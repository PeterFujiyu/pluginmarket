# 数据库迁移管理

## 概述

GeekTools 插件市场使用 SQLx CLI 进行数据库迁移管理。迁移系统提供版本控制的数据库结构变更，确保开发、测试和生产环境的数据库一致性。本项目已从 PostgreSQL 迁移到 SQLite，提供更轻量级的部署和维护体验。

## 迁移工具

### SQLx CLI 安装

```bash
# 安装 SQLx CLI (SQLite版本)
cargo install sqlx-cli --no-default-features --features sqlite

# 验证安装
sqlx --version
```

### 环境配置

在项目根目录创建 `.env` 文件：

```env
# SQLite数据库连接URL
DATABASE_URL=sqlite:./marketplace.db

# 迁移目录（可选，默认为 migrations/）
SQLX_MIGRATIONS_DIR=migrations
```

## 迁移文件结构

```
server/
├── migrations/
│   ├── 001_initial.sql                    # 初始数据库结构
│   ├── 20240127000004_add_admin_features.sql  # 管理功能
│   ├── 20240201000001_add_search_indexes.sql  # 搜索优化
│   └── ...
├── src/
└── Cargo.toml
```

## 现有迁移文件

### 1. 初始迁移 (001_initial.sql)

创建核心数据表和基础结构。

**迁移内容**:
- 创建用户表 (users)
- 创建插件表 (plugins) 
- 创建插件版本表 (plugin_versions)
- 创建插件脚本表 (plugin_scripts)
- 创建插件依赖表 (plugin_dependencies)
- 创建插件标签表 (plugin_tags)
- 创建插件评分表 (plugin_ratings)
- 创建性能优化索引
- 创建自动更新时间戳的触发器

**关键特性**:
```sql
-- 创建核心表 (SQLite语法)
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

-- 创建性能索引
CREATE INDEX idx_plugins_status_downloads ON plugins(status, downloads DESC);
CREATE INDEX idx_plugins_created_at ON plugins(created_at DESC);
CREATE INDEX idx_plugins_rating ON plugins(rating DESC);

-- 创建自动更新触发器 (SQLite语法)
CREATE TRIGGER update_users_timestamp 
    AFTER UPDATE ON users
BEGIN
    UPDATE users 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = NEW.id;
END;
```

### 2. 管理功能迁移 (20240127000004_add_admin_features.sql)

添加管理后台相关功能。

**迁移内容**:
- 为用户表添加角色字段
- 创建用户登录活动跟踪表
- 创建管理员SQL操作日志表
- 创建用户资料变更记录表
- 创建相关索引

**关键特性**:
```sql
-- 添加用户角色 (SQLite语法)
ALTER TABLE users ADD COLUMN role TEXT DEFAULT 'user';
UPDATE users SET role = 'admin' WHERE id = 1; -- 设置第一个用户为管理员

-- 创建登录活动表
CREATE TABLE IF NOT EXISTS user_login_activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    login_time DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    logout_time DATETIME,
    session_duration INTEGER,
    login_method TEXT DEFAULT 'email_verification',
    is_successful BOOLEAN DEFAULT 1,
    failure_reason TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建管理员操作日志表
CREATE TABLE IF NOT EXISTS admin_sql_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    admin_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    admin_email TEXT NOT NULL,
    sql_query TEXT NOT NULL,
    execution_time_ms INTEGER,
    rows_affected INTEGER,
    is_successful BOOLEAN DEFAULT 1,
    error_message TEXT,
    ip_address TEXT,
    executed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## 迁移命令

### 基本命令

```bash
# 进入服务器目录
cd server/

# 检查迁移状态
sqlx migrate info

# 运行迁移（应用所有未执行的迁移）
sqlx migrate run

# 恢复迁移（回滚一个迁移）
sqlx migrate revert

# 重置数据库（删除所有表并重新运行迁移）
sqlx database reset
```

### 创建新迁移

```bash
# 创建新的迁移文件
sqlx migrate add create_new_feature

# 这将创建一个新文件：migrations/YYYYMMDDHHMMSS_create_new_feature.sql
```

### 迁移文件命名规范

**格式**: `YYYYMMDDHHMMSS_description.sql`

**示例**:
- `20240201120000_add_user_preferences.sql`
- `20240201130000_create_notification_system.sql`
- `20240201140000_optimize_search_performance.sql`

## 迁移最佳实践

### 1. 迁移编写原则

**向前兼容**:
```sql
-- ✅ 好的做法：添加可空字段
ALTER TABLE users ADD COLUMN phone TEXT;

-- ❌ 避免：添加非空字段（会影响现有数据）
-- ALTER TABLE users ADD COLUMN phone TEXT NOT NULL;

-- ✅ 正确的方式：先添加可空字段，再设置默认值
-- 注意：SQLite 不支持直接添加 NOT NULL 约束到已有表
ALTER TABLE users ADD COLUMN phone TEXT DEFAULT '';
```

**安全的结构变更**:
```sql
-- ✅ 安全的索引创建
CREATE INDEX IF NOT EXISTS idx_users_email_lower ON users(LOWER(email));

-- ⚠️ SQLite 限制：不支持直接修改字段类型
-- 需要使用表重建策略：
-- 1. 创建新表结构
-- 2. 复制数据
-- 3. 删除旧表
-- 4. 重命名新表

-- ❌ 避免危险操作
-- DROP TABLE old_table;  -- 可能导致数据丢失
-- ALTER TABLE users DROP COLUMN old_field;  -- SQLite不支持删除列
```

### 2. 数据迁移策略

**大表数据迁移**:
```sql
-- SQLite 分批处理大量数据
-- 使用简单的批处理循环

-- 创建临时表记录进度
CREATE TEMP TABLE migration_progress (
    batch_number INTEGER,
    processed_count INTEGER,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 分批更新数据
UPDATE large_table 
SET new_field = calculate_value(old_field)
WHERE id IN (
    SELECT id FROM large_table 
    WHERE new_field IS NULL 
    LIMIT 1000  -- 批处理大小
);

-- 记录处理进度
INSERT INTO migration_progress (batch_number, processed_count)
VALUES (1, changes());

-- 继续处理剩余批次...
```

**复杂数据转换**:
```sql
-- 示例：重构插件标签存储方式 (SQLite版本)
-- 从逗号分隔字符串转换为关联表

-- 1. 首先创建新的标签表
CREATE TABLE plugin_tags_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    UNIQUE(plugin_id, tag)
);

-- 2. 创建递归 CTE 来分割标签字符串
WITH RECURSIVE tag_split(plugin_id, tag, remaining) AS (
    -- 基础情况：提取第一个标签
    SELECT 
        id as plugin_id,
        CASE 
            WHEN tags_string LIKE '%,%' THEN 
                TRIM(SUBSTR(tags_string, 1, INSTR(tags_string, ',') - 1))
            ELSE 
                TRIM(tags_string)
        END as tag,
        CASE 
            WHEN tags_string LIKE '%,%' THEN 
                TRIM(SUBSTR(tags_string, INSTR(tags_string, ',') + 1))
            ELSE 
                ''
        END as remaining
    FROM plugins 
    WHERE tags_string IS NOT NULL AND tags_string != ''
    
    UNION ALL
    
    -- 递归情况：继续分割剩余标签
    SELECT 
        plugin_id,
        CASE 
            WHEN remaining LIKE '%,%' THEN 
                TRIM(SUBSTR(remaining, 1, INSTR(remaining, ',') - 1))
            ELSE 
                TRIM(remaining)
        END,
        CASE 
            WHEN remaining LIKE '%,%' THEN 
                TRIM(SUBSTR(remaining, INSTR(remaining, ',') + 1))
            ELSE 
                ''
        END
    FROM tag_split
    WHERE remaining != ''
)

-- 3. 插入分割后的标签
INSERT INTO plugin_tags_new (plugin_id, tag)
SELECT DISTINCT plugin_id, tag 
FROM tag_split 
WHERE tag != '';

-- 4. 验证数据完整性
-- SQLite 简单验证
SELECT 
    (SELECT COUNT(DISTINCT id) FROM plugins WHERE tags_string IS NOT NULL) as source_count,
    (SELECT COUNT(DISTINCT plugin_id) FROM plugin_tags_new) as target_count;

-- 5. 重命名表（在确认迁移成功后）
-- ALTER TABLE plugin_tags RENAME TO plugin_tags_backup;
-- ALTER TABLE plugin_tags_new RENAME TO plugin_tags;
```

### 3. 性能优化迁移

**索引管理**:
```sql
-- 创建性能优化索引迁移 (SQLite版本)
-- migrations/20240201000001_add_search_indexes.sql

-- 复合索引优化常用查询
CREATE INDEX IF NOT EXISTS idx_plugins_search_optimized 
ON plugins(status, rating DESC, downloads DESC);

-- 条件索引 (SQLite 支持 WHERE 子句)
CREATE INDEX IF NOT EXISTS idx_failed_logins 
ON user_login_activities(ip_address, login_time) 
WHERE is_successful = 0;

-- 表达式索引支持大小写不敏感搜索
CREATE INDEX IF NOT EXISTS idx_plugins_name_lower 
ON plugins(LOWER(name));

-- SQLite FTS5 全文搜索索引
CREATE VIRTUAL TABLE IF NOT EXISTS plugins_fts USING fts5(
    name, 
    description, 
    content='plugins',
    content_rowid='id'
);

-- 同步数据到 FTS 表
INSERT INTO plugins_fts(name, description)
SELECT name, COALESCE(description, '') FROM plugins;

-- 创建触发器保持 FTS 同步
CREATE TRIGGER plugins_fts_insert AFTER INSERT ON plugins BEGIN
    INSERT INTO plugins_fts(rowid, name, description) 
    VALUES (new.id, new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER plugins_fts_update AFTER UPDATE ON plugins BEGIN
    UPDATE plugins_fts 
    SET name = new.name, description = COALESCE(new.description, '') 
    WHERE rowid = new.id;
END;

CREATE TRIGGER plugins_fts_delete AFTER DELETE ON plugins BEGIN
    DELETE FROM plugins_fts WHERE rowid = old.id;
END;
```

## 环境管理

### 1. 开发环境

```bash
# 开发环境数据库设置 (SQLite)
export DATABASE_URL="sqlite:./marketplace_dev.db"

# 快速重置开发数据库
rm -f ./marketplace_dev.db
sqlx database create
sqlx migrate run

# 加载测试数据 (使用 sqlite3 命令行工具)
sqlite3 ./marketplace_dev.db < test_data.sql
```

### 2. 测试环境

```bash
# 测试环境迁移脚本 (SQLite版本)
#!/bin/bash
set -e

export DATABASE_URL="sqlite:./marketplace_test.db"

echo "Setting up test database..."
rm -f ./marketplace_test.db
sqlx database create

echo "Running migrations..."
sqlx migrate run

echo "Loading test fixtures..."
sqlite3 ./marketplace_test.db < fixtures/test_users.sql
sqlite3 ./marketplace_test.db < fixtures/test_plugins.sql

echo "Test database ready!"
```

### 3. 生产环境

```bash
# 生产环境迁移脚本 (SQLite版本)
#!/bin/bash
set -e

# 生产环境安全检查
if [ "$ENVIRONMENT" != "production" ]; then
    echo "Error: This script is only for production environment"
    exit 1
fi

# SQLite 数据库文件路径
DB_FILE="./marketplace.db"

# 备份当前数据库
echo "Creating backup..."
cp "$DB_FILE" "backup_$(date +%Y%m%d_%H%M%S).db"

# 检查迁移状态
echo "Checking migration status..."
sqlx migrate info

# 确认迁移
read -p "Are you sure you want to run migrations in production? (yes/no): " confirm
if [ "$confirm" != "yes" ]; then
    echo "Migration cancelled"
    exit 1
fi

# 运行迁移
echo "Running migrations..."
sqlx migrate run

# 验证迁移结果
echo "Verifying migration..."
sqlx migrate info

# 验证数据库完整性
echo "Running integrity check..."
sqlite3 "$DB_FILE" "PRAGMA integrity_check;"

echo "Production migration completed successfully!"
```

## 回滚策略

### 1. 自动回滚

某些迁移可以自动回滚：

```sql
-- migrations/20240201000002_add_user_settings.sql
-- 添加用户设置表 (SQLite版本)

CREATE TABLE user_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    setting_key TEXT NOT NULL,
    setting_value TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, setting_key)
);

CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);

-- 添加自动更新时间戳触发器
CREATE TRIGGER update_user_settings_timestamp 
    AFTER UPDATE ON user_settings
BEGIN
    UPDATE user_settings 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = NEW.id;
END;
```

回滚命令：
```bash
# 回滚最后一个迁移
sqlx migrate revert
```

### 2. 手动回滚

复杂迁移需要手动编写回滚脚本：

```sql
-- rollback_scripts/revert_20240201000002_add_user_settings.sql
-- 手动回滚用户设置表 (SQLite版本)

-- 记录回滚操作
INSERT INTO migration_rollbacks (migration_name, rollback_time, rollback_reason)
VALUES ('20240201000002_add_user_settings', CURRENT_TIMESTAMP, 'Feature rollback');

-- 备份数据（如果需要）
CREATE TABLE user_settings_backup AS SELECT * FROM user_settings;

-- 删除触发器
DROP TRIGGER IF EXISTS update_user_settings_timestamp;

-- 删除索引
DROP INDEX IF EXISTS idx_user_settings_user_id;

-- 删除表
DROP TABLE IF EXISTS user_settings;
```

### 3. 数据恢复

重大问题的完整恢复 (SQLite版本)：

```bash
#!/bin/bash
# 紧急SQLite数据库恢复脚本

BACKUP_FILE=$1
RECOVERY_POINT=$2
DB_FILE="./marketplace.db"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file> [recovery_point]"
    exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
    echo "Error: Backup file '$BACKUP_FILE' not found"
    exit 1
fi

# 停止应用服务
echo "Stopping application services..."
systemctl stop marketplace || echo "marketplace service not running"
systemctl stop nginx || echo "nginx service not running"

# 创建当前状态的紧急备份
if [ -f "$DB_FILE" ]; then
    echo "Creating emergency backup of current database..."
    cp "$DB_FILE" "emergency_backup_$(date +%Y%m%d_%H%M%S).db"
fi

# 恢复数据库
echo "Restoring database from backup..."
cp "$BACKUP_FILE" "$DB_FILE"

# 如果指定了恢复点，运行到指定迁移
if [ -n "$RECOVERY_POINT" ]; then
    echo "Rolling back to migration: $RECOVERY_POINT"
    # SQLite 迁移回滚需要重新运行到指定点
    # 这需要自定义逻辑来恢复到特定迁移点
fi

# 验证数据完整性
echo "Verifying data integrity..."
sqlite3 "$DB_FILE" "PRAGMA integrity_check;" | grep -q "ok" || {
    echo "Error: Database integrity check failed"
    exit 1
}

# 验证关键表存在且有数据
sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM users;" > /dev/null || {
    echo "Error: Users table verification failed"
    exit 1
}

sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM plugins;" > /dev/null || {
    echo "Error: Plugins table verification failed" 
    exit 1
}

# 优化数据库
echo "Optimizing database..."
sqlite3 "$DB_FILE" "VACUUM; ANALYZE;"

# 重启服务
echo "Restarting services..."
systemctl start marketplace
systemctl start nginx

echo "Database recovery completed successfully!"
echo "Database file: $DB_FILE"
echo "Emergency backup created for rollback if needed"
```

## 迁移监控和日志

### 1. 迁移日志记录

创建迁移操作日志表：

```sql
-- migrations/20240201000003_add_migration_logs.sql
CREATE TABLE migration_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    migration_name TEXT NOT NULL,
    operation_type TEXT NOT NULL, -- 'apply' or 'revert'
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    success BOOLEAN DEFAULT 0,
    error_message TEXT,
    executed_by TEXT,
    environment TEXT,
    database_version TEXT
);

CREATE INDEX idx_migration_logs_migration_name ON migration_logs(migration_name);
CREATE INDEX idx_migration_logs_start_time ON migration_logs(start_time);
```

### 2. 迁移状态监控

```bash
#!/bin/bash
# 迁移状态监控脚本

# 检查待处理的迁移
PENDING_MIGRATIONS=$(sqlx migrate info | grep "pending" | wc -l)

if [ $PENDING_MIGRATIONS -gt 0 ]; then
    echo "WARNING: $PENDING_MIGRATIONS pending migrations found"
    sqlx migrate info
    exit 1
fi

# 检查数据库连接
if ! sqlx migrate info > /dev/null 2>&1; then
    echo "ERROR: Cannot connect to database"
    exit 1
fi

echo "All migrations are up to date"
```

### 3. 性能监控

```sql
-- SQLite 性能监控查询
-- 监控数据库文件大小和页面统计
PRAGMA page_count;
PRAGMA page_size;
PRAGMA freelist_count;

-- 检查索引使用情况
SELECT 
    name,
    tbl_name,
    sql
FROM sqlite_master 
WHERE type = 'index' 
  AND name NOT LIKE 'sqlite_%'
ORDER BY tbl_name, name;

-- 分析查询性能 (需要在查询前执行)
PRAGMA query_only = ON;
EXPLAIN QUERY PLAN SELECT * FROM plugins WHERE status = 'active';
PRAGMA query_only = OFF;

-- 统计表大小
SELECT 
    name as table_name,
    (
        SELECT COUNT(*) 
        FROM pragma_table_info(name)
    ) as column_count,
    (
        CASE name 
            WHEN 'users' THEN (SELECT COUNT(*) FROM users)
            WHEN 'plugins' THEN (SELECT COUNT(*) FROM plugins)
            WHEN 'plugin_ratings' THEN (SELECT COUNT(*) FROM plugin_ratings)
            ELSE 0
        END
    ) as row_count
FROM sqlite_master 
WHERE type = 'table' 
  AND name NOT LIKE 'sqlite_%'
ORDER BY name;
```

## 常见问题解决

### 1. 迁移失败处理

```bash
# 检查失败的迁移
sqlx migrate info

# 手动修复数据问题后重试
sqlx migrate run

# 如果迁移文件有问题，修复后强制重新运行
sqlx migrate revert  # 回滚失败的迁移
# 修复迁移文件
sqlx migrate run     # 重新运行
```

### 2. SQLite 锁定问题

```bash
# SQLite 数据库锁定问题处理
# SQLite 使用文件锁，通常不会有复杂的锁定冲突

# 检查数据库是否被锁定
sqlite3 ./marketplace.db "PRAGMA locking_mode;"

# 如果遇到 "database is locked" 错误：
# 1. 确保没有其他进程正在使用数据库
lsof ./marketplace.db

# 2. 检查是否存在 WAL 模式文件
ls -la ./marketplace.db*

# 3. 如果需要强制解锁（谨慎使用）
# 停止所有相关进程后删除锁文件
rm -f ./marketplace.db-shm ./marketplace.db-wal
```

### 3. SQLite 大表优化策略

```sql
-- SQLite 大表迁移优化策略

-- 1. 启用 WAL 模式提高并发性
PRAGMA journal_mode=WAL;

-- 2. 调整页面大小（适用于新数据库）
PRAGMA page_size = 4096;

-- 3. 分批处理大量数据更新
-- SQLite 批量更新示例
CREATE TEMP TABLE batch_tracking (
    batch_id INTEGER PRIMARY KEY,
    start_id INTEGER,
    end_id INTEGER,
    processed BOOLEAN DEFAULT 0
);

-- 创建批次
INSERT INTO batch_tracking (batch_id, start_id, end_id)
SELECT 
    (id - 1) / 1000 + 1 as batch_id,
    ((id - 1) / 1000) * 1000 + 1 as start_id,
    MIN(((id - 1) / 1000) * 1000 + 1000, MAX(id)) as end_id
FROM large_table
GROUP BY (id - 1) / 1000;

-- 分批处理
UPDATE large_table 
SET new_field = calculate_new_value(old_field)
WHERE id BETWEEN 1 AND 1000;  -- 第一批

-- 更新跟踪
UPDATE batch_tracking 
SET processed = 1 
WHERE batch_id = 1;

-- 4. 优化完成后运行 VACUUM
VACUUM;
ANALYZE;
```

## 总结

数据库迁移管理是确保系统稳定性和数据一致性的关键组件。SQLite 作为轻量级数据库，简化了部署和维护流程，同时提供了足够的功能来支持复杂的迁移需求。

**SQLite 迁移的关键要点**:
1. 始终先在开发环境测试迁移
2. 生产环境迁移前进行文件级备份（简单复制数据库文件）
3. 利用 SQLite 的事务特性确保迁移的原子性
4. 使用 WAL 模式提高并发性和性能
5. 理解 SQLite 的限制（如不支持删除列、修改列类型等）
6. 准备回滚方案和数据恢复脚本
7. 使用 PRAGMA 命令进行性能监控和优化
8. 定期运行 VACUUM 和 ANALYZE 维护数据库健康

**SQLite 相对 PostgreSQL 的优势**:
- **简单部署**: 无需独立数据库服务器
- **文件备份**: 简单的文件复制即可完成备份
- **零配置**: 无需复杂的用户权限和网络配置
- **高性能**: 对于中小型应用性能优异
- **可靠性**: 经过广泛验证的存储引擎

通过遵循本文档的最佳实践，可以安全有效地管理 SQLite 数据库的结构演进。