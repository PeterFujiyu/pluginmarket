# 数据库模式设计

## 概述

GeekTools 插件市场使用 PostgreSQL 作为主数据库，采用关系型数据库设计模式。数据库结构支持用户管理、插件管理、评分系统、管理后台等核心功能，并提供完整的审计日志和性能优化。

## 数据库配置

### 基础信息

- **数据库类型**: PostgreSQL 14+
- **字符集**: UTF-8
- **时区**: UTC
- **连接池**: 最大20个连接
- **事务隔离级别**: READ COMMITTED

### 连接配置

```env
DATABASE_URL=postgres://username:password@localhost:5432/marketplace
DATABASE_MAX_CONNECTIONS=20
```

## 核心数据表

### 1. 用户表 (users)

存储用户基本信息和认证数据。

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    role VARCHAR(20) DEFAULT 'user',
    is_active BOOLEAN DEFAULT true,
    is_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | SERIAL | PRIMARY KEY | 用户唯一标识 |
| username | VARCHAR(100) | UNIQUE, NOT NULL | 用户名，3-100字符 |
| email | VARCHAR(255) | UNIQUE, NOT NULL | 邮箱地址 |
| password_hash | VARCHAR(255) | NOT NULL | bcrypt加密后的密码 |
| display_name | VARCHAR(255) | NULL | 显示名称 |
| role | VARCHAR(20) | DEFAULT 'user' | 用户角色 |
| is_active | BOOLEAN | DEFAULT true | 账户是否激活 |
| is_verified | BOOLEAN | DEFAULT false | 邮箱是否验证 |
| created_at | TIMESTAMPTZ | DEFAULT NOW() | 创建时间 |
| updated_at | TIMESTAMPTZ | DEFAULT NOW() | 更新时间 |

**用户角色枚举**:
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
CREATE TYPE plugin_status AS ENUM ('active', 'deprecated', 'banned');

