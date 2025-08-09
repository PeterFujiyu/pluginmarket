# 生产环境部署指南

## 生产环境架构

### 1. 推荐架构

```
Internet
    |
[Load Balancer (Nginx)]
    |
[Web Servers (Multiple Instances)]
    |
[Application Servers (API)]
    |
[Database (SQLite)]
    |
[Cache Layer (Redis)]
    |
[File Storage (NFS/Object Storage)]
```

### 2. 服务器规格建议

#### 小型部署 (< 1000用户)
```
Web服务器: 2CPU, 4GB RAM, 50GB SSD
API服务器: 2CPU, 4GB RAM, 50GB SSD
应用+数据库服务器: 2CPU, 4GB RAM, 50GB SSD
负载均衡器: 1CPU, 2GB RAM, 20GB SSD
```

#### 中型部署 (1000-10000用户)
```
Web服务器: 4CPU, 8GB RAM, 100GB SSD (2台)
API服务器: 4CPU, 8GB RAM, 100GB SSD (3台)
应用+数据库服务器: 4CPU, 8GB RAM, 100GB SSD (2台)
Redis缓存: 2CPU, 4GB RAM, 50GB SSD
负载均衡器: 2CPU, 4GB RAM, 50GB SSD (2台HA)
```

#### 大型部署 (> 10000用户)
```
Web服务器: 8CPU, 16GB RAM, 200GB SSD (3台+)
API服务器: 8CPU, 16GB RAM, 200GB SSD (5台+)
应用+数据库服务器: 8CPU, 16GB RAM, 200GB SSD (3台+)
Redis缓存: 4CPU, 8GB RAM, 100GB SSD (集群)
负载均衡器: 4CPU, 8GB RAM, 100GB SSD (HA)
文件存储: 对象存储服务
```

## 系统准备

### 1. 操作系统配置

#### Ubuntu 22.04 LTS配置

```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装基础工具
sudo apt install -y curl wget git htop iotop netstat-nat \
    build-essential pkg-config libssl-dev

# 配置时区
sudo timedatectl set-timezone Asia/Shanghai

# 配置NTP时间同步
sudo apt install -y chrony
sudo systemctl enable chrony
sudo systemctl start chrony

# 配置防火墙
sudo ufw enable
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# 优化内核参数
sudo tee /etc/sysctl.d/99-geektools.conf << EOF
# 网络优化
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.tcp_keepalive_time = 600
net.ipv4.tcp_keepalive_intvl = 60
net.ipv4.tcp_keepalive_probes = 3

# 文件描述符限制
fs.file-max = 2097152
fs.nr_open = 2097152

# 内存管理
vm.swappiness = 10
vm.dirty_ratio = 15
vm.dirty_background_ratio = 5
EOF

sudo sysctl -p /etc/sysctl.d/99-geektools.conf

# 配置用户限制
sudo tee /etc/security/limits.d/99-geektools.conf << EOF
* soft nofile 65535
* hard nofile 65535
* soft nproc 65535
* hard nproc 65535
EOF
```

### 2. 用户和目录设置

```bash
# 创建应用用户
sudo useradd -r -m -s /bin/bash geektools
sudo usermod -aG sudo geektools

# 创建应用目录结构
sudo mkdir -p /opt/geektools/{app,uploads,logs,config,backups,scripts}
sudo chown -R geektools:geektools /opt/geektools
sudo chmod 755 /opt/geektools
sudo chmod 750 /opt/geektools/{logs,config,backups}

# 设置日志轮转
sudo tee /etc/logrotate.d/geektools << EOF
/opt/geektools/logs/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 geektools geektools
    postrotate
        systemctl reload geektools || true
    endscript
}
EOF
```

## 数据库部署

### 1. SQLite配置

```bash
# 安装SQLite (通常系统已预装)
sudo apt install -y sqlite3

# 创建数据库目录
sudo mkdir -p /opt/geektools/database
sudo chown geektools:geektools /opt/geektools/database
sudo chmod 750 /opt/geektools/database

# 数据库文件将由应用自动创建
# 位置: /opt/geektools/database/marketplace.db
```

### 2. SQLite生产优化

