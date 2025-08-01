# 数据库性能优化策略

## 概述

GeekTools 插件市场的数据库性能优化涵盖查询优化、索引设计、连接池管理、缓存策略等多个方面。本文档提供全面的性能优化指南，确保系统在高并发和大数据量场景下的稳定运行。

## 查询性能优化

### 1. 索引设计策略

#### 主要索引类型

**B-tree索引**（默认类型）:
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

**部分索引**（过滤索引）:
```sql
-- 只为活跃插件创建索引
CREATE INDEX idx_active_plugins_rating 
ON plugins(rating DESC) 
WHERE status = 'active';

-- 只为失败登录创建索引
CREATE INDEX idx_failed_logins 
ON user_login_activities(ip_address, login_time) 
WHERE is_successful = false;

-- 只为最近数据创建索引
CREATE INDEX idx_recent_plugin_ratings 
ON plugin_ratings(created_at DESC) 
WHERE created_at >= CURRENT_DATE - INTERVAL '30 days';
```

**GIN索引**（全文搜索）:
```sql
-- 全文搜索索引
CREATE INDEX idx_plugins_search_vector 
ON plugins USING gin(to_tsvector('english', name || ' ' || COALESCE(description, '')));

-- 数组字段索引
CREATE INDEX idx_plugin_tags_gin 
ON plugins USING gin(tags) 
WHERE tags IS NOT NULL;
```

**哈希索引**（等值查询）:
```sql
-- 适用于等值查询的字段
CREATE INDEX idx_plugins_id_hash 
ON plugins USING hash(id);
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
SELECT 
    column_name,
    COUNT(DISTINCT column_name) as distinct_values,
    COUNT(*) as total_rows,
    COUNT(DISTINCT column_name)::float / COUNT(*) as selectivity
FROM (
    SELECT status, rating, downloads FROM plugins
) t,
LATERAL (VALUES ('status'), ('rating'), ('downloads')) AS cols(column_name)
GROUP BY column_name
ORDER BY selectivity DESC;
```

### 2. 查询优化技巧

#### 使用EXPLAIN分析查询

```sql
-- 分析查询执行计划
EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) 
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
- **Cost**: 查询成本估算
- **Rows**: 预估返回行数
- **Actual Time**: 实际执行时间
- **Buffers**: 缓冲区命中情况
- **Index Usage**: 索引使用情况

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

-- ✅ 使用IN或UNION
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
-- 优化前的搜索查询
SELECT p.*, 
       ts_rank(to_tsvector('english', p.name || ' ' || COALESCE(p.description, '')), 
               plainto_tsquery('english', $1)) as rank
FROM plugins p
WHERE to_tsvector('english', p.name || ' ' || COALESCE(p.description, '')) 
      @@ plainto_tsquery('english', $1)
  AND p.status = 'active'
ORDER BY rank DESC, p.downloads DESC
LIMIT 20;

-- 优化后：预计算搜索向量
-- 1. 添加搜索向量字段
ALTER TABLE plugins ADD COLUMN search_vector tsvector;

-- 2. 创建更新函数
CREATE OR REPLACE FUNCTION update_plugin_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector := to_tsvector('english', 
        NEW.name || ' ' || COALESCE(NEW.description, '') || ' ' || 
        COALESCE(NEW.author, '') || ' ' || 
        COALESCE(array_to_string(
            (SELECT array_agg(tag) FROM plugin_tags WHERE plugin_id = NEW.id), 
            ' '
        ), '')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 3. 创建触发器
CREATE TRIGGER plugin_search_vector_update
    BEFORE INSERT OR UPDATE ON plugins
    FOR EACH ROW EXECUTE FUNCTION update_plugin_search_vector();

-- 4. 创建GIN索引
CREATE INDEX idx_plugins_search_vector ON plugins USING gin(search_vector);

-- 5. 优化后的查询
SELECT p.*, 
       ts_rank(p.search_vector, plainto_tsquery('english', $1)) as rank
FROM plugins p
WHERE p.search_vector @@ plainto_tsquery('english', $1)
  AND p.status = 'active'
ORDER BY rank DESC, p.downloads DESC
LIMIT 20;
```

#### 统计查询优化

```sql
-- 优化仪表板统计查询
-- 创建物化视图定期更新统计数据
CREATE MATERIALIZED VIEW dashboard_stats AS
SELECT 
    (SELECT COUNT(*) FROM users WHERE is_active = true) as total_users,
    (SELECT COUNT(*) FROM plugins WHERE status = 'active') as total_plugins,
    (SELECT COALESCE(SUM(downloads), 0) FROM plugins) as total_downloads,
    (SELECT COUNT(*) FROM plugins WHERE created_at >= CURRENT_DATE - INTERVAL '7 days') as weekly_new_plugins,
    (SELECT COUNT(*) FROM users WHERE created_at >= CURRENT_DATE - INTERVAL '7 days') as weekly_new_users,
    NOW() as last_updated;

-- 创建索引
CREATE UNIQUE INDEX idx_dashboard_stats_unique ON dashboard_stats(last_updated);

-- 定期刷新统计数据（通过定时任务）
REFRESH MATERIALIZED VIEW CONCURRENTLY dashboard_stats;

-- 快速查询统计数据
SELECT * FROM dashboard_stats;
```

