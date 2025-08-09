# 部署指南

## 部署方式概览

GeekTools插件市场支持多种部署方式，适应不同的环境需求：

### 1. 开发环境部署
- **用途**: 本地开发和测试
- **特点**: 快速启动，支持热重载
- **适用场景**: 开发者日常开发工作

### 2. Docker部署
- **用途**: 容器化部署
- **特点**: 环境隔离，易于管理
- **适用场景**: 开发、测试和生产环境

### 3. 传统服务器部署
- **用途**: 直接在服务器上运行
- **特点**: 性能最优，完全控制
- **适用场景**: 生产环境

### 4. 云平台部署
- **用途**: 云服务提供商部署
- **特点**: 弹性扩展，托管服务
- **适用场景**: 高可用生产环境

## 系统要求

### 最低要求

#### 硬件要求
```
CPU: 1核心 (推荐2核心+)
内存: 1GB RAM (推荐2GB+)
存储: 5GB可用空间 (推荐20GB+)
网络: 10Mbps带宽
```

#### 软件要求
```
操作系统: Linux (Ubuntu 20.04+, CentOS 8+) / macOS / Windows
Rust: 1.70+
SQLite: 3.35+ (内嵌支持，无需单独安装)
Python: 3.7+ (用于代理服务器)
```

### 推荐配置

#### 生产环境
```
CPU: 4核心+
内存: 4GB+ RAM
存储: 50GB+ SSD
网络: 100Mbps+带宽
负载均衡: Nginx/Apache
HTTPS: SSL证书
监控: 系统监控工具
备份: 自动化备份系统
```

## 环境变量配置

### 核心配置

创建 `server/.env` 文件：

```bash
# 数据库配置
DATABASE_URL=sqlite:./data/marketplace.db
DATABASE_MAX_CONNECTIONS=10
DATABASE_POOL_TIMEOUT=30
DATABASE_BUSY_TIMEOUT=5000              # SQLite忙超时时间(毫秒)

# JWT认证配置
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production-environment
JWT_ACCESS_TOKEN_EXPIRES_IN=3600      # 1小时
JWT_REFRESH_TOKEN_EXPIRES_IN=604800   # 7天

# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
SERVER_WORKERS=4                      # 工作线程数

# 存储配置
STORAGE_UPLOAD_PATH=./uploads
STORAGE_MAX_FILE_SIZE=104857600       # 100MB
STORAGE_CLEANUP_INTERVAL=86400        # 24小时

# SMTP邮件配置
SMTP_ENABLED=true
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_NAME=GeekTools Plugin Marketplace
SMTP_FROM_EMAIL=noreply@your-domain.com

# 日志配置
RUST_LOG=info                         # debug,info,warn,error
RUST_BACKTRACE=1
LOG_FILE_PATH=./logs/app.log
LOG_MAX_FILE_SIZE=10MB
LOG_MAX_FILES=5

# 安全配置
CORS_ORIGINS=http://localhost:8080,https://your-domain.com
RATE_LIMIT_REQUESTS=100               # 每分钟请求数
RATE_LIMIT_WINDOW=60                  # 限制窗口(秒)
SESSION_TIMEOUT=3600                  # 会话超时(秒)

# 功能开关
FEATURE_REGISTRATION=true             # 启用用户注册
FEATURE_EMAIL_VERIFICATION=true       # 启用邮箱验证
FEATURE_ADMIN_PANEL=true              # 启用管理面板
FEATURE_METRICS=true                  # 启用指标收集

# SQLite特定配置
SQLITE_BUSY_TIMEOUT=5000              # SQLite忙超时时间(毫秒)
SQLITE_WAL_MODE=true                  # 启用WAL模式
SQLITE_CACHE_SIZE=64000               # 缓存大小(KB)
SQLITE_MMAP_SIZE=268435456            # 内存映射大小(字节)

# 缓存配置
REDIS_URL=redis://localhost:6379     # 可选Redis缓存
CACHE_TTL=300                         # 缓存过期时间(秒)

# 监控配置
METRICS_ENABLED=true                  # 启用指标
METRICS_PORT=9090                     # 指标端口
HEALTH_CHECK_INTERVAL=30              # 健康检查间隔(秒)

# 备份配置
BACKUP_ENABLED=true                   # 启用自动备份
BACKUP_SCHEDULE=0 2 * * *            # 每天凌晨2点备份(cron格式)
BACKUP_RETENTION_DAYS=30              # 备份保留天数
BACKUP_STORAGE_PATH=./backups         # 备份存储路径
```

