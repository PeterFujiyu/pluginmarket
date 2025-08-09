# SQLite数据库性能优化策略

## 概述

GeekTools 插件市场的SQLite数据库性能优化涵盖查询优化、索引设计、连接管理、缓存策略等多个方面。本文档提供全面的SQLite性能优化指南，确保系统在高并发和大数据量场景下的稳定运行。

## 查询性能优化

### 1. 索引设计策略

#### 主要索引类型

**普通索引**（B-tree索引）:
```sql
-- 单字段索引
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_plugins_status ON plugins(status);

-- 复合索引（注意字段顺序）
CREATE INDEX idx_plugins_status_rating_downloads 
ON plugins(status, rating DESC, downloads DESC);

-- 表达式索引
CREATE INDEX idx_users_email_lower ON users(LOWER(email));
CREATE INDEX idx_plugins_created_date ON plugins(DATE(created_at));
```

**部分索引**（WHERE条件索引）:
```sql
-- 只为活跃插件创建索引
CREATE INDEX idx_active_plugins_rating 
ON plugins(rating DESC) 
WHERE status = 'active';

-- 只为失败登录创建索引
CREATE INDEX idx_failed_logins 
ON user_login_activities(ip_address, login_time) 
WHERE is_successful = 0;

-- 只为最近数据创建索引
CREATE INDEX idx_recent_plugin_ratings 
ON plugin_ratings(created_at DESC) 
WHERE created_at >= datetime('now', '-30 days');
```

**FTS5全文搜索索引**:
```sql
-- 创建虚拟FTS5表用于全文搜索
CREATE VIRTUAL TABLE plugins_fts USING fts5(
    name, 
    description, 
    author, 
    tags,
    content=plugins,
    content_rowid=id
);

-- 填充FTS索引
INSERT INTO plugins_fts(rowid, name, description, author, tags)
SELECT id, name, COALESCE(description, ''), author, 
       COALESCE(group_concat(tag, ' '), '') as tags
FROM plugins p
LEFT JOIN plugin_tags pt ON p.id = pt.plugin_id
WHERE p.status = 'active'
GROUP BY p.id;

-- 创建触发器维护FTS索引
CREATE TRIGGER plugins_fts_insert AFTER INSERT ON plugins BEGIN
    INSERT INTO plugins_fts(rowid, name, description, author)
    VALUES (new.id, new.name, COALESCE(new.description, ''), new.author);
END;

CREATE TRIGGER plugins_fts_delete AFTER DELETE ON plugins BEGIN
    DELETE FROM plugins_fts WHERE rowid = old.id;
END;

CREATE TRIGGER plugins_fts_update AFTER UPDATE ON plugins BEGIN
    DELETE FROM plugins_fts WHERE rowid = old.id;
    INSERT INTO plugins_fts(rowid, name, description, author)
    VALUES (new.id, new.name, COALESCE(new.description, ''), new.author);
END;
```

#### 索引使用原则

**最左前缀原则**:
```sql
-- 复合索引：(status, rating, downloads)
-- 可以被以下查询使用：
-- WHERE status = 'active'  ✅
-- WHERE status = 'active' AND rating > 4.0  ✅
-- WHERE status = 'active' AND rating > 4.0 AND downloads > 100  ✅
-- WHERE rating > 4.0  ❌ (不能使用索引)
-- WHERE downloads > 100  ❌ (不能使用索引)
```

**选择性分析**:
```sql
-- 分析字段的选择性，高选择性字段适合创建索引
-- SQLite没有内置的selectivity统计，需要手动计算
WITH column_stats AS (
    SELECT 
        'status' as column_name,
        COUNT(DISTINCT status) as distinct_values,
        COUNT(*) as total_rows,
        CAST(COUNT(DISTINCT status) AS REAL) / COUNT(*) as selectivity
    FROM plugins
    UNION ALL
    SELECT 
        'rating',
        COUNT(DISTINCT CAST(rating * 10 AS INTEGER)),
        COUNT(*),
        CAST(COUNT(DISTINCT CAST(rating * 10 AS INTEGER)) AS REAL) / COUNT(*)
    FROM plugins
    UNION ALL
    SELECT 
        'downloads',
        COUNT(DISTINCT downloads),
        COUNT(*),
        CAST(COUNT(DISTINCT downloads) AS REAL) / COUNT(*)
    FROM plugins
)
SELECT * FROM column_stats
ORDER BY selectivity DESC;
```

### 2. 查询优化技巧

#### 使用EXPLAIN QUERY PLAN分析查询

```sql
-- 分析查询执行计划
EXPLAIN QUERY PLAN 
SELECT p.*, COUNT(pr.id) as rating_count
FROM plugins p
LEFT JOIN plugin_ratings pr ON p.id = pr.plugin_id
WHERE p.status = 'active' 
  AND p.rating >= 4.0
GROUP BY p.id
ORDER BY p.downloads DESC
LIMIT 20;
```