CREATE TABLE plugins (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    author VARCHAR(255) NOT NULL,
    current_version VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    downloads INTEGER DEFAULT 0,
    rating DECIMAL(3,2) DEFAULT 0.00,
    status plugin_status DEFAULT 'active',
    min_geektools_version VARCHAR(50),
    homepage_url VARCHAR(500),
    repository_url VARCHAR(500),
    license VARCHAR(100)
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | VARCHAR(255) | PRIMARY KEY | 插件唯一标识符 |
| name | VARCHAR(255) | NOT NULL | 插件名称 |
| description | TEXT | NULL | 插件描述 |
| author | VARCHAR(255) | NOT NULL | 作者名称 |
| current_version | VARCHAR(50) | NOT NULL | 当前版本号 |
| downloads | INTEGER | DEFAULT 0 | 总下载次数 |
| rating | DECIMAL(3,2) | DEFAULT 0.00 | 平均评分 (0.00-5.00) |
| status | plugin_status | DEFAULT 'active' | 插件状态 |
| min_geektools_version | VARCHAR(50) | NULL | 最低GeekTools版本要求 |
| homepage_url | VARCHAR(500) | NULL | 主页URL |
| repository_url | VARCHAR(500) | NULL | 代码仓库URL |
| license | VARCHAR(100) | NULL | 许可证类型 |

**插件状态枚举**:
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
    id SERIAL PRIMARY KEY,
    plugin_id VARCHAR(255) NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    changelog TEXT,
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    file_hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    downloads INTEGER DEFAULT 0,
    is_stable BOOLEAN DEFAULT true,
    UNIQUE(plugin_id, version)
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | SERIAL | PRIMARY KEY | 版本记录ID |
| plugin_id | VARCHAR(255) | FOREIGN KEY | 关联插件ID |
| version | VARCHAR(50) | NOT NULL | 版本号 |
| changelog | TEXT | NULL | 版本更新日志 |
| file_path | VARCHAR(500) | NOT NULL | 文件存储路径 |
| file_size | BIGINT | NOT NULL | 文件大小（字节） |
| file_hash | VARCHAR(64) | NOT NULL | 文件SHA256哈希 |
| downloads | INTEGER | DEFAULT 0 | 该版本下载次数 |
| is_stable | BOOLEAN | DEFAULT true | 是否为稳定版本 |

**约束设计**:
```sql
-- 复合唯一约束：一个插件的版本号唯一
ALTER TABLE plugin_versions ADD CONSTRAINT uk_plugin_version UNIQUE(plugin_id, version);

-- 检查约束：文件大小限制
ALTER TABLE plugin_versions ADD CONSTRAINT ck_file_size CHECK(file_size > 0 AND file_size <= 104857600); -- 100MB

-- 索引设计
CREATE INDEX idx_plugin_versions_plugin_id ON plugin_versions(plugin_id);
CREATE INDEX idx_plugin_versions_created_at ON plugin_versions(created_at DESC);
CREATE INDEX idx_plugin_versions_is_stable ON plugin_versions(is_stable);
```

### 4. 插件脚本表 (plugin_scripts)

存储插件包内的脚本文件信息。

```sql
CREATE TABLE plugin_scripts (
    id SERIAL PRIMARY KEY,
    plugin_id VARCHAR(255) NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    script_name VARCHAR(255) NOT NULL,
    script_file VARCHAR(255) NOT NULL,
    description TEXT,
    is_executable BOOLEAN DEFAULT false,
    FOREIGN KEY (plugin_id, version) REFERENCES plugin_versions(plugin_id, version) ON DELETE CASCADE
);
```

### 5. 插件依赖表 (plugin_dependencies)

存储插件之间的依赖关系。

```sql
CREATE TABLE plugin_dependencies (
    id SERIAL PRIMARY KEY,
    plugin_id VARCHAR(255) NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    dependency_id VARCHAR(255) NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    min_version VARCHAR(50),
    UNIQUE(plugin_id, dependency_id)
);
```

**约束设计**:
```sql
-- 防止自依赖
ALTER TABLE plugin_dependencies ADD CONSTRAINT ck_no_self_dependency 
CHECK (plugin_id != dependency_id);

-- 索引设计
CREATE INDEX idx_plugin_dependencies_plugin_id ON plugin_dependencies(plugin_id);
CREATE INDEX idx_plugin_dependencies_dependency_id ON plugin_dependencies(dependency_id);
```

### 6. 插件标签表 (plugin_tags)

存储插件的标签信息，支持多标签分类。

```sql
CREATE TABLE plugin_tags (
    id SERIAL PRIMARY KEY,
    plugin_id VARCHAR(255) NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    tag VARCHAR(100) NOT NULL,
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
    id SERIAL PRIMARY KEY,
    plugin_id VARCHAR(255) NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, plugin_id)
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | SERIAL | PRIMARY KEY | 评分记录ID |
| plugin_id | VARCHAR(255) | FOREIGN KEY | 关联插件ID |
| user_id | INTEGER | FOREIGN KEY | 评分用户ID |
| rating | INTEGER | CHECK(1-5) | 评分 (1-5星) |
| review | TEXT | NULL | 评论内容 |

**约束设计**:
```sql
-- 用户对同一插件只能评分一次
ALTER TABLE plugin_ratings ADD CONSTRAINT uk_user_plugin_rating UNIQUE(user_id, plugin_id);

-- 评分范围检查
ALTER TABLE plugin_ratings ADD CONSTRAINT ck_rating_range CHECK(rating >= 1 AND rating <= 5);

-- 索引设计
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
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    ip_address INET,
    user_agent TEXT,
    login_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    logout_time TIMESTAMPTZ,
    session_duration INTEGER, -- in seconds
    login_method VARCHAR(50) DEFAULT 'email_verification',
    is_successful BOOLEAN DEFAULT true,
    failure_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | SERIAL | PRIMARY KEY | 活动记录ID |
| user_id | INTEGER | FOREIGN KEY | 用户ID |
| email | VARCHAR(255) | NOT NULL | 登录邮箱 |
| ip_address | INET | NULL | 客户端IP地址 |
| user_agent | TEXT | NULL | 用户代理字符串 |
| login_time | TIMESTAMPTZ | NOT NULL | 登录时间 |
| logout_time | TIMESTAMPTZ | NULL | 登出时间 |
| session_duration | INTEGER | NULL | 会话持续时间（秒） |
| login_method | VARCHAR(50) | DEFAULT | 登录方式 |
| is_successful | BOOLEAN | DEFAULT true | 登录是否成功 |
| failure_reason | TEXT | NULL | 失败原因 |

**登录方式枚举**:
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

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | SERIAL | PRIMARY KEY | 日志记录ID |
| admin_user_id | INTEGER | FOREIGN KEY | 管理员用户ID |
| admin_email | VARCHAR(255) | NOT NULL | 管理员邮箱 |
| sql_query | TEXT | NOT NULL | 执行的SQL语句 |
| execution_time_ms | INTEGER | NULL | 执行时间（毫秒） |
| rows_affected | INTEGER | NULL | 影响行数 |
| is_successful | BOOLEAN | DEFAULT true | 执行是否成功 |
| error_message | TEXT | NULL | 错误信息 |
| ip_address | INET | NULL | 执行者IP地址 |

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
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    changed_by_user_id INTEGER NOT NULL REFERENCES users(id),
    field_name VARCHAR(50) NOT NULL,
    old_value TEXT,
    new_value TEXT,
    change_reason TEXT,
    ip_address INET,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**字段说明**:

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | SERIAL | PRIMARY KEY | 变更记录ID |
| user_id | INTEGER | FOREIGN KEY | 被修改的用户ID |
| changed_by_user_id | INTEGER | FOREIGN KEY | 执行修改的用户ID |
| field_name | VARCHAR(50) | NOT NULL | 修改的字段名 |
| old_value | TEXT | NULL | 原值 |
| new_value | TEXT | NULL | 新值 |
| change_reason | TEXT | NULL | 修改原因 |
| ip_address | INET | NULL | 修改者IP地址 |

**可变更字段枚举**:
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

## 数据库函数和触发器

### 1. 自动更新时间戳

创建函数自动更新 `updated_at` 字段：

```sql
-- 创建更新时间戳函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 为用户表创建触发器
CREATE TRIGGER update_users_updated_at 
    BEFORE UPDATE ON users 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 为插件表创建触发器
CREATE TRIGGER update_plugins_updated_at 
    BEFORE UPDATE ON plugins 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 为评分表创建触发器
CREATE TRIGGER update_plugin_ratings_updated_at 
    BEFORE UPDATE ON plugin_ratings 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

### 2. 计算插件平均评分

创建函数自动计算和更新插件平均评分：

```sql
-- 创建计算平均评分函数
CREATE OR REPLACE FUNCTION calculate_plugin_rating(plugin_id_param VARCHAR(255))
RETURNS DECIMAL(3,2) AS $$
DECLARE
    avg_rating DECIMAL(3,2);
BEGIN
    SELECT ROUND(AVG(rating), 2) INTO avg_rating
    FROM plugin_ratings 
    WHERE plugin_id = plugin_id_param;
    
    -- 更新插件表中的评分
    UPDATE plugins 
    SET rating = COALESCE(avg_rating, 0.00) 
    WHERE id = plugin_id_param;
    
    RETURN COALESCE(avg_rating, 0.00);
END;
$$ LANGUAGE plpgsql;

-- 创建评分变更触发器
CREATE OR REPLACE FUNCTION update_plugin_rating_trigger()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
        PERFORM calculate_plugin_rating(NEW.plugin_id);
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        PERFORM calculate_plugin_rating(OLD.plugin_id);
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- 为评分表创建触发器
CREATE TRIGGER plugin_rating_update_trigger
    AFTER INSERT OR UPDATE OR DELETE ON plugin_ratings
    FOR EACH ROW EXECUTE FUNCTION update_plugin_rating_trigger();
```

### 3. 记录用户资料变更

创建触发器自动记录用户资料变更：

```sql
-- 创建用户变更记录函数
CREATE OR REPLACE FUNCTION log_user_profile_changes()
RETURNS TRIGGER AS $$
BEGIN
    -- 检查邮箱变更
    IF OLD.email != NEW.email THEN
        INSERT INTO user_profile_changes (user_id, changed_by_user_id, field_name, old_value, new_value, change_reason)
        VALUES (NEW.id, NEW.id, 'email', OLD.email, NEW.email, 'User profile update');
    END IF;
    
    -- 检查用户名变更
    IF OLD.username != NEW.username THEN
        INSERT INTO user_profile_changes (user_id, changed_by_user_id, field_name, old_value, new_value, change_reason)
        VALUES (NEW.id, NEW.id, 'username', OLD.username, NEW.username, 'User profile update');
    END IF;
    
    -- 检查显示名称变更
    IF OLD.display_name IS DISTINCT FROM NEW.display_name THEN
        INSERT INTO user_profile_changes (user_id, changed_by_user_id, field_name, old_value, new_value, change_reason)
        VALUES (NEW.id, NEW.id, 'display_name', OLD.display_name, NEW.display_name, 'User profile update');
    END IF;
    
    -- 检查角色变更
    IF OLD.role != NEW.role THEN
        INSERT INTO user_profile_changes (user_id, changed_by_user_id, field_name, old_value, new_value, change_reason)
        VALUES (NEW.id, NEW.id, 'role', OLD.role, NEW.role, 'Role change');
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建用户变更触发器
CREATE TRIGGER user_profile_changes_trigger
    AFTER UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION log_user_profile_changes();
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
    ARRAY_AGG(DISTINCT pt.tag) FILTER (WHERE pt.tag IS NOT NULL) as tags
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
LEFT JOIN plugins p ON u.username = p.author  -- 假设author字段存储用户名
LEFT JOIN plugin_ratings pr ON u.id = pr.user_id
LEFT JOIN user_login_activities ula ON u.id = ula.user_id AND ula.is_successful = true
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
    WHERE pv.created_at >= CURRENT_DATE - INTERVAL '7 days'
    GROUP BY plugin_id
) weekly_downloads ON p.id = weekly_downloads.plugin_id
LEFT JOIN (
    SELECT plugin_id, COUNT(*) as count  
    FROM plugin_versions pv
    WHERE pv.created_at >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY plugin_id
) monthly_downloads ON p.id = monthly_downloads.plugin_id
WHERE p.status = 'active'
ORDER BY popularity_score DESC;
```

## 数据库安全

### 1. 用户权限管理

```sql
-- 创建应用数据库用户
CREATE USER marketplace_app WITH PASSWORD 'secure_random_password';