创建SQLite优化配置脚本 `/opt/geektools/scripts/sqlite_optimize.sql`：

```sql
-- 性能优化设置
PRAGMA journal_mode = WAL;              -- 启用WAL模式，支持并发读
PRAGMA synchronous = NORMAL;            -- 平衡性能和安全性
PRAGMA cache_size = -64000;             -- 64MB内存缓存
PRAGMA temp_store = MEMORY;             -- 临时表存储在内存
PRAGMA mmap_size = 134217728;           -- 128MB内存映射
PRAGMA optimize;                        -- 优化数据库结构

-- 定期维护设置
PRAGMA auto_vacuum = INCREMENTAL;       -- 启用增量真空
PRAGMA incremental_vacuum(1000);        -- 增量真空页数

-- 连接和锁超时
PRAGMA busy_timeout = 30000;            -- 30秒锁超时
```

创建数据库维护脚本 `/opt/geektools/scripts/sqlite_maintenance.sh`：

```bash
#!/bin/bash
# SQLite数据库维护脚本

DB_PATH="/opt/geektools/database/marketplace.db"
BACKUP_DIR="/opt/geektools/backups"
LOG_FILE="/opt/geektools/logs/sqlite_maintenance.log"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a $LOG_FILE
}

# 创建备份目录
mkdir -p $BACKUP_DIR

# 数据库完整性检查
log "Running integrity check..."
if sqlite3 $DB_PATH "PRAGMA integrity_check;" | grep -q "ok"; then
    log "Integrity check passed"
else
    log "ERROR: Integrity check failed"
    exit 1
fi

# 分析统计信息
log "Analyzing database statistics..."
sqlite3 $DB_PATH "PRAGMA analyze;"

# 优化数据库
log "Optimizing database..."
sqlite3 $DB_PATH "PRAGMA optimize;"

# 增量真空清理
log "Running incremental vacuum..."
sqlite3 $DB_PATH "PRAGMA incremental_vacuum(1000);"

log "Database maintenance completed"
```

设置定期维护：
```bash
# 添加到crontab
sudo crontab -e
# 每天凌晨3点运行维护
0 3 * * * /opt/geektools/scripts/sqlite_maintenance.sh
```

### 3. SQLite高可用配置

由于SQLite是嵌入式数据库，高可用主要通过以下方式实现：

#### 数据同步策略

```bash
# 创建数据同步脚本
sudo tee /opt/geektools/scripts/sqlite_sync.sh << 'EOF'
#!/bin/bash

DB_PATH="/opt/geektools/database/marketplace.db"
REMOTE_SERVERS=("server2" "server3")  # 其他服务器列表
SYNC_USER="geektools"
LOG_FILE="/opt/geektools/logs/sqlite_sync.log"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a $LOG_FILE
}

# WAL文件同步到其他服务器
sync_to_replicas() {
    for server in "${REMOTE_SERVERS[@]}"; do
        log "Syncing database to $server"
        
        # 使用rsync同步数据库文件
        rsync -avz --delete \
            $DB_PATH \
            $DB_PATH-wal \
            $DB_PATH-shm \
            $SYNC_USER@$server:/opt/geektools/database/
        
        if [ $? -eq 0 ]; then
            log "Successfully synced to $server"
        else
            log "ERROR: Failed to sync to $server"
        fi
    done
}

# 检查数据库是否在WAL模式
if sqlite3 $DB_PATH "PRAGMA journal_mode;" | grep -q "wal"; then
    log "Database is in WAL mode, performing sync"
    sync_to_replicas
else
    log "Database is not in WAL mode, skipping sync"
fi
EOF

chmod +x /opt/geektools/scripts/sqlite_sync.sh
```

#### 读负载均衡配置

```bash
# 应用配置支持多个SQLite读副本
# 在.env文件中配置
DATABASE_URL=sqlite:///opt/geektools/database/marketplace.db
DATABASE_READ_REPLICAS=sqlite:///opt/geektools/database/marketplace_replica1.db,sqlite:///opt/geektools/database/marketplace_replica2.db
DATABASE_MAX_CONNECTIONS=20
```