**执行计划关键指标**:
- **SCAN**: 全表扫描（应尽量避免）
- **SEARCH**: 使用索引搜索
- **USE INDEX**: 使用的索引名称
- **USING TEMP B-TREE**: 临时排序操作

#### 避免常见性能陷阱

**N+1查询问题**:
```sql
-- ❌ 差的做法：N+1查询
-- 应用层循环查询每个插件的评分
SELECT * FROM plugins WHERE status = 'active';
-- 然后对每个插件执行：
-- SELECT AVG(rating), COUNT(*) FROM plugin_ratings WHERE plugin_id = ?

-- ✅ 好的做法：一次性关联查询
SELECT 
    p.*,
    COALESCE(AVG(pr.rating), 0) as avg_rating,
    COUNT(pr.id) as rating_count
FROM plugins p
LEFT JOIN plugin_ratings pr ON p.id = pr.plugin_id
WHERE p.status = 'active'
GROUP BY p.id, p.name, p.description, p.author, p.current_version, 
         p.downloads, p.rating, p.status, p.created_at, p.updated_at;
```

**避免SELECT ***:
```sql
-- ❌ 避免使用SELECT *
SELECT * FROM plugins WHERE id = 'plugin_id';

-- ✅ 只查询需要的字段
SELECT id, name, description, current_version, downloads, rating 
FROM plugins WHERE id = 'plugin_id';
```

**优化OR条件**:
```sql
-- ❌ OR条件可能不使用索引
SELECT * FROM plugins 
WHERE status = 'active' OR status = 'deprecated';

-- ✅ 使用IN
SELECT * FROM plugins 
WHERE status IN ('active', 'deprecated');

-- 或者使用UNION（对于复杂条件）
SELECT * FROM plugins WHERE status = 'active'
UNION ALL
SELECT * FROM plugins WHERE status = 'deprecated';
```

**分页查询优化**:
```sql
-- ❌ 深分页性能差
SELECT * FROM plugins 
ORDER BY created_at DESC 
LIMIT 20 OFFSET 10000;

-- ✅ 使用游标分页
SELECT * FROM plugins 
WHERE created_at < '2024-01-15 10:30:00'
ORDER BY created_at DESC 
LIMIT 20;

-- ✅ 使用ID范围分页
SELECT * FROM plugin_ratings
WHERE id > 1000  -- 上一页的最大ID
ORDER BY id
LIMIT 20;
```

### 3. 复杂查询优化

#### 插件搜索查询优化

```sql
-- 使用FTS5进行全文搜索优化
-- 优化前的搜索查询（使用LIKE，性能差）
SELECT p.*, 
       0 as rank  -- SQLite没有内置排序函数
FROM plugins p
WHERE (p.name LIKE '%' || ? || '%' 
       OR p.description LIKE '%' || ? || '%')
  AND p.status = 'active'
ORDER BY p.downloads DESC
LIMIT 20;

-- 优化后：使用FTS5全文搜索
SELECT p.*, 
       plugins_fts.rank
FROM plugins_fts 
JOIN plugins p ON plugins_fts.rowid = p.id
WHERE plugins_fts MATCH ? 
  AND p.status = 'active'
ORDER BY plugins_fts.rank, p.downloads DESC
LIMIT 20;

-- 高级FTS5查询示例
-- 搜索包含多个词的插件
SELECT p.*, plugins_fts.rank
FROM plugins_fts 
JOIN plugins p ON plugins_fts.rowid = p.id
WHERE plugins_fts MATCH 'editor AND syntax'
  AND p.status = 'active'
ORDER BY plugins_fts.rank DESC
LIMIT 20;

-- 短语搜索
SELECT p.*, plugins_fts.rank
FROM plugins_fts 
JOIN plugins p ON plugins_fts.rowid = p.id
WHERE plugins_fts MATCH '"code editor"'
  AND p.status = 'active'
ORDER BY plugins_fts.rank DESC
LIMIT 20;

-- 字段特定搜索
SELECT p.*, plugins_fts.rank
FROM plugins_fts 
JOIN plugins p ON plugins_fts.rowid = p.id
WHERE plugins_fts MATCH 'name:editor OR description:syntax'
  AND p.status = 'active'
ORDER BY plugins_fts.rank DESC
LIMIT 20;
```

#### 统计查询优化

```sql
-- 使用视图预计算统计数据
CREATE VIEW dashboard_stats_view AS
SELECT 
    (SELECT COUNT(*) FROM users WHERE is_active = 1) as total_users,
    (SELECT COUNT(*) FROM plugins WHERE status = 'active') as total_plugins,
    (SELECT COALESCE(SUM(downloads), 0) FROM plugins) as total_downloads,
    (SELECT COUNT(*) FROM plugins WHERE created_at >= datetime('now', '-7 days')) as weekly_new_plugins,
    (SELECT COUNT(*) FROM users WHERE created_at >= datetime('now', '-7 days')) as weekly_new_users,
    datetime('now') as last_updated;

-- 对于经常变化的统计数据，可以使用缓存表
CREATE TABLE dashboard_stats_cache (
    stat_name TEXT PRIMARY KEY,
    stat_value INTEGER,
    last_updated DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 更新统计缓存的存储过程（通过应用层定期执行）
INSERT OR REPLACE INTO dashboard_stats_cache (stat_name, stat_value, last_updated)
VALUES 
    ('total_users', (SELECT COUNT(*) FROM users WHERE is_active = 1), datetime('now')),
    ('total_plugins', (SELECT COUNT(*) FROM plugins WHERE status = 'active'), datetime('now')),
    ('total_downloads', (SELECT COALESCE(SUM(downloads), 0) FROM plugins), datetime('now'));

-- 快速查询统计数据
SELECT stat_name, stat_value, last_updated 
FROM dashboard_stats_cache;
```