### 环境特定配置

#### 开发环境 (`.env.development`)
```bash
# 开发环境配置
RUST_LOG=debug
DATABASE_URL=sqlite:./data/marketplace_dev.db
SMTP_ENABLED=false                    # 开发环境禁用邮件
CORS_ORIGINS=http://localhost:8080,http://127.0.0.1:8080
FEATURE_EMAIL_VERIFICATION=false     # 开发环境跳过邮箱验证
STORAGE_UPLOAD_PATH=./dev_uploads
```

#### 测试环境 (`.env.test`)
```bash
# 测试环境配置
RUST_LOG=warn
DATABASE_URL=sqlite:./data/marketplace_test.db
SMTP_ENABLED=false
FEATURE_REGISTRATION=true
FEATURE_EMAIL_VERIFICATION=false
STORAGE_UPLOAD_PATH=./test_uploads
```

#### 生产环境 (`.env.production`)
```bash
# 生产环境配置
RUST_LOG=info
DATABASE_URL=sqlite:/opt/geektools/data/marketplace_prod.db
SMTP_ENABLED=true
SMTP_HOST=smtp.example.com
SMTP_USERNAME=noreply@example.com
CORS_ORIGINS=https://plugins.example.com
SERVER_WORKERS=8
RATE_LIMIT_REQUESTS=200
BACKUP_ENABLED=true
METRICS_ENABLED=true
```

## 安全配置

### 1. 数据库安全

```bash
# 创建数据库目录并设置权限
sudo mkdir -p /opt/geektools/data
sudo chown geektools:geektools /opt/geektools/data
sudo chmod 750 /opt/geektools/data

# 设置数据库文件权限
sudo chmod 640 /opt/geektools/data/marketplace.db
sudo chown geektools:geektools /opt/geektools/data/marketplace.db

# 确保只有应用用户可以访问数据库
sudo chgrp geektools /opt/geektools/data/marketplace.db
sudo chmod g+rw /opt/geektools/data/marketplace.db
sudo chmod o-rwx /opt/geektools/data/marketplace.db
```

### 2. 文件系统安全

```bash
# 创建专用用户
sudo useradd -r -s /bin/false geektools

# 设置目录权限
sudo mkdir -p /opt/geektools/{data,uploads,logs,backups}
sudo chown -R geektools:geektools /opt/geektools
sudo chmod 750 /opt/geektools
sudo chmod 750 /opt/geektools/data
sudo chmod 755 /opt/geektools/uploads
sudo chmod 750 /opt/geektools/logs
sudo chmod 750 /opt/geektools/backups
```

### 3. 网络安全

```bash
# 防火墙配置
sudo ufw allow 22/tcp          # SSH
sudo ufw allow 80/tcp          # HTTP
sudo ufw allow 443/tcp         # HTTPS
sudo ufw allow 3000/tcp        # 应用端口
sudo ufw enable

# 禁用不必要的服务
sudo systemctl disable telnet
sudo systemctl disable ftp
sudo systemctl disable rsh
```

### 4. SSL/TLS配置

生成SSL证书：

```bash
# 使用Let's Encrypt
sudo certbot --nginx -d your-domain.com

# 或使用自签名证书(开发用)
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

Nginx SSL配置：

```nginx
server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    
    # SSL配置
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers off;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
    
    # 安全头
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# HTTP重定向到HTTPS
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}
```

## 监控和日志

### 1. 日志配置

在 `server/src/main.rs` 中配置日志：

```rust
use tracing::{info, warn, error};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