#### 故障切换脚本

```bash
# 创建故障切换脚本
sudo tee /opt/geektools/scripts/sqlite_failover.sh << 'EOF'
#!/bin/bash

PRIMARY_DB="/opt/geektools/database/marketplace.db"
BACKUP_DB="/opt/geektools/database/marketplace_backup.db"
APP_CONFIG="/opt/geektools/config/.env"
LOG_FILE="/opt/geektools/logs/sqlite_failover.log"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a $LOG_FILE
}

# 检查主数据库状态
if sqlite3 $PRIMARY_DB "SELECT 1;" >/dev/null 2>&1; then
    log "Primary database is healthy"
    exit 0
fi

log "Primary database failure detected, initiating failover"

# 切换到备份数据库
if [ -f "$BACKUP_DB" ]; then
    log "Switching to backup database"
    
    # 更新配置文件
    sed -i "s|DATABASE_URL=sqlite://.*|DATABASE_URL=sqlite://$BACKUP_DB|" $APP_CONFIG
    
    # 重启应用服务
    systemctl restart geektools
    
    log "Failover completed"
else
    log "ERROR: Backup database not found"
    exit 1
fi
EOF

chmod +x /opt/geektools/scripts/sqlite_failover.sh
```

## 应用部署

### 1. Rust环境安装

```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable

# 验证安装
rustc --version
cargo --version
```

### 2. 应用编译部署

```bash
# 切换到应用用户
sudo su - geektools

# 克隆代码
git clone <repository-url> /opt/geektools/app
cd /opt/geektools/app

# 编译应用
cd server
cargo build --release

# 复制可执行文件
cp target/release/server /opt/geektools/app/geektools-server
chmod +x /opt/geektools/app/geektools-server

# 复制配置文件
cp -r migrations /opt/geektools/app/
```

### 3. 配置文件设置

创建 `/opt/geektools/config/.env`：

```bash
# 数据库配置
DATABASE_URL=sqlite:///opt/geektools/database/marketplace.db
DATABASE_MAX_CONNECTIONS=20
DATABASE_POOL_TIMEOUT=30
DATABASE_ENABLE_WAL=true

# JWT配置
JWT_SECRET=your-super-secret-jwt-key-for-production-environment-change-this
JWT_ACCESS_TOKEN_EXPIRES_IN=3600
JWT_REFRESH_TOKEN_EXPIRES_IN=604800

# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
SERVER_WORKERS=8

# 存储配置
STORAGE_UPLOAD_PATH=/opt/geektools/uploads
STORAGE_MAX_FILE_SIZE=104857600

# SMTP配置
SMTP_ENABLED=true
SMTP_HOST=smtp.your-domain.com
SMTP_PORT=587
SMTP_USERNAME=noreply@your-domain.com
SMTP_PASSWORD=your-smtp-password
SMTP_FROM_NAME=GeekTools Plugin Marketplace
SMTP_FROM_EMAIL=noreply@your-domain.com

# 日志配置
RUST_LOG=info
LOG_FILE_PATH=/opt/geektools/logs/app.log

# 安全配置
CORS_ORIGINS=https://your-domain.com
RATE_LIMIT_REQUESTS=200
RATE_LIMIT_WINDOW=60

# 功能配置
FEATURE_REGISTRATION=true
FEATURE_EMAIL_VERIFICATION=true
FEATURE_ADMIN_PANEL=true
FEATURE_METRICS=true

# 缓存配置
REDIS_URL=redis://localhost:6379
CACHE_TTL=300

# 监控配置
METRICS_ENABLED=true
METRICS_PORT=9090
```

### 4. Systemd服务配置

创建 `/etc/systemd/system/geektools.service`：

```ini
[Unit]
Description=GeekTools Plugin Marketplace
After=network.target redis.service
Wants=redis.service

[Service]
Type=simple
User=geektools
Group=geektools
WorkingDirectory=/opt/geektools/app
ExecStart=/opt/geektools/app/geektools-server
ExecReload=/bin/kill -HUP $MAINPID
KillMode=mixed
Restart=always
RestartSec=5
Environment=RUST_LOG=info
EnvironmentFile=/opt/geektools/config/.env

# 安全配置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/geektools/uploads /opt/geektools/logs
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE

# 资源限制
LimitNOFILE=65535
LimitNPROC=65535
MemoryMax=2G
CPUQuota=200%

[Install]
WantedBy=multi-user.target
```