## 连接管理优化

### 1. SQLx连接池配置

```rust
use sqlx::sqlite::SqlitePoolOptions;
use std::time::Duration;

pub async fn create_database_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        // 连接池大小配置
        .max_connections(20)                    // 最大连接数
        .min_connections(1)                     // 最小连接数（SQLite通常保持较少连接）
        
        // 超时配置
        .acquire_timeout(Duration::from_secs(30))   // 获取连接超时
        .idle_timeout(Duration::from_secs(600))     // 空闲连接超时（10分钟）
        .max_lifetime(Duration::from_secs(1800))    // 连接最大生命周期（30分钟）
        
        // 连接测试
        .test_before_acquire(true)              // 获取前测试连接
        
        // SQLite特定配置
        .after_connect(|conn, _meta| Box::pin(async move {
            // 启用WAL模式提高并发性能
            sqlx::query("PRAGMA journal_mode = WAL")
                .execute(conn)
                .await?;
            
            // 设置同步模式为NORMAL（平衡性能和安全性）
            sqlx::query("PRAGMA synchronous = NORMAL")
                .execute(conn)
                .await?;
            
            // 设置缓存大小（以KB为单位）
            sqlx::query("PRAGMA cache_size = -64000")  // 64MB缓存
                .execute(conn)
                .await?;
            
            // 设置临时存储为内存
            sqlx::query("PRAGMA temp_store = MEMORY")
                .execute(conn)
                .await?;
            
            // 启用外键约束
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(conn)
                .await?;
            
            // 设置繁忙超时
            sqlx::query("PRAGMA busy_timeout = 30000")  // 30秒
                .execute(conn)
                .await?;
                
            Ok(())
        }))
        
        .connect(database_url)
        .await
}
```

### 2. 连接池监控

```rust
// SQLite连接池健康检查
pub struct SqlitePoolMonitor {
    pool: SqlitePool,
}

impl SqlitePoolMonitor {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    pub async fn get_pool_status(&self) -> PoolStatus {
        let pool_state = self.pool.state();
        
        PoolStatus {
            connections: pool_state.connections,
            idle_connections: pool_state.idle_connections,
            max_connections: pool_state.max_connections,
            utilization: (pool_state.connections as f64 / pool_state.max_connections as f64) * 100.0,
        }
    }
    
    pub async fn health_check(&self) -> Result<Duration, sqlx::Error> {
        let start = std::time::Instant::now();
        
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
            
        Ok(start.elapsed())
    }
    
    pub async fn get_sqlite_status(&self) -> Result<SqliteStatus, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            PRAGMA database_list;
            PRAGMA cache_size;
            PRAGMA page_count;
            PRAGMA page_size;
            PRAGMA freelist_count;
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        // 获取数据库大小信息
        let page_count: i64 = sqlx::query_scalar("PRAGMA page_count")
            .fetch_one(&self.pool)
            .await?;
            
        let page_size: i64 = sqlx::query_scalar("PRAGMA page_size")
            .fetch_one(&self.pool)
            .await?;
            
        let freelist_count: i64 = sqlx::query_scalar("PRAGMA freelist_count")
            .fetch_one(&self.pool)
            .await?;
            
        Ok(SqliteStatus {
            page_count,
            page_size,
            database_size_bytes: page_count * page_size,
            freelist_count,
            fragmentation_ratio: freelist_count as f64 / page_count as f64,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct SqliteStatus {
    pub page_count: i64,
    pub page_size: i64,
    pub database_size_bytes: i64,
    pub freelist_count: i64,
    pub fragmentation_ratio: f64,
}
```

### 3. SQLite性能优化配置

```rust
// SQLite性能优化PRAGMA设置
pub async fn optimize_sqlite_connection(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    // WAL模式提高并发读写性能
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&mut *conn)
        .await?;
    
    // 同步模式设置
    sqlx::query("PRAGMA synchronous = NORMAL")  // 或 PRAGMA synchronous = OFF 用于最大性能
        .execute(&mut *conn)
        .await?;
    
    // 缓存配置
    sqlx::query("PRAGMA cache_size = -64000")   // 64MB缓存
        .execute(&mut *conn)
        .await?;
    
    // 内存临时存储
    sqlx::query("PRAGMA temp_store = MEMORY")
        .execute(&mut *conn)
        .await?;
    
    // 内存映射大小（提高大文件性能）
    sqlx::query("PRAGMA mmap_size = 268435456") // 256MB
        .execute(&mut *conn)
        .await?;
    
    // 自动清理设置
    sqlx::query("PRAGMA auto_vacuum = INCREMENTAL")
        .execute(&mut *conn)
        .await?;
    
    // 分析表统计信息
    sqlx::query("PRAGMA optimize")
        .execute(&mut *conn)
        .await?;
    
    Ok(())
}
```

