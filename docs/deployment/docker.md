# Docker部署指南

## Docker简介

Docker部署是推荐的部署方式，它提供了以下优势：

- **环境一致性**: 开发、测试、生产环境完全一致
- **快速部署**: 一键启动所有服务
- **资源隔离**: 服务间相互隔离，不会冲突
- **易于管理**: 统一的容器管理方式
- **水平扩展**: 支持容器编排和负载均衡

## 快速开始

### 1. 前置要求

确保系统已安装：

```bash
# Docker (推荐版本20.10+)
docker --version

# Docker Compose (推荐版本2.0+)
docker-compose --version
```

安装Docker（如果未安装）：

```bash
# Ubuntu/Debian
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# CentOS/RHEL
sudo yum install -y docker docker-compose
sudo systemctl enable docker
sudo systemctl start docker

# macOS
# 下载并安装 Docker Desktop for Mac

# Windows
# 下载并安装 Docker Desktop for Windows
```

### 2. 一键启动

```bash
# 克隆项目
git clone <repository-url>
cd pluginmarket

# 启动所有服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f
```

### 3. 访问应用

```bash
# 前端应用
http://localhost:8080

# 管理后台
http://localhost:8080/admin.html

# API文档
http://localhost:3000/api/v1/health

# 数据库管理
http://localhost:8081  # pgAdmin (如果启用)
```

## Docker配置详解

### 1. Dockerfile解析

#### 后端服务Dockerfile

```dockerfile
# server/Dockerfile
# 多阶段构建，减小镜像体积

# 构建阶段
FROM rust:1.75-slim as builder

# 安装系统依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 构建应用
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# 创建应用用户
RUN useradd -r -s /bin/false geektools

# 创建必要目录
RUN mkdir -p /app/uploads /app/logs /app/config
RUN chown -R geektools:geektools /app

# 复制构建产物
COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/migrations /app/migrations

# 设置权限
RUN chmod +x /app/server

# 切换到应用用户
USER geektools

# 设置工作目录
WORKDIR /app

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:3000/api/v1/health || exit 1

# 暴露端口
EXPOSE 3000

# 启动命令
CMD ["./server"]
```

#### 前端代理Dockerfile

```dockerfile
# Dockerfile.proxy
FROM python:3.11-slim

# 安装依赖
RUN pip install --no-cache-dir flask flask-cors requests

# 设置工作目录
WORKDIR /app

# 复制前端文件
COPY . .

# 创建非root用户
RUN useradd -r -s /bin/false proxy_user
RUN chown -R proxy_user:proxy_user /app
USER proxy_user

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080 || exit 1

# 启动命令
CMD ["python", "proxy_server.py"]
```

### 2. Docker Compose配置

#### 完整的docker-compose.yml