## 连接池优化

### 1. SQLx连接池配置

```rust
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub async fn create_database_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        // 连接池大小配置
        .max_connections(20)                    // 最大连接数
        .min_connections(5)                     // 最小连接数
        
        // 超时配置
        .acquire_timeout(Duration::from_secs(30))   // 获取连接超时
        .idle_timeout(Duration::from_secs(600))     // 空闲连接超时（10分钟）
        .max_lifetime(Duration::from_secs(1800))    // 连接最大生命周期（30分钟）
        
        // 连接测试
        .test_before_acquire(true)              // 获取前测试连接
        
        // 连接配置
        .after_connect(|conn, _meta| Box::pin(async move {
            // 设置连接级别的参数
            sqlx::query("SET application_name = 'geektools-marketplace'")
                .execute(conn)
                .await?;
            
            sqlx::query("SET timezone = 'UTC'")
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
// 连接池健康检查
pub struct PoolMonitor {
    pool: PgPool,
}

impl PoolMonitor {
    pub fn new(pool: PgPool) -> Self {
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
}

#[derive(Debug, Serialize)]
pub struct PoolStatus {
    pub connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
    pub utilization: f64,
}
```

### 3. 连接泄漏检测

```rust
// 连接使用模式分析
pub async fn analyze_connection_usage(pool: &PgPool) {
    // 定期收集连接池状态
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    
    loop {
        interval.tick().await;
        
        let state = pool.state();
        
        // 记录连接池状态
        info!(
            "Pool status - Total: {}, Idle: {}, Utilization: {:.1}%",
            state.connections,
            state.idle_connections,
            (state.connections as f64 / state.max_connections as f64) * 100.0
        );
        
        // 检测异常情况
        if state.connections >= state.max_connections {
            warn!("Connection pool exhausted!");
        }
        
        if state.idle_connections == 0 && state.connections > 0 {
            warn!("No idle connections available");
        }
    }
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

// 在服务中使用缓存
pub struct CachedPluginService {
    pool: PgPool,
    plugin_cache: MemoryCache<String, Plugin>,
    stats_cache: MemoryCache<String, DashboardStats>,
}

impl CachedPluginService {
    pub fn new(pool: PgPool) -> Self {
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
            "SELECT * FROM plugins WHERE id = $1",
            plugin_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e))
    }
}
```

### 2. 查询结果缓存

```sql
-- 使用PostgreSQL的查询结果缓存
-- 设置shared_preload_libraries = 'pg_stat_statements'

-- 分析缓存命中率
SELECT 
    schemaname,
    tablename,
    heap_blks_read,
    heap_blks_hit,
    CASE 
        WHEN heap_blks_hit + heap_blks_read = 0 THEN 0
        ELSE round(heap_blks_hit::numeric / (heap_blks_hit + heap_blks_read) * 100, 2)
    END as cache_hit_ratio
FROM pg_statio_user_tables
ORDER BY cache_hit_ratio DESC;

-- 优化缓存配置
-- postgresql.conf
shared_buffers = 256MB          -- 共享缓冲区大小
effective_cache_size = 1GB      -- 系统缓存大小
random_page_cost = 1.1          -- 随机页面读取成本
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

// 分层缓存策略
pub struct TieredCache {
    memory_cache: MemoryCache<String, String>,
    redis_cache: RedisCache,
}

impl TieredCache {
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
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
        
        Ok(None)
    }
}
```

## 监控和诊断

### 1. 性能监控查询

```sql
-- 慢查询分析
SELECT 
    query,
    calls,
    total_time,
    mean_time,
    stddev_time,
    rows,
    100.0 * shared_blks_hit / nullif(shared_blks_hit + shared_blks_read, 0) AS hit_percent
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 20;

-- 表访问统计
SELECT 
    schemaname,
    tablename,
    seq_scan,                    -- 顺序扫描次数
    seq_tup_read,               -- 顺序扫描读取行数
    idx_scan,                   -- 索引扫描次数
    idx_tup_fetch,              -- 索引扫描获取行数
    n_tup_ins + n_tup_upd + n_tup_del as total_modifications
FROM pg_stat_user_tables
ORDER BY seq_scan DESC;

-- 索引使用率分析
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan,                   -- 索引使用次数
    idx_tup_read,              -- 索引读取行数
    idx_tup_fetch              -- 索引获取行数
FROM pg_stat_user_indexes
WHERE idx_scan = 0             -- 找出未使用的索引
ORDER BY schemaname, tablename;

-- 数据库大小监控
SELECT 
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as total_size,
    pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) as table_size,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename) - pg_relation_size(schemaname||'.'||tablename)) as index_size
FROM pg_tables 
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

### 2. 实时性能监控

```rust
use std::time::Instant;
use tracing::{info, warn};