## 缓存策略

### 1. 应用层缓存

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

// 简单的内存缓存实现
pub struct MemoryCache<K, V> {
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
}

impl<K, V> MemoryCache<K, V> 
where 
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        let cache = self.data.read().ok()?;
        let entry = cache.get(key)?;
        
        // 检查是否过期
        if entry.inserted_at.elapsed() > self.ttl {
            return None;
        }
        
        Some(entry.value.clone())
    }
    
    pub fn insert(&self, key: K, value: V) {
        if let Ok(mut cache) = self.data.write() {
            cache.insert(key, CacheEntry {
                value,
                inserted_at: Instant::now(),
            });
        }
    }
    
    // 清理过期条目
    pub fn cleanup(&self) {
        if let Ok(mut cache) = self.data.write() {
            cache.retain(|_, entry| entry.inserted_at.elapsed() <= self.ttl);
        }
    }
}

// SQLite缓存服务
pub struct CachedSqliteService {
    pool: SqlitePool,
    plugin_cache: MemoryCache<String, Plugin>,
    stats_cache: MemoryCache<String, DashboardStats>,
}

impl CachedSqliteService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            plugin_cache: MemoryCache::new(Duration::from_secs(300)), // 5分钟TTL
            stats_cache: MemoryCache::new(Duration::from_secs(60)),   // 1分钟TTL
        }
    }
    
    pub async fn get_plugin_cached(&self, plugin_id: &str) -> Result<Plugin, ServiceError> {
        // 尝试从缓存获取
        if let Some(plugin) = self.plugin_cache.get(plugin_id) {
            return Ok(plugin);
        }
        
        // 缓存未命中，从数据库查询
        let plugin = self.get_plugin_from_db(plugin_id).await?;
        
        // 存入缓存
        self.plugin_cache.insert(plugin_id.to_string(), plugin.clone());
        
        Ok(plugin)
    }
    
    async fn get_plugin_from_db(&self, plugin_id: &str) -> Result<Plugin, ServiceError> {
        sqlx::query_as!(
            Plugin,
            "SELECT * FROM plugins WHERE id = ?",
            plugin_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e))
    }
}
```

### 2. SQLite查询结果缓存

```sql
-- SQLite不像PostgreSQL有内置的查询结果缓存
-- 可以通过应用层或数据库视图实现类似功能

-- 创建缓存表存储频繁查询的结果
CREATE TABLE query_cache (
    cache_key TEXT PRIMARY KEY,
    cache_data TEXT,  -- JSON格式的缓存数据
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME
);

-- 创建索引加速缓存查询
CREATE INDEX idx_query_cache_expires ON query_cache(expires_at);

-- 缓存热门插件列表
INSERT OR REPLACE INTO query_cache (cache_key, cache_data, expires_at)
VALUES (
    'popular_plugins_24h',
    (SELECT json_group_array(
        json_object(
            'id', id,
            'name', name,
            'rating', rating,
            'downloads', downloads
        )
    ) FROM plugins 
    WHERE status = 'active' 
    ORDER BY downloads DESC 
    LIMIT 20),
    datetime('now', '+1 hour')
);

-- 查询缓存数据
SELECT cache_data 
FROM query_cache 
WHERE cache_key = 'popular_plugins_24h' 
  AND expires_at > datetime('now');
```

### 3. Redis集成缓存

```rust
use redis::{AsyncCommands, Client, RedisResult};
use serde::{Deserialize, Serialize};

pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }
    
    pub async fn get<T>(&self, key: &str) -> RedisResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut conn = self.client.get_async_connection().await?;
        let data: Option<String> = conn.get(key).await?;
        
        match data {
            Some(json) => {
                let value: T = serde_json::from_str(&json)
                    .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "JSON parse error", e.to_string())))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
    
    pub async fn set<T>(&self, key: &str, value: &T, ttl: u64) -> RedisResult<()>
    where
        T: Serialize,
    {
        let mut conn = self.client.get_async_connection().await?;
        let json = serde_json::to_string(value)
            .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "JSON serialize error", e.to_string())))?;
        
        conn.set_ex(key, json, ttl).await
    }
    
    pub async fn invalidate(&self, pattern: &str) -> RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        
        // 获取匹配的键
        let keys: Vec<String> = conn.keys(pattern).await?;
        
        if !keys.is_empty() {
            conn.del(keys).await?;
        }
        
        Ok(())
    }
}