```yaml
version: '3.8'

services:
  # SQLite数据库 (嵌入在应用中，无需单独服务)
  # 数据库文件通过卷挂载持久化

  # Redis缓存 (可选)
  redis:
    image: redis:7-alpine
    container_name: geektools_redis
    restart: unless-stopped
    command: redis-server --requirepass ${REDIS_PASSWORD}
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"
    networks:
      - geektools_network
    healthcheck:
      test: ["CMD", "redis-cli", "--raw", "incr", "ping"]
      interval: 10s
      timeout: 3s
      retries: 5

  # 后端API服务
  api:
    build:
      context: ./server
      dockerfile: Dockerfile
    container_name: geektools_api
    restart: unless-stopped
    environment:
      DATABASE_URL: sqlite:///app/database/marketplace.db
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379
      JWT_SECRET: ${JWT_SECRET}
      SMTP_ENABLED: ${SMTP_ENABLED:-false}
      SMTP_HOST: ${SMTP_HOST}
      SMTP_PORT: ${SMTP_PORT:-587}
      SMTP_USERNAME: ${SMTP_USERNAME}
      SMTP_PASSWORD: ${SMTP_PASSWORD}
      RUST_LOG: ${RUST_LOG:-info}
      SERVER_HOST: 0.0.0.0
      SERVER_PORT: 3000
      STORAGE_UPLOAD_PATH: /app/uploads
      CORS_ORIGINS: http://localhost:8080,https://${DOMAIN:-localhost}
    volumes:
      - upload_data:/app/uploads
      - log_data:/app/logs
      - database_data:/app/database
    ports:
      - "3000:3000"
    networks:
      - geektools_network
    depends_on:
      redis:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/v1/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

  # 前端代理服务
  frontend:
    build:
      context: .
      dockerfile: Dockerfile.proxy
    container_name: geektools_frontend
    restart: unless-stopped
    environment:
      API_BASE_URL: http://api:3000/api/v1
      PROXY_HOST: 0.0.0.0
      PROXY_PORT: 8080
    ports:
      - "8080:8080"
    networks:
      - geektools_network
    depends_on:
      api:
        condition: service_healthy

  # Nginx反向代理 (生产环境)
  nginx:
    image: nginx:alpine
    container_name: geektools_nginx
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./docker/nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./docker/nginx/conf.d:/etc/nginx/conf.d:ro
      - ./ssl:/etc/nginx/ssl:ro
      - log_data:/var/log/nginx
    networks:
      - geektools_network
    depends_on:
      - frontend
      - api
    profiles:
      - production

  # SQLite Web管理界面 (开发环境)
  sqlite-web:
    image: coleifer/sqlite-web
    container_name: geektools_sqlite_web
    restart: unless-stopped
    command: ["-H", "0.0.0.0", "-x", "/database/marketplace.db"]
    volumes:
      - database_data:/database
    ports:
      - "8081:8080"
    networks:
      - geektools_network
    depends_on:
      - api
    profiles:
      - development

  # Prometheus监控 (可选)
  prometheus:
    image: prom/prometheus:latest
    container_name: geektools_prometheus
    restart: unless-stopped
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=200h'
      - '--web.enable-lifecycle'
    volumes:
      - ./docker/prometheus/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    ports:
      - "9090:9090"
    networks:
      - geektools_network
    profiles:
      - monitoring

  # Grafana仪表板 (可选)
  grafana:
    image: grafana/grafana:latest
    container_name: geektools_grafana
    restart: unless-stopped
    environment:
      GF_SECURITY_ADMIN_PASSWORD: ${GRAFANA_PASSWORD}
      GF_USERS_ALLOW_SIGN_UP: false
    volumes:
      - grafana_data:/var/lib/grafana
      - ./docker/grafana/provisioning:/etc/grafana/provisioning:ro
    ports:
      - "3001:3000"
    networks:
      - geektools_network
    depends_on:
      - prometheus
    profiles:
      - monitoring

# 网络配置
networks:
  geektools_network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16

# 数据卷配置
volumes:
  database_data:
    driver: local
  redis_data:
    driver: local
  upload_data:
    driver: local
  log_data:
    driver: local
  prometheus_data:
    driver: local
  grafana_data:
    driver: local
```

### 3. 环境变量配置

#### .env文件

```bash
# 数据库配置 (SQLite嵌入式，无需密码)
# DATABASE_PATH=/app/database/marketplace.db

# Redis配置
REDIS_PASSWORD=redis_password_123

# JWT配置
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production-environment

# SMTP配置
SMTP_ENABLED=true
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password

# 域名配置
DOMAIN=your-domain.com

# 监控配置
GRAFANA_PASSWORD=grafana_password_123

# 日志级别
RUST_LOG=info
```

#### 环境特定配置

**开发环境** (`.env.development`):
```bash
POSTGRES_PASSWORD=dev_password
SMTP_ENABLED=false
RUST_LOG=debug
```

**生产环境** (`.env.production`):
```bash
POSTGRES_PASSWORD=production_secure_password
SMTP_ENABLED=true
RUST_LOG=warn
```

## 高级配置

### 1. Nginx配置

#### docker/nginx/nginx.conf