启用并启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable geektools
sudo systemctl start geektools
sudo systemctl status geektools
```

## 负载均衡配置

### 1. Nginx安装配置

```bash
# 安装Nginx
sudo apt install -y nginx

# 启用并启动
sudo systemctl enable nginx
sudo systemctl start nginx
```

### 2. Nginx配置

创建 `/etc/nginx/sites-available/geektools`：

```nginx
# 上游服务器配置
upstream geektools_backend {
    least_conn;
    server 127.0.0.1:3000 weight=3 max_fails=3 fail_timeout=30s;
    server 10.0.1.10:3000 weight=3 max_fails=3 fail_timeout=30s;
    server 10.0.1.11:3000 weight=2 max_fails=3 fail_timeout=30s;
    
    # 健康检查
    keepalive 32;
}

# 限制请求速率
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=login:10m rate=1r/s;

# 缓存配置
proxy_cache_path /var/cache/nginx/geektools levels=1:2 keys_zone=geektools_cache:10m 
                 max_size=1g inactive=60m use_temp_path=off;

# HTTP重定向到HTTPS
server {
    listen 80;
    server_name your-domain.com www.your-domain.com;
    return 301 https://$server_name$request_uri;
}

# 主HTTPS服务器
server {
    listen 443 ssl http2;
    server_name your-domain.com www.your-domain.com;
    
    # SSL配置
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:50m;
    ssl_session_tickets off;
    
    # 现代SSL配置
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    
    # HSTS
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
    
    # 安全头
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com https://cdnjs.cloudflare.com; style-src 'self' 'unsafe-inline' https://cdnjs.cloudflare.com; img-src 'self' data: https:; font-src 'self' https://cdnjs.cloudflare.com;" always;
    
    # 日志配置
    access_log /var/log/nginx/geektools_access.log combined;
    error_log /var/log/nginx/geektools_error.log;
    
    # 根目录
    root /opt/geektools/app;
    index index.html;
    
    # 客户端上传限制
    client_max_body_size 100M;
    client_body_timeout 60s;
    client_header_timeout 60s;
    
    # Gzip压缩
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types
        text/plain
        text/css
        text/xml
        text/javascript
        application/json
        application/javascript
        application/xml+rss
        application/atom+xml
        image/svg+xml;
    
    # API代理配置
    location /api/ {
        # 速率限制
        limit_req zone=api burst=20 nodelay;
        
        # 代理设置
        proxy_pass http://geektools_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # 超时设置
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
        # 缓冲设置
        proxy_buffering on;
        proxy_buffer_size 4k;
        proxy_buffers 8 4k;
        proxy_busy_buffers_size 8k;
        
        # 特殊路径处理
        location /api/v1/auth/login {
            limit_req zone=login burst=3 nodelay;
            proxy_pass http://geektools_backend;
        }
        
        location /api/v1/plugins/download {
            proxy_pass http://geektools_backend;
            proxy_buffering off;
            proxy_cache off;
        }
    }
    
    # 静态文件处理
    location / {
        try_files $uri $uri/ /index.html;
        
        # 缓存设置
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
            access_log off;
        }
        
        location ~* \.(html)$ {
            expires 1h;
            add_header Cache-Control "public";
        }
    }
    
    # 健康检查
    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
    
    # 隐藏敏感文件
    location ~ /\. {
        deny all;
        access_log off;
        log_not_found off;
    }
    
    location ~ ~$ {
        deny all;
        access_log off;
        log_not_found off;
    }
}
```

启用站点配置：

```bash
sudo ln -s /etc/nginx/sites-available/geektools /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### 3. SSL证书配置

使用Let's Encrypt免费SSL证书：

```bash
# 安装Certbot
sudo apt install -y certbot python3-certbot-nginx

# 获取证书
sudo certbot --nginx -d your-domain.com -d www.your-domain.com

# 设置自动续期
sudo crontab -e
# 添加以下行
0 12 * * * /usr/bin/certbot renew --quiet
```