// SQLite + Redis分层缓存策略
pub struct TieredSqliteCache {
    memory_cache: MemoryCache<String, String>,
    redis_cache: RedisCache,
    sqlite_pool: SqlitePool,
}

impl TieredSqliteCache {
    pub async fn get<T>(&self, key: &str, db_query: impl Fn(&SqlitePool) -> BoxFuture<Result<T, sqlx::Error>>) -> Result<Option<T>, CacheError>
    where
        T: for<'de> Deserialize<'de> + Serialize + Clone,
    {
        // L1: 内存缓存
        if let Some(data) = self.memory_cache.get(key) {
            let value: T = serde_json::from_str(&data)?;
            return Ok(Some(value));
        }
        
        // L2: Redis缓存
        if let Some(value) = self.redis_cache.get::<T>(key).await? {
            // 回填到内存缓存
            let json = serde_json::to_string(&value)?;
            self.memory_cache.insert(key.to_string(), json);
            return Ok(Some(value));
        }
        
        // L3: SQLite数据库
        let db_result = db_query(&self.sqlite_pool).await;
        if let Ok(value) = db_result {
            // 存储到所有缓存层
            let json = serde_json::to_string(&value)?;
            self.memory_cache.insert(key.to_string(), json.clone());
            self.redis_cache.set(key, &value, 3600).await?; // 1小时TTL
            return Ok(Some(value));
        }
        
        Ok(None)
    }
}
```

## 监控和诊断

### 1. SQLite性能监控

```sql
-- SQLite性能分析查询
-- 分析表大小和页面使用情况
SELECT 
    name as table_name,
    tbl_name,
    CAST(COUNT(*) AS INTEGER) * 
    (SELECT CAST(value AS INTEGER) FROM pragma_page_size()) as estimated_size_bytes
FROM sqlite_master 
WHERE type = 'table'
GROUP BY name, tbl_name;

-- 分析索引使用情况（需要开启统计）
PRAGMA stats = ON;

-- 检查数据库完整性
PRAGMA integrity_check;

-- 快速检查
PRAGMA quick_check;

-- 分析数据库统计信息
PRAGMA table_info(plugins);
PRAGMA index_list(plugins);
PRAGMA index_info(idx_plugins_status);

-- 查看编译选项
PRAGMA compile_options;

-- WAL模式状态检查
PRAGMA journal_mode;
PRAGMA wal_checkpoint;

-- 缓存统计
PRAGMA cache_size;
PRAGMA cache_spill;

-- 分析查询计划
EXPLAIN QUERY PLAN SELECT * FROM plugins WHERE status = 'active';
```

### 2. 实时性能监控

```rust
use std::time::Instant;
use tracing::{info, warn};

// SQLite查询性能监控
pub struct SqliteQueryMonitor;

impl SqliteQueryMonitor {
    pub async fn execute_with_monitoring<T>(
        pool: &SqlitePool,
        query: &str,
        operation_name: &str,
    ) -> Result<T, sqlx::Error>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let start = Instant::now();
        
        let result = sqlx::query_as::<_, T>(query)
            .fetch_one(pool)
            .await;
        
        let duration = start.elapsed();
        
        // 记录查询性能
        if duration.as_millis() > 100 {
            warn!(
                operation = operation_name,
                duration_ms = duration.as_millis(),
                query = query,
                "Slow SQLite query detected"
            );
        } else {
            info!(
                operation = operation_name,
                duration_ms = duration.as_millis(),
                "SQLite query executed"
            );
        }
        
        result
    }
}

// SQLite性能指标收集
pub struct SqliteMetrics {
    pool: SqlitePool,
}

impl SqliteMetrics {
    pub async fn collect_metrics(&self) -> SqliteMetricsData {
        let database_stats = self.get_database_stats().await;
        let performance_stats = self.get_performance_stats().await;
        
        SqliteMetricsData {
            database_stats,
            performance_stats,
            collected_at: chrono::Utc::now(),
        }
    }
    
    async fn get_database_stats(&self) -> DatabaseStats {
        let page_count: i64 = sqlx::query_scalar("PRAGMA page_count")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
            
        let page_size: i64 = sqlx::query_scalar("PRAGMA page_size")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(4096);
            
        let freelist_count: i64 = sqlx::query_scalar("PRAGMA freelist_count")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
            
        let cache_size: i64 = sqlx::query_scalar("PRAGMA cache_size")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
            
        DatabaseStats {
            total_pages: page_count,
            page_size_bytes: page_size,
            database_size_bytes: page_count * page_size,
            free_pages: freelist_count,
            fragmentation_ratio: if page_count > 0 {
                freelist_count as f64 / page_count as f64
            } else {
                0.0
            },
            cache_size_pages: cache_size.abs(), // cache_size可能为负数（表示KB）
        }
    }
    