```nginx
user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;

events {
    worker_connections 1024;
    use epoll;
    multi_accept on;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # 日志格式
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    access_log /var/log/nginx/access.log main;

    # 基础配置
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    client_max_body_size 100M;

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

    # 安全头
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Referrer-Policy "strict-origin-when-cross-origin";

    # 包含站点配置
    include /etc/nginx/conf.d/*.conf;
}
```

#### docker/nginx/conf.d/default.conf

```nginx
# HTTP到HTTPS重定向
server {
    listen 80;
    server_name _;
    return 301 https://$host$request_uri;
}

# HTTPS主站
server {
    listen 443 ssl http2;
    server_name your-domain.com;

    # SSL配置
    ssl_certificate /etc/nginx/ssl/fullchain.pem;
    ssl_certificate_key /etc/nginx/ssl/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers off;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:50m;
    ssl_stapling on;
    ssl_stapling_verify on;

    # HSTS
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload";

    # API代理
    location /api/ {
        proxy_pass http://api:3000/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # 超时设置
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
        # 缓冲设置
        proxy_buffering on;
        proxy_buffer_size 4k;
        proxy_buffers 8 4k;
    }

    # 静态文件
    location / {
        proxy_pass http://frontend:8080/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # 健康检查
    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
}
```

### 2. PostgreSQL初始化

#### docker/postgres/init/01-init.sql

```sql
-- 创建扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- 创建只读用户 (用于监控)
CREATE USER monitoring_user WITH PASSWORD 'monitoring_password';
GRANT CONNECT ON DATABASE marketplace TO monitoring_user;
GRANT USAGE ON SCHEMA public TO monitoring_user;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO monitoring_user;

-- 设置默认权限
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO monitoring_user;

-- 优化配置
ALTER SYSTEM SET shared_preload_libraries = 'pg_stat_statements';
ALTER SYSTEM SET pg_stat_statements.track = 'all';
ALTER SYSTEM SET log_statement = 'all';
ALTER SYSTEM SET log_min_duration_statement = 1000;
```

### 3. 监控配置

#### docker/prometheus/prometheus.yml

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "rules/*.yml"

scrape_configs:
  - job_name: 'geektools-api'
    static_configs:
      - targets: ['api:3000']
    metrics_path: /metrics
    scrape_interval: 10s

  - job_name: 'sqlite'
    static_configs:
      - targets: ['api:9090']  # SQLite metrics through application

  - job_name: 'nginx'
    static_configs:
      - targets: ['nginx-exporter:9113']

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

## 运维操作

### 1. 常用命令

```bash
# 启动服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f [service_name]

# 重启服务
docker-compose restart [service_name]

# 停止服务
docker-compose down

# 完全清理 (包括数据)
docker-compose down -v --remove-orphans

# 重新构建镜像
docker-compose build --no-cache

# 执行数据库迁移
docker-compose exec api ./migrate

# 进入容器
docker-compose exec api bash
docker-compose exec postgres psql -U marketplace_user -d marketplace

# 查看资源使用情况
docker stats

# 清理未使用的资源
docker system prune -a
```

### 2. 数据备份与恢复

#### 数据备份脚本

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="./backups"
DATE=$(date +%Y%m%d_%H%M%S)

# 创建备份目录
mkdir -p $BACKUP_DIR

# 备份SQLite数据库
docker-compose exec -T api sqlite3 /app/database/marketplace.db ".backup stdout" | \
    gzip > $BACKUP_DIR/database_$DATE.db.gz

# 备份上传文件
docker run --rm -v geektools_upload_data:/data -v $(pwd)/$BACKUP_DIR:/backup \
    alpine tar czf /backup/uploads_$DATE.tar.gz -C /data .

# 备份配置文件
tar czf $BACKUP_DIR/config_$DATE.tar.gz docker-compose.yml .env docker/

echo "Backup completed: $DATE"
```

#### 数据恢复脚本

```bash
#!/bin/bash
# restore.sh

BACKUP_FILE=$1
if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file>"
    exit 1