fn init_logging() {
    let log_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(log_filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
    
    info!("Logging initialized");
}
```

### 2. 系统监控

创建监控脚本 `scripts/monitor.sh`：

```bash
#!/bin/bash

# 系统监控脚本
LOG_FILE="/var/log/geektools/monitor.log"
ALERT_EMAIL="admin@example.com"

# 检查服务状态
check_service_status() {
    if ! systemctl is-active --quiet geektools; then
        echo "$(date): GeekTools service is down!" | tee -a $LOG_FILE
        # 发送告警邮件
        echo "GeekTools service is down!" | mail -s "Service Alert" $ALERT_EMAIL
        # 尝试重启服务
        systemctl restart geektools
    fi
}

# 检查数据库连接
check_database() {
    if ! sqlite3 /opt/geektools/data/marketplace.db "SELECT 1;" > /dev/null 2>&1; then
        echo "$(date): Database is not accessible!" | tee -a $LOG_FILE
        echo "SQLite database connection failed!" | mail -s "Database Alert" $ALERT_EMAIL
    fi
    
    # 检查数据库文件权限
    if [ ! -r /opt/geektools/data/marketplace.db ]; then
        echo "$(date): Database file is not readable!" | tee -a $LOG_FILE
        echo "Database file permission issue!" | mail -s "Database Alert" $ALERT_EMAIL
    fi
}

# 检查磁盘空间
check_disk_space() {
    USAGE=$(df /opt/geektools | awk 'NR==2 {print $5}' | sed 's/%//')
    if [ $USAGE -gt 85 ]; then
        echo "$(date): Disk usage is at ${USAGE}%!" | tee -a $LOG_FILE
        echo "Disk usage warning: ${USAGE}%" | mail -s "Disk Space Alert" $ALERT_EMAIL
    fi
}

# 检查内存使用
check_memory() {
    MEMORY_USAGE=$(free | awk 'NR==2{printf "%.0f", $3/$2*100}')
    if [ $MEMORY_USAGE -gt 90 ]; then
        echo "$(date): Memory usage is at ${MEMORY_USAGE}%!" | tee -a $LOG_FILE
        echo "High memory usage: ${MEMORY_USAGE}%" | mail -s "Memory Alert" $ALERT_EMAIL
    fi
}

# 检查日志错误
check_error_logs() {
    ERROR_COUNT=$(grep -c "ERROR" /opt/geektools/logs/app.log 2>/dev/null || echo "0")
    if [ $ERROR_COUNT -gt 10 ]; then
        echo "$(date): High error count: $ERROR_COUNT" | tee -a $LOG_FILE
        tail -20 /opt/geektools/logs/app.log | mail -s "Error Log Alert" $ALERT_EMAIL
    fi
}

# 执行所有检查
main() {
    echo "$(date): Starting system monitoring..." | tee -a $LOG_FILE
    
    check_service_status
    check_database
    check_disk_space
    check_memory
    check_error_logs
    
    echo "$(date): Monitoring completed." | tee -a $LOG_FILE
}

main
```

设置定时任务：

```bash
# 添加到crontab
crontab -e

# 每5分钟检查一次
*/5 * * * * /opt/geektools/scripts/monitor.sh

# 每天凌晨进行完整系统检查
0 2 * * * /opt/geektools/scripts/full_system_check.sh
```

### 3. 性能监控

使用Prometheus和Grafana进行性能监控：

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'geektools'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 5s
    metrics_path: /metrics

  - job_name: 'node'
    static_configs:
      - targets: ['localhost:9100']
    
  - job_name: 'sqlite_exporter'
    static_configs:
      - targets: ['localhost:9191']
    scrape_interval: 30s
```

## 备份策略

### 1. 数据库备份

创建数据库备份脚本：

```bash
#!/bin/bash
# backup_database.sh

BACKUP_DIR="/opt/geektools/backups"
DB_FILE="/opt/geektools/data/marketplace.db"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/db_backup_$DATE.db"

# 创建备份目录
mkdir -p $BACKUP_DIR

# 方法1: 使用SQLite内置备份命令 (推荐)
sqlite3 $DB_FILE ".backup $BACKUP_FILE"

# 方法2: 文件复制备份 (需要确保数据库不在使用中)
# cp $DB_FILE $BACKUP_FILE

# 检查备份是否成功
if [ $? -eq 0 ]; then
    echo "$(date): Database backup successful: $BACKUP_FILE"
    
    # 压缩备份文件
    gzip $BACKUP_FILE
    COMPRESSED_BACKUP="${BACKUP_FILE}.gz"
    
    # 删除30天前的备份
    find $BACKUP_DIR -name "db_backup_*.db.gz" -mtime +30 -delete
    
    # 验证备份文件完整性
    if ! gzip -t $COMPRESSED_BACKUP; then
        echo "$(date): Backup file is corrupted: $COMPRESSED_BACKUP"
        exit 1
    fi
    
    # 验证SQLite数据库完整性
    if gunzip -c $COMPRESSED_BACKUP | sqlite3 ":memory:" "PRAGMA integrity_check;" > /dev/null 2>&1; then
        echo "$(date): Backup integrity check passed"
    else
        echo "$(date): Backup integrity check failed!"
        exit 1
    fi
    
    # 记录备份大小
    BACKUP_SIZE=$(du -h $COMPRESSED_BACKUP | cut -f1)
    echo "$(date): Backup size: $BACKUP_SIZE"
    
else
    echo "$(date): Database backup failed!"
    exit 1
fi
```

### 2. 文件备份

```bash
#!/bin/bash
# backup_files.sh

BACKUP_DIR="/opt/geektools/backups"
SOURCE_DIRS="/opt/geektools/uploads /opt/geektools/config"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/files_backup_$DATE.tar.gz"

# 创建文件备份
tar -czf $BACKUP_FILE $SOURCE_DIRS

if [ $? -eq 0 ]; then
    echo "$(date): File backup successful: $BACKUP_FILE"
    
    # 清理旧备份
    find $BACKUP_DIR -name "files_backup_*.tar.gz" -mtime +30 -delete
else
    echo "$(date): File backup failed!"
    exit 1
fi
```

### 3. 自动化备份

创建systemd定时器：

```ini
# /etc/systemd/system/geektools-backup.service
[Unit]
Description=GeekTools Backup Service
Wants=geektools-backup.timer

[Service]
Type=oneshot
User=geektools
ExecStart=/opt/geektools/scripts/backup_database.sh
ExecStartPost=/opt/geektools/scripts/backup_files.sh

[Install]
WantedBy=multi-user.target
```

```ini
# /etc/systemd/system/geektools-backup.timer
[Unit]
Description=GeekTools Backup Timer
Requires=geektools-backup.service

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

启用定时器：
```bash
sudo systemctl enable geektools-backup.timer
sudo systemctl start geektools-backup.timer
```

## 故障排除

### 1. 常见问题诊断

#### 服务无法启动
```bash
# 检查服务状态
systemctl status geektools

# 查看错误日志
journalctl -u geektools -f

# 检查端口占用
ss -tulpn | grep :3000

# 检查文件权限
ls -la /opt/geektools/
```

#### 数据库连接失败
```bash
# 测试数据库连接
sqlite3 /opt/geektools/data/marketplace.db "SELECT 1;"

# 检查数据库文件状态
ls -la /opt/geektools/data/marketplace.db

# 检查数据库完整性
sqlite3 /opt/geektools/data/marketplace.db "PRAGMA integrity_check;"

# 检查数据库是否被锁定
sqlite3 /opt/geektools/data/marketplace.db "PRAGMA lock_status;"

# 验证连接字符串
echo $DATABASE_URL

# 检查数据库文件权限
stat /opt/geektools/data/marketplace.db
```

#### 上传功能异常
```bash
# 检查上传目录权限
ls -la /opt/geektools/uploads/

# 检查磁盘空间
df -h /opt/geektools/

# 查看上传相关日志
grep -i "upload" /opt/geektools/logs/app.log
```

### 2. 性能问题排查

```bash
# 检查系统资源使用
top
htop
iotop

# 检查数据库性能
# 连接到数据库
sqlite3 /opt/geektools/data/marketplace.db

-- 查看数据库统计信息
.dbinfo

-- 检查表信息和大小
SELECT 
    name as table_name,
    sql as create_statement
FROM sqlite_master 
WHERE type = 'table' 
ORDER BY name;

-- 查看数据库页面统计
PRAGMA page_count;
PRAGMA page_size;
PRAGMA freelist_count;

-- 查看索引使用情况
PRAGMA index_list('plugins');
PRAGMA index_info('idx_plugins_downloads');

-- 分析查询计划
EXPLAIN QUERY PLAN SELECT * FROM plugins WHERE status = 'active';

-- 检查数据库完整性和统计
PRAGMA integrity_check;
PRAGMA quick_check;
PRAGMA optimize;
```

### 3. 日志分析

```bash
# 分析错误日志
grep -i "error" /opt/geektools/logs/app.log | tail -20

# 统计API请求量
grep "GET\|POST\|PUT\|DELETE" /opt/geektools/logs/app.log | \
    awk '{print $1}' | sort | uniq -c | sort -nr

# 查看慢请求
grep -E "took [0-9]{3,}ms" /opt/geektools/logs/app.log

# 分析用户活动
grep "user_id" /opt/geektools/logs/app.log | \
    awk '{print $3}' | sort | uniq -c | sort -nr | head -10
```

## 扩展和维护

### 1. 水平扩展

当单个服务器无法满足需求时，可以进行水平扩展：

```bash
# 负载均衡器配置 (nginx)
upstream geektools_backend {
    server 192.168.1.10:3000 weight=3;
    server 192.168.1.11:3000 weight=3;
    server 192.168.1.12:3000 weight=2;
}

server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://geektools_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

### 2. 数据库优化

```sql
-- 创建索引优化查询性能
CREATE INDEX IF NOT EXISTS idx_plugins_downloads ON plugins(downloads DESC);
CREATE INDEX IF NOT EXISTS idx_plugins_created_at ON plugins(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_plugins_status ON plugins(status) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- 配置SQLite性能参数
PRAGMA journal_mode = WAL;              -- 使用WAL模式提高并发性能
PRAGMA synchronous = NORMAL;            -- 平衡性能和安全性
PRAGMA cache_size = -64000;             -- 设置缓存大小为64MB
PRAGMA temp_store = MEMORY;             -- 临时表存储在内存中
PRAGMA mmap_size = 268435456;           -- 使用256MB内存映射

-- 定期维护数据库
-- 每周执行
VACUUM;                                 -- 重组数据库，回收空间
PRAGMA optimize;                        -- 更新查询优化器统计信息

-- 每月执行
REINDEX;                                -- 重建所有索引

-- 检查和优化
PRAGMA integrity_check;                 -- 检查数据库完整性
PRAGMA quick_check;                     -- 快速完整性检查
```

### 3. 定期维护任务

创建维护脚本：

```bash
#!/bin/bash
# maintenance.sh

echo "$(date): Starting maintenance tasks..."

# 清理临时文件
find /tmp -name "geektools_*" -mtime +7 -delete

# 清理过期会话
sqlite3 /opt/geektools/data/marketplace.db "DELETE FROM user_sessions WHERE expires_at < datetime('now');"

# 清理过期验证码
sqlite3 /opt/geektools/data/marketplace.db "DELETE FROM verification_codes WHERE expires_at < datetime('now');"

# 压缩旧日志
find /opt/geektools/logs -name "*.log" -mtime +7 -exec gzip {} \;

# 清理旧日志压缩文件
find /opt/geektools/logs -name "*.log.gz" -mtime +30 -delete

# 优化数据库性能
sqlite3 /opt/geektools/data/marketplace.db "PRAGMA optimize; VACUUM;"

echo "$(date): Maintenance tasks completed."
```

通过这个部署指南，可以安全、可靠地将GeekTools插件市场部署到生产环境中。