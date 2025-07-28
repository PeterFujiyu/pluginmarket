# GeekTools Plugin Marketplace 安装指南

这是一份完整的安装指南，包含了前端和后端的部署说明。

## 🚀 快速开始

### 方法一：Docker 一键部署（推荐）

这是最简单的安装方式，适合生产环境。

```bash
# 1. 克隆项目并进入目录
git clone https://github.com/your-repo/geektools.git
cd geektools/plugin_server

# 2. 复制并配置环境变量
cp server/.env.example server/.env
# 编辑 server/.env 文件，设置数据库密码和JWT密钥

# 3. 启动所有服务
docker-compose up -d

# 4. 等待服务启动完成（约30秒）
docker-compose logs -f

# 5. 验证安装
curl http://localhost:3000/api/v1/health
```

**访问地址**：
- 插件市场前端：http://localhost:8080
- 后端API：http://localhost:3000/api/v1
- 管理员面板：http://localhost:8080/admin.html

**🔑 管理员设置**：
- 第一位注册的用户将自动获得管理员权限
- 管理员可以访问用户管理、插件管理、系统监控等功能
- 后续注册的用户为普通用户权限

### 方法二：本地开发部署

适合开发者进行代码修改和调试。

#### 步骤 1：安装依赖

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install postgresql postgresql-contrib pkg-config libssl-dev libpq-dev curl
```

**macOS:**
```bash
brew install postgresql pkg-config openssl
brew services start postgresql
```

**Windows:**
```powershell
# 使用 Chocolatey
choco install postgresql rust
```

#### 步骤 2：设置数据库

```bash
# 创建数据库
createdb marketplace

# 设置数据库URL（如果使用默认设置）
export DATABASE_URL="postgres://postgres@localhost/marketplace"
```

#### 步骤 3：配置后端

```bash
cd server/

# 复制环境配置
cp .env.example .env

# 编辑配置文件
nano .env
```

**重要配置项**：
```bash
# 数据库连接
DATABASE_URL=postgres://postgres:password@localhost/marketplace

# JWT密钥（生产环境请更改）
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production

# API基础URL（用于前端配置）
API_BASE_URL=http://localhost:3000/api/v1

# SMTP配置（可选，不配置则显示验证码）
SMTP_ENABLED=false
```

#### 步骤 4：安装Rust和依赖

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 安装数据库迁移工具
cargo install sqlx-cli --no-default-features --features postgres

# 运行数据库迁移
sqlx migrate run
```

#### 步骤 5：启动服务

```bash
# 启动后端服务
cargo run

# 新开终端启动前端
cd ..
python3 -m http.server 8080
# 或使用 Node.js
# npx serve . -p 8080
```

### 方法三：使用代理服务器（解决CORS问题）

如果遇到跨域问题，可以使用代理服务器：

```bash
# 启动后端
cd server/
cargo run

# 启动代理服务器
cd ..
python3 proxy_server.py

# 访问 http://localhost:8080
```

## ⚙️ 配置选项

### 环境变量配置

在 `server/.env` 文件中配置：

| 配置项 | 说明 | 默认值 | 必填 |
|--------|------|--------|------|
| `DATABASE_URL` | PostgreSQL连接字符串 | - | ✅ |
| `JWT_SECRET` | JWT签名密钥 | - | ✅ |
| `SERVER_HOST` | 服务器监听地址 | `0.0.0.0` | ❌ |
| `SERVER_PORT` | 服务器端口 | `3000` | ❌ |
| `API_BASE_URL` | API基础URL | `http://localhost:3000/api/v1` | ❌ |
| `SMTP_ENABLED` | 是否启用邮件发送 | `false` | ❌ |
| `SMTP_HOST` | SMTP服务器地址 | - | ❌ |
| `SMTP_PORT` | SMTP端口 | `587` | ❌ |
| `SMTP_USERNAME` | SMTP用户名 | - | ❌ |
| `SMTP_PASSWORD` | SMTP密码/应用密码 | - | ❌ |
| `SMTP_FROM_ADDRESS` | 发件人邮箱 | - | ❌ |

### 前端配置

在 `app.js` 中配置API基础URL：

```javascript
// 自动从环境变量读取，或手动配置
this.baseURL = process.env.API_BASE_URL || '/api/v1';
```

### SMTP 邮件配置

如果要启用真实邮件发送，请配置SMTP设置：

```bash
# Gmail 示例配置
SMTP_ENABLED=true
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password  # 使用应用专用密码
SMTP_FROM_ADDRESS=noreply@geektools.dev
SMTP_FROM_NAME=GeekTools Plugin Marketplace
```

**获取Gmail应用密码**：
1. 访问 Google Account Settings
2. 启用两步验证
3. 生成应用专用密码
4. 使用该密码作为 `SMTP_PASSWORD`

## 🔧 生产环境部署

### 使用 Nginx 反向代理

创建 `/etc/nginx/sites-available/geektools-marketplace`：

```nginx
server {
    listen 80;
    server_name your-domain.com;

    # 前端静态文件
    location / {
        root /path/to/geektools/plugin_server;
        index index.html;
        try_files $uri $uri/ /index.html;
    }

    # API 代理
    location /api/ {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # 文件上传大小限制
    client_max_body_size 100M;
}
```