fi

# 恢复SQLite数据库
if [[ $BACKUP_FILE == *"database"* ]]; then
    gunzip -c $BACKUP_FILE | \
        docker-compose exec -T api sqlite3 /app/database/marketplace.db ".restore stdin"
fi

# 恢复上传文件
if [[ $BACKUP_FILE == *"uploads"* ]]; then
    docker run --rm -v geektools_upload_data:/data -v $(pwd):/backup \
        alpine tar xzf /backup/$BACKUP_FILE -C /data
fi

echo "Restore completed"
```

### 3. 健康检查和监控

```bash
#!/bin/bash
# health_check.sh

# 检查容器状态
check_containers() {
    echo "Checking container status..."
    docker-compose ps
}

# 检查服务健康状态
check_health() {
    echo "Checking service health..."
    
    # API健康检查
    if curl -f http://localhost:3000/api/v1/health; then
        echo "✅ API service is healthy"
    else
        echo "❌ API service is unhealthy"
    fi
    
    # 前端健康检查
    if curl -f http://localhost:8080; then
        echo "✅ Frontend service is healthy"
    else
        echo "❌ Frontend service is unhealthy"
    fi
    
    # 数据库健康检查
    if docker-compose exec api sqlite3 /app/database/marketplace.db "SELECT 1;" >/dev/null 2>&1; then
        echo "✅ Database is healthy"
    else
        echo "❌ Database is unhealthy"
    fi
}

# 检查资源使用
check_resources() {
    echo "Checking resource usage..."
    docker stats --no-stream
}

# 检查日志错误
check_logs() {
    echo "Checking for errors in logs..."
    docker-compose logs --tail=100 | grep -i error
}

# 执行所有检查
main() {
    echo "=== Docker Health Check ==="
    check_containers
    echo
    check_health
    echo
    check_resources
    echo
    check_logs
}

main
```

### 4. 性能优化

#### 生产环境优化配置

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  postgres:
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '1.0'
        reservations:
          memory: 1G
          cpus: '0.5'
    command: >
      postgres
      -c max_connections=200
      -c shared_buffers=256MB
      -c effective_cache_size=1GB
      -c maintenance_work_mem=64MB
      -c checkpoint_completion_target=0.9
      -c wal_buffers=16MB
      -c default_statistics_target=100

  api:
    deploy:
      replicas: 3
      resources:
        limits:
          memory: 1G
          cpus: '0.5'
        reservations:
          memory: 512M
          cpus: '0.25'
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
    environment:
      SERVER_WORKERS: 4
      DATABASE_MAX_CONNECTIONS: 20

  redis:
    command: >
      redis-server
      --maxmemory 256mb
      --maxmemory-policy allkeys-lru
      --save 900 1
      --save 300 10
      --save 60 10000
```

使用生产配置：
```bash
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### 5. 故障排除

#### 常见问题解决

```bash
# 1. 容器启动失败
docker-compose logs [service_name]

# 2. 数据库连接失败
docker-compose exec api sqlite3 /app/database/marketplace.db

# 3. 端口冲突
sudo netstat -tulpn | grep :3000
docker-compose down
sudo pkill -f "port 3000"

# 4. 磁盘空间不足
docker system df
docker system prune -a
docker volume prune

# 5. 内存不足
docker stats
docker-compose restart

# 6. 网络问题
docker network ls
docker network inspect geektools_geektools_network
```

#### 应急恢复流程

```bash
# 应急恢复脚本
#!/bin/bash

echo "Starting emergency recovery..."

# 停止所有服务
docker-compose down

# 检查系统资源
df -h
free -h

# 清理Docker资源
docker system prune -f

# 从最新备份恢复
./restore.sh backups/$(ls -t backups/ | head -1)

# 重新启动服务
docker-compose up -d

# 验证服务状态
sleep 30
./health_check.sh

echo "Emergency recovery completed"
```

通过这个Docker部署指南，可以快速、可靠地部署GeekTools插件市场，并具备完整的运维能力。