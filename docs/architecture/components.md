# 组件设计详解

## 后端组件架构

### 1. 应用入口 (main.rs)

```rust
// 应用启动和配置
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志系统
    // 加载配置文件
    // 建立数据库连接池
    // 配置路由和中间件
    // 启动HTTP服务器
}
```

**职责说明**:
- 应用程序入口点和生命周期管理
- 全局配置和环境变量加载
- 数据库连接池初始化和管理
- HTTP服务器启动和优雅关闭
- 中间件栈配置 (CORS、日志、压缩等)

### 2. 处理器层 (handlers/)

#### 2.1 认证处理器 (auth.rs)

```rust
pub struct AuthHandler {
    // JWT配置
    // 邮件服务配置
    // 数据库连接池
}

impl AuthHandler {
    pub async fn register() -> Result<Json<AuthResponse>>
    pub async fn login() -> Result<Json<AuthResponse>>
    pub async fn send_verification_code() -> Result<StatusCode>
    pub async fn verify_code() -> Result<Json<AuthResponse>>
    pub async fn refresh_token() -> Result<Json<TokenResponse>>
}
```

**功能特性**:
- 用户注册和登录逻辑
- JWT Token生成和验证
- 邮箱验证码发送和验证
- Token刷新机制
- 密码安全处理 (bcrypt哈希)

#### 2.2 插件处理器 (plugins.rs)

```rust
pub struct PluginHandler {
    // 存储服务
    // 数据库连接池
    // 文件处理配置
}

impl PluginHandler {
    pub async fn list_plugins() -> Result<Json<PluginListResponse>>
    pub async fn get_plugin() -> Result<Json<PluginDetailResponse>>
    pub async fn upload_plugin() -> Result<Json<PluginResponse>>
    pub async fn download_plugin() -> Result<Response<Body>>
    pub async fn rate_plugin() -> Result<Json<RatingResponse>>
}
```

**功能特性**:
- 插件列表查询和分页
- 插件详情获取
- 插件文件上传和验证
- 插件下载和统计
- 插件评分和评论管理

#### 2.3 搜索处理器 (search.rs)

```rust
pub struct SearchHandler {
    // 搜索配置
    // 数据库连接池
}

impl SearchHandler {
    pub async fn search_plugins() -> Result<Json<SearchResponse>>
    pub async fn get_suggestions() -> Result<Json<SuggestionResponse>>
    pub async fn get_popular_tags() -> Result<Json<TagResponse>>
}
```

**功能特性**:
- 全文搜索和模糊匹配
- 搜索建议和自动补全
- 热门标签统计
- 高级搜索过滤器

#### 2.4 管理处理器 (admin.rs)

```rust
pub struct AdminHandler {
    // 管理员权限验证
    // 数据库连接池
}

impl AdminHandler {
    pub async fn get_dashboard() -> Result<Json<DashboardResponse>>
    pub async fn list_users() -> Result<Json<UserListResponse>>
    pub async fn manage_plugin() -> Result<Json<PluginResponse>>
    pub async fn execute_sql() -> Result<Json<SqlResponse>>
}
```

**功能特性**:
- 系统仪表板数据
- 用户管理和权限控制
- 插件审核和状态管理
- SQL查询执行 (仅超级管理员)

#### 2.5 健康检查处理器 (health.rs)

```rust
pub struct HealthHandler;

impl HealthHandler {
    pub async fn health_check() -> Result<Json<HealthResponse>>
    pub async fn readiness_check() -> Result<Json<ReadinessResponse>>
    pub async fn metrics() -> Result<String>
}
```

**功能特性**:
- 服务健康状态检查
- 就绪状态验证
- 性能指标收集和暴露

### 3. 服务层 (services/)

#### 3.1 认证服务 (auth.rs)

```rust
pub struct AuthService {
    jwt_secret: String,
    db_pool: PgPool,
    smtp_service: SmtpService,
}

impl AuthService {
    pub async fn create_user() -> Result<User>
    pub async fn authenticate_user() -> Result<User>
    pub async fn generate_tokens() -> Result<(String, String)>
    pub async fn verify_token() -> Result<Claims>
    pub async fn send_verification_code() -> Result<()>
    pub async fn verify_code() -> Result<bool>
}
```