// 查询性能监控中间件
pub struct QueryMonitor;

impl QueryMonitor {
    pub async fn execute_with_monitoring<T>(
        pool: &PgPool,
        query: &str,
        operation_name: &str,
    ) -> Result<T, sqlx::Error>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
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
                "Slow query detected"
            );
        } else {
            info!(
                operation = operation_name,
                duration_ms = duration.as_millis(),
                "Query executed"
            );
        }
        
        result
    }
}

// 性能指标收集
pub struct DatabaseMetrics {
    pool: PgPool,
}

impl DatabaseMetrics {
    pub async fn collect_metrics(&self) -> DatabaseMetricsData {
        let connection_stats = self.get_connection_stats().await;
        let query_stats = self.get_query_stats().await;
        let table_stats = self.get_table_stats().await;
        
        DatabaseMetricsData {
            connection_stats,
            query_stats,
            table_stats,
            collected_at: chrono::Utc::now(),
        }
    }
    
    async fn get_connection_stats(&self) -> ConnectionStats {
        let result = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_connections,
                COUNT(*) FILTER (WHERE state = 'active') as active_connections,
                COUNT(*) FILTER (WHERE state = 'idle') as idle_connections
            FROM pg_stat_activity
            WHERE datname = current_database()
            "#
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or_default();
        
        ConnectionStats {
            total: result.total_connections.unwrap_or(0) as u32,
            active: result.active_connections.unwrap_or(0) as u32,
            idle: result.idle_connections.unwrap_or(0) as u32,
        }
    }
    
    async fn get_query_stats(&self) -> Vec<QueryStat> {
        sqlx::query!(
            r#"
            SELECT 
                query,
                calls,
                total_time,
                mean_time,
                rows
            FROM pg_stat_statements
            ORDER BY mean_time DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|row| QueryStat {
            query: row.query.unwrap_or_default(),
            calls: row.calls.unwrap_or(0) as u64,
            total_time: row.total_time.unwrap_or(0.0),
            mean_time: row.mean_time.unwrap_or(0.0),
            rows: row.rows.unwrap_or(0) as u64,
        })
        .collect()
    }
}
```

### 3. 告警和报警

```rust
pub struct PerformanceAlerter {
    thresholds: PerformanceThresholds,
}

#[derive(Debug)]
pub struct PerformanceThresholds {
    pub slow_query_ms: u64,
    pub connection_utilization: f64,
    pub cache_hit_ratio: f64,
    pub table_bloat_ratio: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            slow_query_ms: 1000,      // 1秒
            connection_utilization: 0.8, // 80%
            cache_hit_ratio: 0.95,    // 95%
            table_bloat_ratio: 0.2,   // 20%
        }
    }
}

impl PerformanceAlerter {
    pub async fn check_performance(&self, metrics: &DatabaseMetricsData) {
        // 检查连接池使用率
        let pool_utilization = metrics.connection_stats.active as f64 
            / (metrics.connection_stats.total as f64);
        
        if pool_utilization > self.thresholds.connection_utilization {
            self.send_alert(Alert {
                severity: AlertSeverity::Warning,
                message: format!(
                    "High connection pool utilization: {:.1}%",
                    pool_utilization * 100.0
                ),
                category: AlertCategory::ConnectionPool,
            }).await;
        }
        
        // 检查慢查询
        for query_stat in &metrics.query_stats {
            if query_stat.mean_time > self.thresholds.slow_query_ms as f64 {
                self.send_alert(Alert {
                    severity: AlertSeverity::Warning,
                    message: format!(
                        "Slow query detected: {:.2}ms average time",
                        query_stat.mean_time
                    ),
                    category: AlertCategory::SlowQuery,
                }).await;
            }
        }
    }
    
    async fn send_alert(&self, alert: Alert) {
        // 实现告警发送逻辑
        // 可以发送到Slack、邮件、监控系统等
        warn!(
            severity = ?alert.severity,
            category = ?alert.category,
            message = alert.message,
            "Performance alert triggered"
        );
    }
}
```

## 压力测试和基准测试

### 1. 数据库压测

```bash
#!/bin/bash
# 数据库压力测试脚本

# 使用pgbench进行基准测试
DATABASE_URL="postgres://user:pass@localhost:5432/marketplace"

# 初始化测试数据
pgbench -i -s 10 $DATABASE_URL

# 运行标准测试
echo "Running standard benchmark..."
pgbench -c 10 -j 2 -t 1000 $DATABASE_URL