-- 授予基本权限
GRANT CONNECT ON DATABASE marketplace TO marketplace_app;
GRANT USAGE ON SCHEMA public TO marketplace_app;

-- 授予表权限
GRANT SELECT, INSERT, UPDATE ON users TO marketplace_app;
GRANT SELECT, INSERT, UPDATE ON plugins TO marketplace_app;
GRANT SELECT, INSERT, UPDATE ON plugin_versions TO marketplace_app;
GRANT SELECT, INSERT, UPDATE ON plugin_ratings TO marketplace_app;
GRANT SELECT, INSERT, UPDATE ON plugin_tags TO marketplace_app;
GRANT SELECT, INSERT, UPDATE ON plugin_dependencies TO marketplace_app;
GRANT SELECT, INSERT, UPDATE ON plugin_scripts TO marketplace_app;
GRANT SELECT, INSERT, UPDATE ON user_login_activities TO marketplace_app;
GRANT SELECT, INSERT, UPDATE ON user_profile_changes TO marketplace_app;

-- 授予序列权限
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO marketplace_app;

-- 禁止危险操作
REVOKE DELETE ON plugins FROM marketplace_app;
REVOKE CREATE ON SCHEMA public FROM marketplace_app;
REVOKE ALL ON pg_catalog FROM marketplace_app;