**核心功能**:
- 用户身份验证和授权
- JWT Token生命周期管理
- 邮箱验证流程控制
- 密码强度验证和加密
- 用户会话管理

#### 3.2 插件服务 (plugin.rs)

```rust
pub struct PluginService {
    db_pool: PgPool,
    storage_service: StorageService,
}

impl PluginService {
    pub async fn create_plugin() -> Result<Plugin>
    pub async fn update_plugin() -> Result<Plugin>
    pub async fn get_plugin_by_id() -> Result<Option<Plugin>>
    pub async fn search_plugins() -> Result<Vec<Plugin>>
    pub async fn increment_downloads() -> Result<()>
    pub async fn calculate_rating() -> Result<f64>
}
```

**核心功能**:
- 插件CRUD操作
- 插件版本管理
- 插件搜索和过滤
- 下载统计和排行
- 评分计算和更新

#### 3.3 存储服务 (storage.rs)

```rust
pub struct StorageService {
    upload_path: PathBuf,
    max_file_size: u64,
}

impl StorageService {
    pub async fn store_plugin() -> Result<StoredFile>
    pub async fn get_plugin_file() -> Result<File>
    pub async fn delete_plugin() -> Result<()>
    pub async fn validate_file() -> Result<PluginMetadata>
    pub async fn cleanup_temp_files() -> Result<()>
}
```

**核心功能**:
- 文件上传和存储管理
- 文件格式验证和安全检查
- 临时文件清理机制
- 文件完整性校验
- 存储空间管理

#### 3.4 邮件服务 (smtp.rs)

```rust
pub struct SmtpService {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from_address: String,
}

impl SmtpService {
    pub async fn send_verification_code() -> Result<()>
    pub async fn send_welcome_email() -> Result<()>
    pub async fn send_notification() -> Result<()>
}
```

**核心功能**:
- 邮件模板管理
- 异步邮件发送
- 发送状态追踪
- 邮件队列管理
- SMTP连接池优化

#### 3.5 管理服务 (admin.rs)

```rust
pub struct AdminService {
    db_pool: PgPool,
}

impl AdminService {
    pub async fn get_system_stats() -> Result<SystemStats>
    pub async fn get_user_activities() -> Result<Vec<UserActivity>>
    pub async fn moderate_plugin() -> Result<Plugin>
    pub async fn execute_raw_sql() -> Result<SqlResult>
}
```

**核心功能**:
- 系统统计数据收集
- 用户行为分析
- 内容审核和管理
- 数据库维护工具
- 系统监控和告警

### 4. 中间件层 (middleware/)

#### 4.1 认证中间件 (auth.rs)

```rust
pub struct AuthMiddleware;

impl AuthMiddleware {
    pub fn require_auth() -> middleware
    pub fn optional_auth() -> middleware
    pub fn require_admin() -> middleware
    pub fn extract_user() -> Result<User>
}
```

**功能实现**:
- JWT Token验证和解析
- 用户权限检查
- 请求上下文注入
- 认证失败处理

#### 4.2 限流中间件 (rate_limit.rs)

```rust
pub struct RateLimitMiddleware {
    governor: Arc<DefaultDirectRateLimiter>,
}

impl RateLimitMiddleware {
    pub fn new() -> Self
    pub async fn check_rate_limit() -> Result<()>
}
```

**功能实现**:
- 基于IP的请求频率限制
- 动态限流策略调整
- 限流状态存储
- 限流异常处理

### 5. 数据模型层 (models/)

#### 5.1 用户模型 (user.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub is_active: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 100))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}
```

#### 5.2 插件模型 (plugin.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: String,
    pub current_version: String,
    pub downloads: i32,
    pub rating: BigDecimal,
    pub status: PluginStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Type, Serialize, Deserialize)]
#[sqlx(type_name = "plugin_status", rename_all = "lowercase")]
pub enum PluginStatus {
    Active,
    Deprecated,
    Banned,
}
```

#### 5.3 评分模型 (rating.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PluginRating {
    pub id: i32,
    pub plugin_id: String,
    pub user_id: i32,
    pub rating: i32,
    pub review: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 6. 工具层 (utils/)

#### 6.1 配置管理 (config.rs)

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub smtp: SmtpConfig,
    pub storage: StorageConfig,
}