## Redis缓存部署

### 1. Redis安装配置

```bash
# 安装Redis
sudo apt install -y redis-server

# 配置Redis
sudo tee /etc/redis/redis.conf << EOF
# 网络配置
bind 127.0.0.1
port 6379
protected-mode yes
tcp-backlog 511
timeout 0
tcp-keepalive 300

# 内存配置
maxmemory 1gb
maxmemory-policy allkeys-lru

# 持久化配置
save 900 1
save 300 10
save 60 10000
stop-writes-on-bgsave-error yes
rdbcompression yes
rdbchecksum yes
dbfilename dump.rdb
dir /var/lib/redis

# 日志配置
loglevel notice
logfile /var/log/redis/redis-server.log

# 安全配置
requirepass your_redis_password

# 性能配置
databases 16
maxclients 10000
EOF

# 启动Redis
sudo systemctl enable redis-server
sudo systemctl start redis-server
sudo systemctl status redis-server
```

## 监控系统部署

### 1. Prometheus安装

```bash
# 创建prometheus用户
sudo useradd --no-create-home --shell /bin/false prometheus

# 创建目录
sudo mkdir /etc/prometheus /var/lib/prometheus
sudo chown prometheus:prometheus /etc/prometheus /var/lib/prometheus

# 下载和安装Prometheus
cd /tmp
wget https://github.com/prometheus/prometheus/releases/download/v2.40.0/prometheus-2.40.0.linux-amd64.tar.gz
tar xzf prometheus-2.40.0.linux-amd64.tar.gz
cd prometheus-2.40.0.linux-amd64

sudo cp prometheus promtool /usr/local/bin/
sudo chown prometheus:prometheus /usr/local/bin/prometheus /usr/local/bin/promtool
sudo cp -r consoles console_libraries /etc/prometheus/
sudo chown -R prometheus:prometheus /etc/prometheus/consoles /etc/prometheus/console_libraries
```

配置Prometheus (`/etc/prometheus/prometheus.yml`)：

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "rules/*.yml"

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'geektools-api'
    static_configs:
      - targets: ['localhost:9090']  # API metrics endpoint
    scrape_interval: 10s

  - job_name: 'postgres'
    static_configs:
      - targets: ['localhost:9187']  # postgres_exporter

  - job_name: 'nginx'
    static_configs:
      - targets: ['localhost:9113']  # nginx-prometheus-exporter

  - job_name: 'node'
    static_configs:
      - targets: ['localhost:9100']  # node_exporter

  - job_name: 'redis'
    static_configs:
      - targets: ['localhost:9121']  # redis_exporter

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - localhost:9093
```

创建systemd服务文件：

```ini
# /etc/systemd/system/prometheus.service
[Unit]
Description=Prometheus
Wants=network-online.target
After=network-online.target

[Service]
User=prometheus
Group=prometheus
Type=simple
ExecStart=/usr/local/bin/prometheus \
    --config.file /etc/prometheus/prometheus.yml \
    --storage.tsdb.path /var/lib/prometheus/ \
    --web.console.templates=/etc/prometheus/consoles \
    --web.console.libraries=/etc/prometheus/console_libraries \
    --web.listen-address=0.0.0.0:9090 \
    --web.enable-lifecycle \
    --storage.tsdb.retention.time=30d

[Install]
WantedBy=multi-user.target
```

启动Prometheus：

```bash
sudo systemctl daemon-reload
sudo systemctl enable prometheus
sudo systemctl start prometheus
```

### 2. 导出器安装

#### Node Exporter

```bash
cd /tmp
wget https://github.com/prometheus/node_exporter/releases/download/v1.6.0/node_exporter-1.6.0.linux-amd64.tar.gz
tar xzf node_exporter-1.6.0.linux-amd64.tar.gz
sudo cp node_exporter-1.6.0.linux-amd64/node_exporter /usr/local/bin/
sudo useradd --no-create-home --shell /bin/false node_exporter
sudo chown node_exporter:node_exporter /usr/local/bin/node_exporter
```

#### PostgreSQL Exporter

```bash
cd /tmp
wget https://github.com/prometheus-community/postgres_exporter/releases/download/v0.13.0/postgres_exporter-0.13.0.linux-amd64.tar.gz
tar xzf postgres_exporter-0.13.0.linux-amd64.tar.gz
sudo cp postgres_exporter-0.13.0.linux-amd64/postgres_exporter /usr/local/bin/
sudo useradd --no-create-home --shell /bin/false postgres_exporter