-- 创建管理员用户（用于管理操作）
CREATE USER marketplace_admin WITH PASSWORD 'admin_secure_password';
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO marketplace_admin;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO marketplace_admin;
```

### 2. 行级安全策略 (RLS)

启用行级安全策略保护敏感数据：

```sql
-- 启用用户表的行级安全
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- 用户只能查看和修改自己的信息
CREATE POLICY user_isolation_policy ON users
    USING (id = current_setting('app.current_user_id')::integer);

-- 管理员可以查看所有用户
CREATE POLICY admin_all_users_policy ON users
    TO marketplace_admin
    USING (true);

-- 启用登录活动表的行级安全
ALTER TABLE user_login_activities ENABLE ROW LEVEL SECURITY;

-- 用户只能查看自己的登录活动
CREATE POLICY user_login_isolation_policy ON user_login_activities
    USING (user_id = current_setting('app.current_user_id')::integer);
```

### 3. 数据加密

对敏感字段进行加密：

```sql
-- 安装pgcrypto扩展
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- 创建加密函数
CREATE OR REPLACE FUNCTION encrypt_sensitive_data(data TEXT, key TEXT)
RETURNS TEXT AS $$
BEGIN
    RETURN encode(encrypt(data::bytea, key::bytea, 'aes'), 'base64');