impl Config {
    pub fn from_env() -> Result<Self>
    pub fn from_file() -> Result<Self>
    pub fn validate() -> Result<()>
}
```

#### 6.2 验证工具 (validation.rs)

```rust
pub struct ValidationUtils;

impl ValidationUtils {
    pub fn validate_email(email: &str) -> bool
    pub fn validate_plugin_id(id: &str) -> bool
    pub fn validate_version(version: &str) -> bool
    pub fn sanitize_input(input: &str) -> String
}
```

## 前端组件架构

### 1. 主应用 (app.js)

```javascript
class PluginMarketApp {
    constructor() {
        this.apiClient = new ApiClient();
        this.authManager = new AuthManager();
        this.pluginManager = new PluginManager();
        this.searchManager = new SearchManager();
    }

    init() {
        // 初始化应用
        // 设置事件监听器
        // 加载初始数据
    }
}
```

### 2. API客户端 (app.js)

```javascript
class ApiClient {
    constructor() {
        this.baseUrl = CONFIG.API_BASE_URL;
        this.defaultHeaders = {
            'Content-Type': 'application/json'
        };
    }

    async request(method, url, data = null) {
        // 统一的API请求处理
        // 自动添加认证头
        // 错误处理和重试
    }
}
```

### 3. 认证管理 (app.js)

```javascript
class AuthManager {
    constructor() {
        this.token = localStorage.getItem('auth_token');
        this.refreshToken = localStorage.getItem('refresh_token');
    }

    async login(email, verificationCode) {
        // 登录逻辑
        // Token存储
        // 用户状态更新
    }

    async refreshAccessToken() {
        // Token刷新
        // 自动重新认证
    }
}
```

### 4. 插件管理 (app.js)

```javascript
class PluginManager {
    constructor() {
        this.plugins = [];
        this.currentPage = 1;
        this.pageSize = 12;
    }

    async loadPlugins(page = 1) {
        // 加载插件列表
        // 分页处理
        // UI更新
    }

    async uploadPlugin(file) {
        // 文件上传
        // 进度显示
        // 结果处理
    }
}
```

### 5. 搜索管理 (app.js)

```javascript
class SearchManager {
    constructor() {
        this.searchQuery = "";
        this.filters = {};
        this.suggestions = [];
    }

    async search(query, filters = {}) {
        // 搜索请求
        // 结果过滤
        // 历史记录
    }

    async getSuggestions(query) {
        // 搜索建议
        // 防抖处理
        // 缓存优化
    }
}
```

## 管理后台组件 (admin.js)

### 1. 管理应用 (admin.js)

```javascript
class AdminApp {
    constructor() {
        this.apiClient = new ApiClient();
        this.userManager = new UserManager();
        this.pluginManager = new AdminPluginManager();
        this.systemManager = new SystemManager();
    }
}
```

### 2. 用户管理 (admin.js)

```javascript
class UserManager {
    async loadUsers(page = 1) {
        // 用户列表加载
        // 分页和搜索
    }

    async updateUserStatus(userId, status) {
        // 用户状态更新
        // 权限管理
    }
}
```

### 3. 系统管理 (admin.js)

```javascript
class SystemManager {
    async executeSql(query) {
        // SQL查询执行
        // 结果展示
        // 安全验证
    }

    async getSystemStats() {
        // 系统统计
        // 图表展示
    }
}
```

## 组件间通信机制

### 1. 后端组件通信

- **依赖注入**: 通过构造函数注入服务依赖
- **异步消息**: 使用Tokio channels进行异步通信
- **共享状态**: Arc<RwLock<T>>管理共享数据
- **事件驱动**: 自定义事件系统处理业务逻辑

### 2. 前后端通信

- **RESTful API**: 标准HTTP方法和状态码
- **JSON格式**: 统一的数据交换格式  
- **认证机制**: JWT Bearer Token认证
- **错误处理**: 统一的错误响应格式

### 3. 前端组件通信

- **发布订阅**: 自定义事件系统进行组件通信
- **状态管理**: 全局状态对象管理应用状态
- **本地存储**: localStorage持久化用户数据
- **URL路由**: 基于hash的客户端路由

这种组件化架构设计确保了代码的模块化、可测试性和可维护性，同时支持系统的横向和纵向扩展。