# 配置环境变量
sudo tee /etc/default/postgres_exporter << EOF
DATA_SOURCE_NAME="postgresql://monitoring_user:monitoring_password@localhost:5432/marketplace?sslmode=disable"
EOF
```

### 3. Grafana安装

```bash
# 添加Grafana repository
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
echo "deb https://packages.grafana.com/oss/deb stable main" | sudo tee /etc/apt/sources.list.d/grafana.list

# 安装Grafana
sudo apt update
sudo apt install -y grafana

# 启动Grafana
sudo systemctl enable grafana-server
sudo systemctl start grafana-server
```

## 自动化部署脚本

### 1. 部署脚本

创建 `/opt/geektools/scripts/deploy.sh`：

```bash
#!/bin/bash
set -e

# 配置变量
APP_DIR="/opt/geektools/app"
BACKUP_DIR="/opt/geektools/backups"
LOG_FILE="/opt/geektools/logs/deploy.log"
REPO_URL="https://github.com/your-username/geektools.git"
BRANCH="${1:-main}"

# 日志函数
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a $LOG_FILE
}

# 备份当前版本
backup_current() {
    log "Creating backup of current version..."
    
    if [ -d "$APP_DIR" ]; then
        BACKUP_NAME="backup_$(date +%Y%m%d_%H%M%S)"
        cp -r $APP_DIR $BACKUP_DIR/$BACKUP_NAME
        log "Backup created: $BACKUP_DIR/$BACKUP_NAME"
    fi
}

# 下载新版本
download_new_version() {
    log "Downloading new version from $REPO_URL (branch: $BRANCH)..."
    
    TEMP_DIR="/tmp/geektools_deploy_$(date +%s)"
    git clone -b $BRANCH $REPO_URL $TEMP_DIR
    
    log "Downloaded to: $TEMP_DIR"
    echo $TEMP_DIR
}

# 编译应用
build_application() {
    local temp_dir=$1
    log "Building application..."
    
    cd $temp_dir/server
    cargo build --release
    
    if [ $? -eq 0 ]; then
        log "Build successful"
    else
        log "Build failed"
        exit 1
    fi
}

# 执行数据库迁移
run_migrations() {
    log "Running database migrations..."
    
    cd $APP_DIR
    ./migrate
    
    if [ $? -eq 0 ]; then
        log "Migrations completed successfully"
    else
        log "Migrations failed"
        exit 1
    fi
}