END;
$$ LANGUAGE plpgsql;

-- 创建解密函数
CREATE OR REPLACE FUNCTION decrypt_sensitive_data(encrypted_data TEXT, key TEXT)
RETURNS TEXT AS $$
BEGIN
    RETURN convert_from(decrypt(decode(encrypted_data, 'base64'), key::bytea, 'aes'), 'UTF8');
END;
$$ LANGUAGE plpgsql;
```

## 备份和恢复

### 1. 定期备份策略

```bash
#!/bin/bash
# 数据库备份脚本

BACKUP_DIR="/var/backups/marketplace"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="marketplace"

# 创建备份目录
mkdir -p $BACKUP_DIR

# 全量备份
pg_dump -h localhost -U postgres -d $DB_NAME -f $BACKUP_DIR/marketplace_full_$DATE.sql

# 压缩备份文件
gzip $BACKUP_DIR/marketplace_full_$DATE.sql

# 删除7天前的备份
find $BACKUP_DIR -name "marketplace_full_*.sql.gz" -mtime +7 -delete

# 备份到远程存储（可选）
# aws s3 cp $BACKUP_DIR/marketplace_full_$DATE.sql.gz s3://backup-bucket/
```

### 2. 恢复操作

```bash
#!/bin/bash
# 数据库恢复脚本

BACKUP_FILE=$1
DB_NAME="marketplace"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file>"
    exit 1
fi

# 停止应用服务
systemctl stop marketplace

# 创建新数据库（如果需要）
createdb -h localhost -U postgres $DB_NAME

# 恢复数据
if [[ $BACKUP_FILE == *.gz ]]; then
    gunzip -c $BACKUP_FILE | psql -h localhost -U postgres -d $DB_NAME
else
    psql -h localhost -U postgres -d $DB_NAME -f $BACKUP_FILE
fi

# 重建索引
psql -h localhost -U postgres -d $DB_NAME -c "REINDEX DATABASE $DB_NAME;"

# 更新统计信息
psql -h localhost -U postgres -d $DB_NAME -c "ANALYZE;"