    async fn get_performance_stats(&self) -> PerformanceStats {
        // SQLite没有内置的查询统计，需要应用层收集
        let pool_state = self.pool.state();
        
        PerformanceStats {
            active_connections: pool_state.connections,
            idle_connections: pool_state.idle_connections,
            max_connections: pool_state.max_connections,
            connection_utilization: pool_state.connections as f64 / pool_state.max_connections as f64,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SqliteMetricsData {
    pub database_stats: DatabaseStats,
    pub performance_stats: PerformanceStats,
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct DatabaseStats {
    pub total_pages: i64,
    pub page_size_bytes: i64,
    pub database_size_bytes: i64,
    pub free_pages: i64,
    pub fragmentation_ratio: f64,
    pub cache_size_pages: i64,
}
```

### 3. SQLite健康检查和告警

```rust
pub struct SqliteHealthChecker {
    pool: SqlitePool,
    thresholds: SqliteThresholds,
}

#[derive(Debug)]
pub struct SqliteThresholds {
    pub slow_query_ms: u64,
    pub database_size_mb: u64,
    pub fragmentation_ratio: f64,
    pub connection_utilization: f64,
}

impl Default for SqliteThresholds {
    fn default() -> Self {
        Self {
            slow_query_ms: 500,           // 500ms
            database_size_mb: 1000,       // 1GB
            fragmentation_ratio: 0.15,    // 15%
            connection_utilization: 0.8,   // 80%
        }
    }
}

impl SqliteHealthChecker {
    pub async fn check_health(&self) -> Result<HealthReport, HealthCheckError> {
        let mut issues = Vec::new();
        let metrics = SqliteMetrics::new(self.pool.clone()).collect_metrics().await;
        
        // 检查数据库大小
        let db_size_mb = metrics.database_stats.database_size_bytes / (1024 * 1024);
        if db_size_mb > self.thresholds.database_size_mb as i64 {
            issues.push(HealthIssue {
                severity: Severity::Warning,
                category: Category::Storage,
                message: format!("Database size ({} MB) exceeds threshold", db_size_mb),
            });
        }
        
        // 检查碎片化程度
        if metrics.database_stats.fragmentation_ratio > self.thresholds.fragmentation_ratio {
            issues.push(HealthIssue {
                severity: Severity::Warning,
                category: Category::Performance,
                message: format!(
                    "Database fragmentation ({:.1}%) exceeds threshold",
                    metrics.database_stats.fragmentation_ratio * 100.0
                ),
                recommendation: Some("Consider running VACUUM to defragment the database".to_string()),
            });
        }
        
        // 检查连接池使用率
        if metrics.performance_stats.connection_utilization > self.thresholds.connection_utilization {
            issues.push(HealthIssue {
                severity: Severity::Warning,
                category: Category::Connections,
                message: format!(
                    "Connection utilization ({:.1}%) is high",
                    metrics.performance_stats.connection_utilization * 100.0
                ),
            });
        }
        
        Ok(HealthReport {
            status: if issues.is_empty() { HealthStatus::Healthy } else { HealthStatus::Warning },
            issues,
            metrics,
        })
    }
    
    pub async fn perform_maintenance(&self) -> Result<MaintenanceReport, MaintenanceError> {
        let mut actions = Vec::new();
        
        // 执行VACUUM清理碎片
        let start = Instant::now();
        sqlx::query("VACUUM")
            .execute(&self.pool)
            .await?;
        actions.push(MaintenanceAction {
            action_type: ActionType::Vacuum,
            duration: start.elapsed(),
            result: "Database defragmented successfully".to_string(),
        });
        
        // 更新统计信息
        let start = Instant::now();
        sqlx::query("PRAGMA optimize")
            .execute(&self.pool)
            .await?;
        actions.push(MaintenanceAction {
            action_type: ActionType::Analyze,
            duration: start.elapsed(),
            result: "Statistics updated successfully".to_string(),
        });
        
        // 检查数据库完整性
        let start = Instant::now();
        let integrity_result: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await?;
        actions.push(MaintenanceAction {
            action_type: ActionType::IntegrityCheck,
            duration: start.elapsed(),
            result: integrity_result,
        });
        
        Ok(MaintenanceReport {
            performed_at: chrono::Utc::now(),
            actions,
        })
    }
}
```

## 压力测试和基准测试

### 1. SQLite基准测试

```bash
#!/bin/bash
# SQLite性能测试脚本

DATABASE_FILE="./test_performance.db"
TEST_DATA_SIZE=10000

# 清理旧的测试数据
rm -f $DATABASE_FILE

# 创建测试表
sqlite3 $DATABASE_FILE << EOF
CREATE TABLE test_plugins (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    author TEXT NOT NULL,
    rating REAL DEFAULT 0.0,
    downloads INTEGER DEFAULT 0,
    status TEXT DEFAULT 'active',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_test_status ON test_plugins(status);
CREATE INDEX idx_test_rating ON test_plugins(rating DESC);
CREATE INDEX idx_test_downloads ON test_plugins(downloads DESC);
EOF

# 插入测试数据
echo "Inserting $TEST_DATA_SIZE test records..."
time sqlite3 $DATABASE_FILE << EOF
BEGIN TRANSACTION;
$(for i in $(seq 1 $TEST_DATA_SIZE); do
    echo "INSERT INTO test_plugins (id, name, description, author, rating, downloads) VALUES ('plugin_$i', 'Test Plugin $i', 'Description for plugin $i', 'Author $i', $((RANDOM % 50 + 10))/10.0, $((RANDOM % 10000)));"
done)
COMMIT;
EOF

# 执行查询性能测试
echo "Running query performance tests..."

# 测试简单查询
echo "Simple SELECT test:"
time sqlite3 $DATABASE_FILE "SELECT COUNT(*) FROM test_plugins;"

# 测试带WHERE条件的查询
echo "WHERE clause test:"
time sqlite3 $DATABASE_FILE "SELECT * FROM test_plugins WHERE status = 'active' LIMIT 100;"

# 测试排序查询
echo "ORDER BY test:"
time sqlite3 $DATABASE_FILE "SELECT * FROM test_plugins ORDER BY downloads DESC LIMIT 100;"

# 测试聚合查询
echo "Aggregation test:"
time sqlite3 $DATABASE_FILE "SELECT status, COUNT(*), AVG(rating) FROM test_plugins GROUP BY status;"

# 测试复杂JOIN查询（如果有关联表）
echo "Complex query test:"
time sqlite3 $DATABASE_FILE "SELECT * FROM test_plugins WHERE rating > 4.0 AND downloads > 1000 ORDER BY rating DESC, downloads DESC LIMIT 50;"

# 清理测试数据
rm -f $DATABASE_FILE
echo "Performance test completed."
```

### 2. 应用层压力测试

```rust
use tokio::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use sqlx::SqlitePool;

pub struct SqliteLoadTester {
    pool: SqlitePool,
    stats: Arc<LoadTestStats>,
}

#[derive(Default)]
pub struct LoadTestStats {
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub total_duration_ms: AtomicU64,
    pub read_operations: AtomicU64,
    pub write_operations: AtomicU64,
}

impl SqliteLoadTester {
    pub async fn run_mixed_load_test(&self, concurrent_users: usize, duration: Duration) {
        let start_time = Instant::now();
        let mut handles = Vec::new();
        
        // 80% 读操作，20% 写操作
        let read_users = (concurrent_users as f64 * 0.8) as usize;
        let write_users = concurrent_users - read_users;
        
        // 启动读操作用户
        for _ in 0..read_users {
            let pool = self.pool.clone();
            let stats = self.stats.clone();
            let test_duration = duration;
            
            let handle = tokio::spawn(async move {
                Self::read_user_simulation(pool, stats, test_duration).await;
            });
            
            handles.push(handle);
        }
        
        // 启动写操作用户
        for user_id in 0..write_users {
            let pool = self.pool.clone();
            let stats = self.stats.clone();
            let test_duration = duration;
            
            let handle = tokio::spawn(async move {
                Self::write_user_simulation(pool, stats, test_duration, user_id).await;
            });
            
            handles.push(handle);
        }
        
        // 等待所有测试完成
        for handle in handles {
            let _ = handle.await;
        }
        
        self.print_results(start_time.elapsed()).await;
    }
    
    async fn read_user_simulation(pool: SqlitePool, stats: Arc<LoadTestStats>, duration: Duration) {
        let start = Instant::now();
        
        while start.elapsed() < duration {
            let request_start = Instant::now();
            stats.total_requests.fetch_add(1, Ordering::Relaxed);
            stats.read_operations.fetch_add(1, Ordering::Relaxed);
            
            // 模拟各种读操作
            let operations = [
                // 热门插件查询
                "SELECT id, name, rating, downloads FROM plugins WHERE status = 'active' ORDER BY downloads DESC LIMIT 20",
                // 按评分查询
                "SELECT id, name, rating FROM plugins WHERE rating >= 4.0 ORDER BY rating DESC LIMIT 10",
                // 特定插件查询
                "SELECT * FROM plugins WHERE id = 'plugin_1'",
                // 统计查询
                "SELECT COUNT(*) as total, AVG(rating) as avg_rating FROM plugins WHERE status = 'active'",
            ];
            
            let query = operations[fastrand::usize(0..operations.len())];
            let result = sqlx::query(query).fetch_all(&pool).await;
            
            let request_duration = request_start.elapsed();
            stats.total_duration_ms.fetch_add(request_duration.as_millis() as u64, Ordering::Relaxed);
            
            match result {
                Ok(_) => {
                    stats.successful_requests.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    stats.failed_requests.fetch_add(1, Ordering::Relaxed);
                    eprintln!("Read request failed: {}", e);
                }
            }
            
            // 模拟用户思考时间
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
    
    async fn write_user_simulation(pool: SqlitePool, stats: Arc<LoadTestStats>, duration: Duration, user_id: usize) {
        let start = Instant::now();
        let mut operation_counter = 0;
        
        while start.elapsed() < duration {
            let request_start = Instant::now();
            stats.total_requests.fetch_add(1, Ordering::Relaxed);
            stats.write_operations.fetch_add(1, Ordering::Relaxed);
            operation_counter += 1;
            
            // 模拟写操作：插入评分
            let plugin_id = format!("plugin_{}", fastrand::u32(1..=1000));
            let rating = fastrand::f32() * 4.0 + 1.0; // 1.0-5.0
            let review = format!("Test review {} from user {}", operation_counter, user_id);
            
            let result = sqlx::query!(
                "INSERT INTO plugin_ratings (plugin_id, user_id, rating, review, created_at) VALUES (?, ?, ?, ?, datetime('now'))",
                plugin_id,
                format!("user_{}", user_id),
                rating,
                review
            )
            .execute(&pool)
            .await;
            
            let request_duration = request_start.elapsed();
            stats.total_duration_ms.fetch_add(request_duration.as_millis() as u64, Ordering::Relaxed);
            
            match result {
                Ok(_) => {
                    stats.successful_requests.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    stats.failed_requests.fetch_add(1, Ordering::Relaxed);
                    eprintln!("Write request failed: {}", e);
                }
            }
            
            // 写操作间隔较长
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }
    
    async fn print_results(&self, total_duration: Duration) {
        let total = self.stats.total_requests.load(Ordering::Relaxed);
        let successful = self.stats.successful_requests.load(Ordering::Relaxed);
        let failed = self.stats.failed_requests.load(Ordering::Relaxed);
        let total_time_ms = self.stats.total_duration_ms.load(Ordering::Relaxed);
        let read_ops = self.stats.read_operations.load(Ordering::Relaxed);
        let write_ops = self.stats.write_operations.load(Ordering::Relaxed);
        
        println!("=== SQLite Load Test Results ===");
        println!("Total Duration: {:.2}s", total_duration.as_secs_f64());
        println!("Total Requests: {}", total);
        println!("Successful Requests: {}", successful);
        println!("Failed Requests: {}", failed);
        println!("Success Rate: {:.2}%", (successful as f64 / total as f64) * 100.0);
        println!("Average Response Time: {:.2}ms", total_time_ms as f64 / total as f64);
        println!("Requests per Second: {:.2}", total as f64 / total_duration.as_secs_f64());
        println!("Read Operations: {} ({:.1}%)", read_ops, (read_ops as f64 / total as f64) * 100.0);
        println!("Write Operations: {} ({:.1}%)", write_ops, (write_ops as f64 / total as f64) * 100.0);
    }
}

// 使用示例
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_database_pool("sqlite:./test.db").await?;
    let load_tester = SqliteLoadTester {
        pool,
        stats: Arc::new(LoadTestStats::default()),
    };
    
    // 运行混合负载测试：10个用户，持续60秒
    load_tester.run_mixed_load_test(10, Duration::from_secs(60)).await;
    
    Ok(())
}
```

## SQLite性能优化清单

### 数据库层面

- [ ] **WAL模式**: 启用WAL模式提高并发性能
- [ ] **索引优化**: 分析查询模式，创建合适的B-tree和FTS5索引
- [ ] **PRAGMA优化**: 配置cache_size、synchronous、temp_store等
- [ ] **查询优化**: 避免N+1查询，使用适当的JOIN和LIMIT
- [ ] **分页优化**: 使用游标分页替代OFFSET
- [ ] **FTS5搜索**: 使用FTS5替代LIKE进行全文搜索
- [ ] **定期维护**: 执行VACUUM和PRAGMA optimize

### 应用层面

- [ ] **连接池**: 合理配置SQLite连接池参数
- [ ] **缓存策略**: 实施多层缓存机制（内存+Redis）
- [ ] **查询监控**: 监控慢查询和数据库大小
- [ ] **批量操作**: 使用事务批量处理写操作
- [ ] **异步处理**: 非关键操作异步化
- [ ] **读写分离**: 考虑使用只读副本处理查询密集型操作

### 系统层面

- [ ] **存储优化**: 使用SSD存储提高I/O性能
- [ ] **内存配置**: 配置充足的系统内存用于文件系统缓存
- [ ] **并发限制**: 合理控制并发写操作数量
- [ ] **监控告警**: 建立数据库大小、碎片化等监控
- [ ] **备份策略**: 建立定期备份和恢复测试流程
- [ ] **容量规划**: 监控数据库增长趋势并规划扩容

### SQLite特定优化

- [ ] **文件系统**: 使用支持fallocate的现代文件系统
- [ ] **内存映射**: 适当配置mmap_size提高大文件访问性能
- [ ] **预写日志**: 定期检查点WAL文件避免过度增长
- [ ] **完整性检查**: 定期运行PRAGMA integrity_check
- [ ] **统计更新**: 定期执行PRAGMA optimize更新查询计划统计
- [ ] **碎片整理**: 监控碎片化程度，必要时执行VACUUM

通过系统性的SQLite性能优化策略，GeekTools 插件市场能够在高并发场景下保持良好的响应性能和稳定性。SQLite的简单性和可靠性使其成为中小型应用的理想选择。