# 自定义测试脚本
cat > custom_test.sql << EOF
\set plugin_id random(1, 1000)
SELECT * FROM plugins WHERE id = :plugin_id;
INSERT INTO plugin_ratings (plugin_id, user_id, rating, review) 
VALUES (:plugin_id, :plugin_id, random(1, 5), 'Test review');
EOF

# 运行自定义测试
pgbench -c 20 -j 4 -t 500 -f custom_test.sql $DATABASE_URL
```

### 2. 应用层压测

```rust
use tokio::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct LoadTester {
    pool: PgPool,
    stats: Arc<TestStats>,
}

#[derive(Default)]
pub struct TestStats {
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub total_duration_ms: AtomicU64,
}

impl LoadTester {
    pub async fn run_load_test(&self, concurrent_users: usize, duration: Duration) {
        let start_time = Instant::now();
        let mut handles = Vec::new();
        
        for _ in 0..concurrent_users {
            let pool = self.pool.clone();
            let stats = self.stats.clone();
            let test_duration = duration;
            
            let handle = tokio::spawn(async move {
                Self::user_simulation(pool, stats, test_duration).await;
            });
            
            handles.push(handle);
        }
        
        // 等待所有测试完成
        for handle in handles {
            let _ = handle.await;
        }
        
        self.print_results(start_time.elapsed()).await;
    }
    
    async fn user_simulation(pool: PgPool, stats: Arc<TestStats>, duration: Duration) {
        let start = Instant::now();
        
        while start.elapsed() < duration {
            let request_start = Instant::now();
            stats.total_requests.fetch_add(1, Ordering::Relaxed);
            
            // 模拟用户操作：搜索插件
            let result = sqlx::query!(
                "SELECT id, name, rating FROM plugins WHERE status = 'active' ORDER BY downloads DESC LIMIT 10"
            )
            .fetch_all(&pool)
            .await;
            
            let request_duration = request_start.elapsed();
            stats.total_duration_ms.fetch_add(request_duration.as_millis() as u64, Ordering::Relaxed);
            
            match result {
                Ok(_) => {
                    stats.successful_requests.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    stats.failed_requests.fetch_add(1, Ordering::Relaxed);
                    eprintln!("Request failed: {}", e);
                }
            }
            
            // 模拟用户思考时间
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    async fn print_results(&self, total_duration: Duration) {
        let total = self.stats.total_requests.load(Ordering::Relaxed);
        let successful = self.stats.successful_requests.load(Ordering::Relaxed);
        let failed = self.stats.failed_requests.load(Ordering::Relaxed);
        let total_time_ms = self.stats.total_duration_ms.load(Ordering::Relaxed);
        
        println!("=== Load Test Results ===");
        println!("Total Duration: {:.2}s", total_duration.as_secs_f64());
        println!("Total Requests: {}", total);
        println!("Successful Requests: {}", successful);
        println!("Failed Requests: {}", failed);
        println!("Success Rate: {:.2}%", (successful as f64 / total as f64) * 100.0);
        println!("Average Response Time: {:.2}ms", total_time_ms as f64 / total as f64);
        println!("Requests per Second: {:.2}", total as f64 / total_duration.as_secs_f64());
    }
}

// 使用示例
#[tokio::main]
async fn main() {
    let pool = create_database_pool("postgres://...").await.unwrap();
    let load_tester = LoadTester {
        pool,
        stats: Arc::new(TestStats::default()),
    };
    
    // 运行10个并发用户，持续60秒的负载测试
    load_tester.run_load_test(10, Duration::from_secs(60)).await;
}
```

## 性能优化清单

### 数据库层面

- [ ] **索引优化**: 分析查询模式，创建合适的索引
- [ ] **查询优化**: 避免N+1查询，使用适当的JOIN
- [ ] **分页优化**: 使用游标分页替代OFFSET
- [ ] **统计信息**: 定期更新表统计信息 (ANALYZE)
- [ ] **定期维护**: 清理无用索引，重建碎片化索引

### 应用层面

- [ ] **连接池**: 合理配置连接池大小和超时
- [ ] **缓存策略**: 实施多层缓存机制
- [ ] **查询监控**: 监控慢查询和资源使用
- [ ] **批量操作**: 使用批量插入/更新减少网络开销
- [ ] **异步处理**: 非关键操作异步化

### 系统层面

- [ ] **硬件资源**: 充足的内存和快速的存储
- [ ] **数据库配置**: 优化PostgreSQL配置参数
- [ ] **网络优化**: 减少网络延迟和带宽使用
- [ ] **监控告警**: 建立完善的性能监控体系
- [ ] **容量规划**: 基于增长趋势进行容量规划

通过系统性的性能优化策略，GeekTools 插件市场能够在高并发场景下保持良好的响应性能和稳定性。