启用站点：
```bash
sudo ln -s /etc/nginx/sites-available/geektools-marketplace /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### 使用 Systemd 服务

创建 `/etc/systemd/system/geektools-marketplace.service`：

```ini
[Unit]
Description=GeekTools Plugin Marketplace Server
After=network.target postgresql.service

[Service]
Type=simple
User=marketplace
WorkingDirectory=/opt/geektools/plugin_server/server
ExecStart=/opt/geektools/plugin_server/server/target/release/server
Restart=always
RestartSec=5
Environment=RUST_LOG=info
EnvironmentFile=/opt/geektools/plugin_server/server/.env

[Install]
WantedBy=multi-user.target
```

启用服务：
```bash
sudo systemctl daemon-reload
sudo systemctl enable geektools-marketplace
sudo systemctl start geektools-marketplace
```

### SSL/HTTPS 配置

使用 Let's Encrypt 获取免费SSL证书：

```bash
# 安装 Certbot
sudo apt-get install certbot python3-certbot-nginx

# 获取证书
sudo certbot --nginx -d your-domain.com

# 自动续期
sudo crontab -e
# 添加：0 12 * * * /usr/bin/certbot renew --quiet
```

## 🧪 验证安装

### 1. 健康检查

```bash
# 检查后端服务
curl http://localhost:3000/api/v1/health

# 期望输出：
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "1.0.0",
    "timestamp": "2025-01-27T...",
    "services": {
      "database": "healthy",
      "storage": "healthy"
    }
  }
}
```

### 2. 测试注册功能

```bash
curl -X POST http://localhost:3000/api/v1/auth/send-code \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com"}'
```

### 3. 测试前端

访问 http://localhost:8080，检查：
- [ ] 页面正常加载
- [ ] 插件列表显示
- [ ] 统计数据显示
- [ ] 搜索功能正常
- [ ] 上传功能可用

## 🛠️ 故障排除

### 常见问题

#### 1. 数据库连接失败

```bash
# 检查 PostgreSQL 是否运行
sudo systemctl status postgresql

# 检查连接
psql $DATABASE_URL -c "SELECT 1;"

# 重置数据库
dropdb marketplace
createdb marketplace
sqlx migrate run
```

#### 2. 端口被占用

```bash
# 查找占用端口的进程
sudo lsof -i :3000
sudo lsof -i :8080

# 杀死进程或修改配置
export SERVER_PORT=3001
```

#### 3. CORS 错误

确保后端 CORS 配置正确，或使用代理服务器：

```bash
# 使用代理服务器
python3 proxy_server.py
```

#### 4. 文件上传失败

```bash
# 检查上传目录权限
ls -la server/uploads/
chmod 755 server/uploads/

# 检查磁盘空间
df -h
```

#### 5. SMTP 配置问题

```bash
# 测试 SMTP 连接
telnet smtp.gmail.com 587

# 检查应用密码是否正确
# 确保启用了两步验证
```

### 日志调试

```bash
# 查看详细日志
export RUST_LOG=debug
cargo run

# Docker 日志
docker-compose logs -f app

# 系统服务日志
sudo journalctl -u geektools-marketplace -f
```

## 📈 性能优化

### 数据库优化

```sql
-- 添加索引
CREATE INDEX idx_plugins_downloads ON plugins(downloads DESC);
CREATE INDEX idx_plugins_created_at ON plugins(created_at DESC);
CREATE INDEX idx_plugins_status ON plugins(status);
```

### 缓存配置

在生产环境中考虑添加Redis缓存：

```bash
# 安装 Redis
sudo apt-get install redis-server

# 修改配置启用缓存
REDIS_URL=redis://localhost:6379
```

### 负载均衡

使用多个服务实例：

```bash
# 启动多个后端实例
SERVER_PORT=3000 cargo run &
SERVER_PORT=3001 cargo run &
SERVER_PORT=3002 cargo run &

# 配置 Nginx 负载均衡
upstream backend {
    server localhost:3000;
    server localhost:3001;
    server localhost:3002;
}
```

## 🔒 安全配置

### 生产环境安全检查

- [ ] 更改默认JWT密钥
- [ ] 使用强密码保护数据库
- [ ] 启用HTTPS/SSL
- [ ] 配置防火墙规则
- [ ] 设置文件上传限制
- [ ] 启用审计日志
- [ ] 定期备份数据库
- [ ] 更新系统补丁

### 备份配置

```bash
# 数据库备份
pg_dump marketplace > backup_$(date +%Y%m%d_%H%M%S).sql

# 文件备份
tar -czf uploads_backup_$(date +%Y%m%d_%H%M%S).tar.gz server/uploads/

# 自动备份脚本
echo "0 2 * * * /path/to/backup-script.sh" | crontab -
```

## 📞 支持与帮助

- 📚 **文档**: 查看 `plugin-marketplace-implementation.md` 获取详细技术信息
- 🐛 **问题报告**: 提交 GitHub Issues
- 💬 **社区支持**: 加入讨论群组
- 📧 **技术支持**: support@geektools.dev

---

**安装成功后，您将拥有一个功能完整的插件市场系统！** 🎉