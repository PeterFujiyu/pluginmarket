# 数据库迁移管理

## 概述

GeekTools 插件市场使用 SQLx CLI 进行数据库迁移管理。迁移系统提供版本控制的数据库结构变更，确保开发、测试和生产环境的数据库一致性。

## 迁移工具

### SQLx CLI 安装

```bash
# 安装 SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres

# 验证安装
sqlx --version
```

### 环境配置

在项目根目录创建 `.env` 文件：

```env
# 数据库连接URL
DATABASE_URL=postgres://username:password@localhost:5432/marketplace

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
- 创建自定义枚举类型
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
-- 创建自定义类型
CREATE TYPE plugin_status AS ENUM ('active', 'deprecated', 'banned');

-- 创建核心表
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    is_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 创建性能索引
CREATE INDEX idx_plugins_status_downloads ON plugins(status, downloads DESC);
CREATE INDEX idx_plugins_created_at ON plugins(created_at DESC);
CREATE INDEX idx_plugins_rating ON plugins(rating DESC);

-- 创建自动更新触发器
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';
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
-- 添加用户角色
ALTER TABLE users ADD COLUMN IF NOT EXISTS role VARCHAR(20) DEFAULT 'user';
UPDATE users SET role = 'admin' WHERE id = 1; -- 设置第一个用户为管理员

-- 创建登录活动表
CREATE TABLE IF NOT EXISTS user_login_activities (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    ip_address INET,
    user_agent TEXT,
    login_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    logout_time TIMESTAMPTZ,
    session_duration INTEGER,
    login_method VARCHAR(50) DEFAULT 'email_verification',
    is_successful BOOLEAN DEFAULT true,
    failure_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建管理员操作日志表
CREATE TABLE IF NOT EXISTS admin_sql_logs (
    id SERIAL PRIMARY KEY,
    admin_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    admin_email VARCHAR(255) NOT NULL,
    sql_query TEXT NOT NULL,
    execution_time_ms INTEGER,
    rows_affected INTEGER,
    is_successful BOOLEAN DEFAULT true,
    error_message TEXT,
    ip_address INET,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
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
ALTER TABLE users ADD COLUMN phone VARCHAR(20);

-- ❌ 避免：添加非空字段（会影响现有数据）
-- ALTER TABLE users ADD COLUMN phone VARCHAR(20) NOT NULL;

-- ✅ 正确的方式：先添加可空字段，再设置默认值，最后设置非空
ALTER TABLE users ADD COLUMN phone VARCHAR(20);
UPDATE users SET phone = '' WHERE phone IS NULL;
ALTER TABLE users ALTER COLUMN phone SET NOT NULL;
```

**安全的结构变更**:
```sql
-- ✅ 安全的索引创建（不阻塞表）
CREATE INDEX CONCURRENTLY idx_users_email_lower ON users(LOWER(email));

-- ✅ 安全的字段类型扩展
ALTER TABLE users ALTER COLUMN username TYPE VARCHAR(150);

-- ❌ 避免危险操作
-- DROP TABLE old_table;  -- 可能导致数据丢失
-- ALTER TABLE users DROP COLUMN old_field;  -- 不可逆操作
```

### 2. 数据迁移策略

**大表数据迁移**:
```sql
-- 分批处理大量数据
DO $$
DECLARE
    batch_size INTEGER := 1000;
    processed INTEGER := 0;
    total_rows INTEGER;
BEGIN
    SELECT COUNT(*) INTO total_rows FROM large_table WHERE condition;
    
    WHILE processed < total_rows LOOP
        UPDATE large_table 
        SET new_field = calculate_value(old_field)
        WHERE id IN (
            SELECT id FROM large_table 
            WHERE condition AND new_field IS NULL
            LIMIT batch_size
        );
        
        processed := processed + batch_size;
        RAISE NOTICE 'Processed % of % rows', processed, total_rows;
        
        -- 防止长时间锁表
        COMMIT;
    END LOOP;
END $$;
```

**复杂数据转换**:
```sql
-- 示例：重构插件标签存储方式
-- 从逗号分隔字符串转换为关联表

-- 1. 首先创建新的标签表
CREATE TABLE plugin_tags_new (
    id SERIAL PRIMARY KEY,
    plugin_id VARCHAR(255) NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    tag VARCHAR(100) NOT NULL,
    UNIQUE(plugin_id, tag)
);

-- 2. 迁移数据
INSERT INTO plugin_tags_new (plugin_id, tag)
SELECT 
    p.id as plugin_id,
    TRIM(tag_part) as tag
FROM plugins p,
     LATERAL unnest(string_to_array(p.tags_string, ',')) as tag_part
WHERE p.tags_string IS NOT NULL AND p.tags_string != '';

-- 3. 验证数据完整性
DO $$
BEGIN
    IF (SELECT COUNT(*) FROM plugin_tags_new) < (SELECT COUNT(*) FROM plugins WHERE tags_string IS NOT NULL) THEN
        RAISE EXCEPTION 'Data migration validation failed: insufficient tags migrated';
    END IF;
END $$;

-- 4. 重命名表（在确认迁移成功后）
-- ALTER TABLE plugin_tags_old RENAME TO plugin_tags_backup;
-- ALTER TABLE plugin_tags_new RENAME TO plugin_tags;
```