# 启动应用服务
systemctl start marketplace
```

## 性能优化

### 1. 查询优化

```sql
-- 创建复合索引优化常用查询
CREATE INDEX idx_plugins_status_rating_downloads ON plugins(status, rating DESC, downloads DESC);
CREATE INDEX idx_plugin_ratings_plugin_rating ON plugin_ratings(plugin_id, rating);
CREATE INDEX idx_user_login_activities_composite ON user_login_activities(user_id, login_time DESC, is_successful);

-- 创建部分索引
CREATE INDEX idx_active_plugins ON plugins(created_at DESC) WHERE status = 'active';
CREATE INDEX idx_failed_logins ON user_login_activities(ip_address, login_time) WHERE is_successful = false;

-- 创建表达式索引
CREATE INDEX idx_plugins_lower_name ON plugins(LOWER(name));
CREATE INDEX idx_users_lower_email ON users(LOWER(email));
```

### 2. 连接池配置

```rust
// Rust SQLx 连接池配置
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(20)           // 最大连接数
    .min_connections(5)            // 最小连接数
    .acquire_timeout(Duration::from_secs(30))  // 获取连接超时
    .idle_timeout(Duration::from_secs(600))    // 空闲连接超时
    .max_lifetime(Duration::from_secs(1800))   // 连接最大生命周期
    .connect(&database_url)
    .await?;
```

### 3. 数据库配置优化

```sql
-- PostgreSQL 配置优化
-- postgresql.conf

-- 内存设置
shared_buffers = 256MB              -- 共享缓冲区
effective_cache_size = 1GB          -- 有效缓存大小
work_mem = 4MB                      -- 工作内存

-- 连接设置  
max_connections = 100               -- 最大连接数
listen_addresses = '*'              -- 监听地址

-- 日志设置
log_statement = 'mod'               -- 记录修改语句
log_min_duration_statement = 1000   -- 记录慢查询
log_line_prefix = '%t [%p]: [%l-1] user=%u,db=%d,app=%a,client=%h '

-- 检查点设置
checkpoint_completion_target = 0.9  -- 检查点完成目标
wal_buffers = 16MB                  -- WAL缓冲区

-- 维护设置
autovacuum = on                     -- 启用自动清理
autovacuum_vacuum_scale_factor = 0.1
autovacuum_analyze_scale_factor = 0.05
```

## 监控和维护

### 1. 性能监控查询

```sql
-- 监控数据库大小
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- 监控索引使用情况
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan as index_scans,
    idx_tup_read as tuples_read,
    idx_tup_fetch as tuples_fetched
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;

-- 监控慢查询
SELECT 
    query,
    calls,
    total_time,
    mean_time,
    rows
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;

-- 监控表统计信息
SELECT 
    schemaname,
    tablename,
    n_tup_ins as inserts,
    n_tup_upd as updates,
    n_tup_del as deletes,
    n_live_tup as live_tuples,
    n_dead_tup as dead_tuples,
    last_vacuum,
    last_autovacuum,
    last_analyze,
    last_autoanalyze
FROM pg_stat_user_tables;
```

### 2. 定期维护任务

```sql
-- 每日维护脚本
-- 1. 更新表统计信息
ANALYZE;

-- 2. 重新索引频繁更新的表
REINDEX TABLE user_login_activities;
REINDEX TABLE plugin_ratings;

-- 3. 清理过期数据
DELETE FROM user_login_activities 
WHERE created_at < CURRENT_DATE - INTERVAL '90 days';

-- 4. 清理临时文件记录
DELETE FROM plugin_versions 
WHERE file_path LIKE '%/temp/%' 
  AND created_at < CURRENT_DATE - INTERVAL '1 day';

-- 5. 更新插件统计
UPDATE plugins SET downloads = (
    SELECT COALESCE(SUM(downloads), 0)
    FROM plugin_versions 
    WHERE plugin_id = plugins.id
);
```

这个数据库设计文档提供了完整的数据库架构说明，包括表结构、索引设计、安全策略、性能优化和维护策略。所有设计都基于实际的生产环境需求，确保系统的稳定性、安全性和可扩展性。