# 部署新版本
deploy_new_version() {
    local temp_dir=$1
    log "Deploying new version..."
    
    # 停止服务
    sudo systemctl stop geektools
    
    # 备份当前版本
    backup_current
    
    # 复制新版本
    rm -rf $APP_DIR/*
    cp -r $temp_dir/* $APP_DIR/
    
    # 设置权限
    chown -R geektools:geektools $APP_DIR
    chmod +x $APP_DIR/server/target/release/server
    
    # 复制可执行文件
    cp $APP_DIR/server/target/release/server $APP_DIR/geektools-server
    
    # 运行迁移
    run_migrations
    
    # 启动服务
    sudo systemctl start geektools
    
    # 检查服务状态
    sleep 5
    if sudo systemctl is-active --quiet geektools; then
        log "Service started successfully"
    else
        log "Service failed to start"
        exit 1
    fi
    
    # 清理临时文件
    rm -rf $temp_dir
    
    log "Deployment completed successfully"
}

# 健康检查
health_check() {
    log "Performing health check..."
    
    for i in {1..10}; do
        if curl -f http://localhost:3000/api/v1/health > /dev/null 2>&1; then
            log "Health check passed"
            return 0
        fi
        log "Health check attempt $i failed, retrying..."
        sleep 5
    done
    
    log "Health check failed after 10 attempts"
    return 1
}

# 回滚函数
rollback() {
    log "Rolling back to previous version..."
    
    LATEST_BACKUP=$(ls -t $BACKUP_DIR/backup_* | head -n1)
    if [ -n "$LATEST_BACKUP" ]; then
        sudo systemctl stop geektools
        rm -rf $APP_DIR/*
        cp -r $LATEST_BACKUP/* $APP_DIR/
        chown -R geektools:geektools $APP_DIR
        sudo systemctl start geektools
        log "Rollback completed"
    else
        log "No backup found for rollback"
        exit 1
    fi
}

# 主函数
main() {
    log "Starting deployment process..."
    
    # 下载新版本
    temp_dir=$(download_new_version)
    
    # 编译应用
    build_application $temp_dir
    
    # 部署
    deploy_new_version $temp_dir
    
    # 健康检查
    if ! health_check; then
        log "Health check failed, rolling back..."
        rollback
        exit 1
    fi
    
    log "Deployment process completed successfully"
}

# 执行主函数
main "$@"
```

### 2. 监控脚本

创建 `/opt/geektools/scripts/monitor.sh`：

```bash
#!/bin/bash

LOG_FILE="/opt/geektools/logs/monitor.log"
ALERT_EMAIL="admin@your-domain.com"
WEBHOOK_URL="https://your-webhook-url.com/alerts"  # Slack/Discord webhook

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a $LOG_FILE
}

send_alert() {
    local message="$1"
    local severity="$2"
    
    # 发送邮件
    echo "$message" | mail -s "GeekTools Alert [$severity]" $ALERT_EMAIL
    
    # 发送Webhook通知
    curl -X POST -H 'Content-type: application/json' \
        --data "{\"text\":\"GeekTools Alert [$severity]: $message\"}" \
        $WEBHOOK_URL
}

check_service() {
    if ! systemctl is-active --quiet geektools; then
        log "ERROR: GeekTools service is down"
        send_alert "GeekTools service is down" "CRITICAL"
        
        # 尝试重启
        log "Attempting to restart service..."
        systemctl restart geektools
        sleep 10
        
        if systemctl is-active --quiet geektools; then
            log "Service restarted successfully"
            send_alert "GeekTools service has been restarted" "INFO"
        else
            log "Failed to restart service"
            send_alert "Failed to restart GeekTools service" "CRITICAL"
        fi
    fi
}

check_database() {
    local db_path="/opt/geektools/database/marketplace.db"
    if ! sqlite3 $db_path "SELECT 1;" > /dev/null 2>&1; then
        log "ERROR: Database is not responding"
        send_alert "Database is not responding" "CRITICAL"
    fi
}

check_redis() {
    if ! redis-cli -a your_redis_password ping > /dev/null 2>&1; then
        log "ERROR: Redis is not responding"
        send_alert "Redis is not responding" "WARNING"
    fi
}

check_disk_space() {
    local usage=$(df /opt/geektools | awk 'NR==2 {print $5}' | sed 's/%//')
    if [ $usage -gt 85 ]; then
        log "WARNING: Disk usage is at ${usage}%"
        send_alert "Disk usage is at ${usage}%" "WARNING"
    fi
}

check_memory() {
    local usage=$(free | awk 'NR==2{printf "%.0f", $3/$2*100}')
    if [ $usage -gt 90 ]; then
        log "WARNING: Memory usage is at ${usage}%"
        send_alert "Memory usage is at ${usage}%" "WARNING"
    fi
}

check_api_health() {
    if ! curl -f http://localhost:3000/api/v1/health > /dev/null 2>&1; then
        log "ERROR: API health check failed"
        send_alert "API health check failed" "CRITICAL"
    fi
}

check_ssl_certificate() {
    local days_until_expiry=$(echo | openssl s_client -servername your-domain.com -connect your-domain.com:443 2>/dev/null | openssl x509 -noout -dates | grep notAfter | cut -d= -f2 | xargs -I {} date -d {} +%s | xargs -I {} expr \( {} - $(date +%s) \) / 86400)
    
    if [ $days_until_expiry -lt 30 ]; then
        log "WARNING: SSL certificate expires in $days_until_expiry days"
        send_alert "SSL certificate expires in $days_until_expiry days" "WARNING"
    fi
}

main() {
    log "Starting monitoring checks..."
    
    check_service
    check_database
    check_redis
    check_disk_space
    check_memory
    check_api_health
    check_ssl_certificate
    
    log "Monitoring checks completed"
}

main
```

### 3. 备份脚本

创建 `/opt/geektools/scripts/backup.sh`：

```bash
#!/bin/bash

BACKUP_DIR="/opt/geektools/backups"
DATE=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=30
LOG_FILE="/opt/geektools/logs/backup.log"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a $LOG_FILE
}

backup_database() {
    log "Starting database backup..."
    
    local backup_file="$BACKUP_DIR/database_$DATE.db.gz"
    local db_path="/opt/geektools/database/marketplace.db"
    
    # 使用SQLite backup命令创建在线备份
    sqlite3 $db_path ".backup /tmp/marketplace_backup_$DATE.db"
    
    if [ $? -eq 0 ]; then
        # 压缩备份文件
        gzip -c "/tmp/marketplace_backup_$DATE.db" > $backup_file
        rm "/tmp/marketplace_backup_$DATE.db"
        
        log "Database backup completed: $backup_file"
        
        # 验证备份文件
        if gzip -t $backup_file; then
            log "Backup file verified successfully"
        else
            log "ERROR: Backup file is corrupted"
            exit 1
        fi
    else
        log "ERROR: Database backup failed"
        exit 1
    fi
}

backup_uploads() {
    log "Starting uploads backup..."
    
    local backup_file="$BACKUP_DIR/uploads_$DATE.tar.gz"
    
    tar -czf $backup_file -C /opt/geektools uploads
    
    if [ $? -eq 0 ]; then
        log "Uploads backup completed: $backup_file"
    else
        log "ERROR: Uploads backup failed"
        exit 1
    fi
}

backup_config() {
    log "Starting config backup..."
    
    local backup_file="$BACKUP_DIR/config_$DATE.tar.gz"
    
    tar -czf $backup_file -C /opt/geektools config
    
    if [ $? -eq 0 ]; then
        log "Config backup completed: $backup_file"
    else
        log "ERROR: Config backup failed"
        exit 1
    fi
}

cleanup_old_backups() {
    log "Cleaning up old backups (older than $RETENTION_DAYS days)..."
    
    find $BACKUP_DIR -name "*.sql.gz" -mtime +$RETENTION_DAYS -delete
    find $BACKUP_DIR -name "*.tar.gz" -mtime +$RETENTION_DAYS -delete
    
    log "Old backups cleaned up"
}

sync_to_remote() {
    if [ -n "$REMOTE_BACKUP_HOST" ]; then
        log "Syncing backups to remote host..."
        
        rsync -avz --delete $BACKUP_DIR/ $REMOTE_BACKUP_HOST:/opt/backups/geektools/
        
        if [ $? -eq 0 ]; then
            log "Remote sync completed"
        else
            log "ERROR: Remote sync failed"
        fi
    fi
}

main() {
    log "Starting backup process..."
    
    mkdir -p $BACKUP_DIR
    
    backup_database
    backup_uploads
    backup_config
    cleanup_old_backups
    sync_to_remote
    
    log "Backup process completed successfully"
}

main
```

### 4. 设置定时任务

```bash
# 编辑crontab
sudo crontab -e

# 添加以下任务
# 每5分钟检查系统状态
*/5 * * * * /opt/geektools/scripts/monitor.sh

# 每天凌晨2点备份
0 2 * * * /opt/geektools/scripts/backup.sh

# 每周日凌晨1点清理日志
0 1 * * 0 find /opt/geektools/logs -name "*.log" -mtime +7 -delete

# 每月1号检查SSL证书
0 0 1 * * /opt/geektools/scripts/check_ssl.sh
```

通过这个生产环境部署指南，可以构建一个高可用、高性能、安全可靠的GeekTools插件市场生产环境。