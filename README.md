# GeekTools Plugin Marketplace

完整的插件市场解决方案，包含现代化的前端界面和高性能的 Rust 后端服务。

> 📖 **快速安装指南**: 查看 [INSTALL.md](./INSTALL.md) 获取详细的安装说明

## ✨ 核心特性

### 前端界面
- 🎨 **精美设计**: 参考 Claude 官网风格，使用圆角图标和渐变效果
- 📱 **响应式布局**: 完全适配桌面端和移动端
- 🔍 **智能搜索**: 支持关键词搜索和分类筛选
- 📊 **数据统计**: 插件统计信息展示
- 🔄 **实时更新**: 动态加载和分页功能
- 📤 **文件上传**: 拖拽上传插件文件
- ⚙️ **灵活配置**: 支持自定义API基础URL和主题配置

### 后端服务
- 🚀 **高性能**: 基于 Rust + Axum 构建，内存安全且高并发
- 🔐 **安全认证**: JWT + 邮箱验证码，支持SMTP邮件发送
- 📦 **插件管理**: 完整的插件上传、下载、搜索、评分功能
- 🗄️ **数据库**: PostgreSQL支持，自动迁移和连接池
- 🔧 **管理功能**: 内置管理员面板，用户管理和系统监控
- 🐳 **容器化**: Docker和Docker Compose支持
- 📈 **监控**: 健康检查、指标统计和结构化日志

## 🏗️ 项目结构

```
plugin_server/
├── 📄 INSTALL.md           # 安装指南
├── 📄 README.md            # 项目说明
├── 📄 config.js            # 前端配置文件
├── 📄 index.html           # 主页面
├── 📄 admin.html           # 管理员面板
├── 📄 app.js               # 前端主应用
├── 📄 admin.js             # 管理员应用
├── 📄 proxy_server.py      # 代理服务器
├── 📄 docker-compose.yml   # Docker编排
└── server/                 # 后端服务
    ├── 📁 src/             # Rust源代码
    ├── 📁 migrations/      # 数据库迁移
    ├── 📄 Cargo.toml       # Rust依赖配置
    ├── 📄 .env.example     # 环境变量示例
    └── 📄 Dockerfile       # Docker镜像配置
```

## 🛠️ 技术栈

### 前端技术
- **HTML5**: 语义化标记
- **Tailwind CSS**: 响应式 CSS 框架  
- **Vanilla JavaScript**: 原生 JavaScript，无框架依赖
- **Font Awesome**: 图标库

### 后端技术
- **Rust**: 系统编程语言，内存安全
- **Axum**: 异步 Web 框架
- **SQLx**: 类型安全的 SQL 查询
- **PostgreSQL**: 生产级关系数据库
- **JWT**: 无状态身份认证
- **Lettre**: SMTP 邮件发送
- **Docker**: 容器化部署

## 🚀 快速开始

### 一键部署（推荐）

```bash
# 克隆项目
git clone https://github.com/your-repo/geektools.git
cd geektools/plugin_server

# 启动所有服务
docker-compose up -d

# 访问应用
echo "🌐 插件市场: http://localhost:8080"
echo "🔧 管理面板: http://localhost:8080/admin.html"
echo "📡 API文档: http://localhost:3000/api/v1/health"
```

### 配置选项

#### 前端配置 (`config.js`)
```javascript
window.GeekToolsConfig = {
    // API基础URL - 根据部署环境修改
    apiBaseUrl: '/api/v1',
    
    // 功能开关
    features: {
        enableRegistration: true,  // 用户注册
        enableUpload: true,        // 插件上传
        enableRating: true,        // 评分系统
        enableAdminPanel: true     // 管理面板
    },
    
    // 主题配置
    theme: {
        primaryColor: '#FF8C47',
        darkMode: false
    }
};
```

#### 后端配置 (`server/.env`)
```bash
# 数据库配置
DATABASE_URL=postgres://postgres:password@localhost/marketplace

# JWT配置
JWT_SECRET=your-super-secret-jwt-key

# SMTP邮件配置（可选）
SMTP_ENABLED=true
SMTP_HOST=smtp.gmail.com
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
```

## 功能特性

### 1. 插件浏览
- 网格布局展示插件卡片
- 显示插件名称、描述、作者、下载量、评分
- 支持分页浏览

### 2. 搜索和筛选
- 实时搜索插件名称和描述
- 按分类筛选
- 多种排序方式（下载量、评分、更新时间、名称）

### 3. 插件详情
- 模态框展示详细信息
- 版本历史和更新日志
- 包含的脚本列表
- 下载和统计信息

### 4. 插件上传
- 支持拖拽上传 .tar.gz 文件
- 文件格式和大小验证
- 上传进度提示

### 5. 响应式设计
- 移动端友好
- 自适应网格布局
- 触摸友好的交互

## 使用方法

1. **本地预览**
   ```bash
   # 在 plugin_server 目录下启动本地服务器
   python -m http.server 8080
   # 或使用 Node.js
   npx serve .
   ```

2. **访问页面**
   打开浏览器访问 `http://localhost:8080`

## 配置说明

### API 基础 URL
在 `app.js` 中修改 `baseURL` 配置：
```javascript
this.baseURL = 'https://api.geektools.dev/v1';
```

### 模拟数据
当前使用模拟数据进行演示，在实际部署时需要：
1. 替换 `getMockPlugins()` 方法为真实 API 调用
2. 实现真实的文件上传逻辑
3. 添加用户认证功能

## 样式定制

### 主题颜色
在 Tailwind 配置中定义了自定义颜色：
```javascript
colors: {
    'claude-orange': '#FF8C47',
    'claude-bg': '#F9F9F8',
    'claude-text': '#2F2F2F',
    'claude-light': '#FEFEFE',
}
```

### 字体
使用系统字体栈确保最佳显示效果：
```javascript
fontFamily: {
    'claude': ['system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
}
```

## 浏览器支持

- Chrome 60+
- Firefox 60+
- Safari 12+
- Edge 79+

## 性能优化

- 使用 CDN 加载 Tailwind CSS
- 图片懒加载
- 分页减少数据量
- 防抖搜索减少请求

## 后续开发建议

1. **用户系统**: 添加用户注册/登录功能
2. **评论系统**: 插件评论和评分功能
3. **收藏功能**: 用户收藏插件
4. **插件分析**: 下载统计和使用分析
5. **版本管理**: 更完善的版本控制
6. **安全性**: 添加 CSRF 保护和输入验证

## License

MIT License