### 3. 性能优化迁移

**索引管理**:
```sql
-- 创建性能优化索引迁移
-- migrations/20240201000001_add_search_indexes.sql

-- 复合索引优化常用查询
CREATE INDEX CONCURRENTLY idx_plugins_search_optimized 
ON plugins(status, rating DESC, downloads DESC) 
WHERE status = 'active';

-- 部分索引减少索引大小
CREATE INDEX CONCURRENTLY idx_failed_logins 
ON user_login_activities(ip_address, login_time) 
WHERE is_successful = false;

-- 表达式索引支持大小写不敏感搜索
CREATE INDEX CONCURRENTLY idx_plugins_name_lower 
ON plugins(LOWER(name));

-- GIN索引支持全文搜索
CREATE INDEX CONCURRENTLY idx_plugins_search_vector
ON plugins USING gin(to_tsvector('english', name || ' ' || COALESCE(description, '')));
```

## 环境管理

### 1. 开发环境

```bash
# 开发环境数据库设置
export DATABASE_URL="postgres://dev_user:dev_pass@localhost:5432/marketplace_dev"

# 快速重置开发数据库
sqlx database drop
sqlx database create
sqlx migrate run

# 加载测试数据
psql $DATABASE_URL -f test_data.sql
```

### 2. 测试环境

```bash
# 测试环境迁移脚本
#!/bin/bash
set -e

export DATABASE_URL="postgres://test_user:test_pass@localhost:5432/marketplace_test"

echo "Setting up test database..."
sqlx database reset -y

echo "Running migrations..."
sqlx migrate run

echo "Loading test fixtures..."
psql $DATABASE_URL -f fixtures/test_users.sql
psql $DATABASE_URL -f fixtures/test_plugins.sql

echo "Test database ready!"
```

### 3. 生产环境

```bash
# 生产环境迁移脚本
#!/bin/bash
set -e

# 生产环境安全检查
if [ "$ENVIRONMENT" != "production" ]; then
    echo "Error: This script is only for production environment"
    exit 1
fi

# 备份当前数据库
echo "Creating backup..."
pg_dump $DATABASE_URL > "backup_$(date +%Y%m%d_%H%M%S).sql"

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

echo "Production migration completed successfully!"
```

## 回滚策略

### 1. 自动回滚

某些迁移可以自动回滚：

```sql
-- migrations/20240201000002_add_user_settings.sql
-- 添加用户设置表

CREATE TABLE user_settings (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    setting_key VARCHAR(100) NOT NULL,
    setting_value TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, setting_key)
);

CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);
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
-- 手动回滚用户设置表

-- 记录回滚操作
INSERT INTO migration_rollbacks (migration_name, rollback_time, rollback_reason)
VALUES ('20240201000002_add_user_settings', NOW(), 'Feature rollback');

-- 备份数据（如果需要）
CREATE TABLE user_settings_backup AS SELECT * FROM user_settings;

-- 删除索引
DROP INDEX IF EXISTS idx_user_settings_user_id;

-- 删除表
DROP TABLE IF EXISTS user_settings;
```

### 3. 数据恢复

重大问题的完整恢复：

```bash
#!/bin/bash
# 紧急数据库恢复脚本

BACKUP_FILE=$1
RECOVERY_POINT=$2

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file> [recovery_point]"
    exit 1
fi

# 停止应用服务
echo "Stopping application services..."
systemctl stop marketplace
systemctl stop nginx

# 恢复数据库
echo "Restoring database from backup..."
psql $DATABASE_URL -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'marketplace' AND pid <> pg_backend_pid();"
dropdb marketplace
createdb marketplace

if [[ $BACKUP_FILE == *.gz ]]; then
    gunzip -c $BACKUP_FILE | psql $DATABASE_URL
else
    psql $DATABASE_URL -f $BACKUP_FILE
fi

# 如果指定了恢复点，运行到指定迁移
if [ -n "$RECOVERY_POINT" ]; then
    echo "Rolling back to migration: $RECOVERY_POINT"
    # 这需要自定义逻辑来恢复到特定迁移点
fi

# 验证数据完整性
echo "Verifying data integrity..."
psql $DATABASE_URL -c "SELECT COUNT(*) FROM users;" > /dev/null
psql $DATABASE_URL -c "SELECT COUNT(*) FROM plugins;" > /dev/null

# 重启服务
echo "Restarting services..."
systemctl start marketplace
systemctl start nginx

echo "Database recovery completed!"
```

## 迁移监控和日志

### 1. 迁移日志记录

创建迁移操作日志表：

```sql
-- migrations/20240201000003_add_migration_logs.sql
CREATE TABLE migration_logs (
    id SERIAL PRIMARY KEY,
    migration_name VARCHAR(255) NOT NULL,
    operation_type VARCHAR(20) NOT NULL, -- 'apply' or 'revert'
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    success BOOLEAN DEFAULT false,
    error_message TEXT,
    executed_by VARCHAR(100),
    environment VARCHAR(50),
    database_version VARCHAR(50)
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
-- 监控长时间运行的迁移
SELECT 
    pid,
    now() - pg_stat_activity.query_start AS duration,
    query
FROM pg_stat_activity
WHERE state = 'active'
  AND query LIKE '%ALTER TABLE%'
  OR query LIKE '%CREATE INDEX%'
ORDER BY duration DESC;

-- 监控锁等待
SELECT 
    blocked_locks.pid AS blocked_pid,
    blocked_activity.usename AS blocked_user,
    blocking_locks.pid AS blocking_pid,
    blocking_activity.usename AS blocking_user,
    blocked_activity.query AS blocked_statement,
    blocking_activity.query AS current_statement_in_blocking_process
FROM pg_catalog.pg_locks blocked_locks
JOIN pg_catalog.pg_stat_activity blocked_activity 
    ON blocked_activity.pid = blocked_locks.pid
JOIN pg_catalog.pg_locks blocking_locks 
    ON blocking_locks.locktype = blocked_locks.locktype
    AND blocking_locks.DATABASE IS NOT DISTINCT FROM blocked_locks.DATABASE
    AND blocking_locks.relation IS NOT DISTINCT FROM blocked_locks.relation
    AND blocking_locks.page IS NOT DISTINCT FROM blocked_locks.page
    AND blocking_locks.tuple IS NOT DISTINCT FROM blocked_locks.tuple
    AND blocking_locks.virtualxid IS NOT DISTINCT FROM blocked_locks.virtualxid
    AND blocking_locks.transactionid IS NOT DISTINCT FROM blocked_locks.transactionid
    AND blocking_locks.classid IS NOT DISTINCT FROM blocked_locks.classid
    AND blocking_locks.objid IS NOT DISTINCT FROM blocked_locks.objid
    AND blocking_locks.objsubid IS NOT DISTINCT FROM blocked_locks.objsubid
    AND blocking_locks.pid != blocked_locks.pid
JOIN pg_catalog.pg_stat_activity blocking_activity 
    ON blocking_activity.pid = blocking_locks.pid
WHERE NOT blocked_locks.GRANTED;
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

### 2. 并发迁移冲突

```sql
-- 处理迁移锁表问题
-- 查看当前锁状态
SELECT * FROM pg_locks WHERE locktype = 'relation';

-- 终止阻塞的连接（小心使用）
SELECT pg_terminate_backend(pid) 
FROM pg_stat_activity 
WHERE state = 'idle in transaction' 
  AND query_start < now() - interval '1 hour';
```

### 3. 大表迁移优化

```sql
-- 对于大表的安全迁移策略
-- 示例：为大表添加索引

-- 1. 使用 CONCURRENTLY 避免锁表
CREATE INDEX CONCURRENTLY idx_large_table_new_field ON large_table(new_field);

-- 2. 分批处理数据更新
CREATE OR REPLACE FUNCTION batch_update_large_table()
RETURNS void AS $$
DECLARE
    batch_size INTEGER := 10000;
    updated_count INTEGER;
BEGIN
    LOOP
        UPDATE large_table 
        SET new_field = calculate_new_value(old_field)
        WHERE id IN (
            SELECT id FROM large_table 
            WHERE new_field IS NULL 
            LIMIT batch_size
        );
        
        GET DIAGNOSTICS updated_count = ROW_COUNT;
        EXIT WHEN updated_count = 0;
        
        RAISE NOTICE 'Updated % rows', updated_count;
        PERFORM pg_sleep(0.1); -- 短暂休息避免长时间锁定
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- 3. 执行批量更新
SELECT batch_update_large_table();
```

## 总结

数据库迁移管理是确保系统稳定性和数据一致性的关键组件。通过严格的迁移规范、完善的测试流程和详细的监控机制，可以安全地管理数据库结构的演进。

**关键要点**:
1. 始终先在开发环境测试迁移
2. 生产环境迁移前进行完整备份
3. 使用事务确保迁移的原子性
4. 监控迁移性能和锁等待
5. 准备回滚方案应对意外情况
6. 记录所有迁移操作的详细日志