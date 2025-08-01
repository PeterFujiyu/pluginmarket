# 前端开发指南

## 开发环境搭建

### 1. 环境要求

```bash
# 必需工具
- Python 3.7+ (用于开发代理服务器)
- 现代浏览器 (Chrome 88+, Firefox 85+, Safari 14+)
- 文本编辑器 (VS Code, WebStorm 等)

# 可选工具
- Live Server 扩展 (VS Code)
- Browser Sync
- HTTP服务器工具
```

### 2. 本地开发服务器

#### 方法一：使用Python代理服务器（推荐）

```bash
# 启动开发代理服务器
python3 proxy_server.py

# 访问地址
http://localhost:8080
```

**代理服务器功能**：
- 自动处理CORS跨域问题
- 代理API请求到后端服务器
- 提供静态文件服务
- 支持热重载

#### 方法二：使用其他HTTP服务器

```bash
# Python内置服务器
python3 -m http.server 8080

# Node.js服务器（需安装http-server）
npx http-server -p 8080 -c-1

# PHP内置服务器
php -S localhost:8080
```

### 3. 开发配置

修改 `config.js` 中的开发配置：

```javascript
window.GeekToolsConfig = {
    // 开发环境API配置
    apiBaseUrl: '/api/v1',  // 使用代理时的相对路径
    
    frontend: {
        // 开发模式配置
        debug: true,  // 启用调试模式
        pageSize: 10, // 减少分页大小便于测试
        searchDebounceDelay: 150, // 更快的搜索响应
    }
};
```

## 项目结构解析

### 1. 文件组织

```
frontend/
├── index.html          # 主应用页面
├── admin.html          # 管理后台页面
├── app.js             # 主应用逻辑
├── admin.js           # 管理后台逻辑
├── config.js          # 配置管理
├── proxy_server.py    # 开发代理服务器
├── assets/            # 静态资源
│   ├── images/        # 图片资源
│   ├── icons/         # 图标文件
│   └── fonts/         # 字体文件（如果有）
└── styles/            # 自定义样式
    └── custom.css     # 覆盖样式
```

### 2. 依赖管理

项目使用CDN方式加载外部依赖：

```html
<!-- Tailwind CSS -->
<script src="https://cdn.tailwindcss.com"></script>

<!-- Font Awesome -->
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css">
```

**注意事项**：
- 生产环境建议下载到本地以提高加载速度
- 定期检查CDN版本更新
- 考虑使用Webpack等构建工具管理依赖

## 核心开发概念

### 1. 模块化架构

```javascript
// 主应用类
class PluginMarketplace {
    constructor() {
        this.initializeConfig();    // 配置初始化
        this.initializeState();     // 状态初始化
        this.bindEventHandlers();   // 事件绑定
        this.setupRouting();        // 路由设置
    }
}

// 管理面板类
class AdminPanel {
    constructor() {
        this.authenticateUser();    // 用户认证
        this.initializeTabs();      // 标签页初始化
        this.loadDashboard();       // 加载仪表板
    }
}

// 工具类
class MessageSystem {
    // 消息提示系统
}

class HttpClient {
    // HTTP请求封装
}
```

### 2. 状态管理

```javascript
class StateManager {
    constructor() {
        this.state = {
            // 用户状态
            user: {
                isAuthenticated: false,
                profile: null,
                permissions: []
            },
            
            // 应用状态
            app: {
                currentPage: 1,
                searchQuery: '',
                selectedCategory: '',
                sortOrder: 'downloads'
            },
            
            // UI状态
            ui: {
                isLoading: false,
                modals: {
                    login: false,
                    upload: false
                },
                notifications: []
            }
        };
    }
    
    // 状态更新方法
    updateState(path, value) {
        // 使用点表示法更新嵌套状态
        this.setState(path, value);
        this.notifySubscribers(path, value);
    }
    
    // 状态订阅
    subscribe(path, callback) {
        // 监听状态变化
    }
}
```

### 3. 事件处理系统

```javascript
class EventBus {
    constructor() {
        this.events = {};
    }
    
    on(event, callback) {
        if (!this.events[event]) {
            this.events[event] = [];
        }
        this.events[event].push(callback);
    }
    
    emit(event, data) {
        if (this.events[event]) {
            this.events[event].forEach(callback => callback(data));
        }
    }
    
    off(event, callback) {
        if (this.events[event]) {
            this.events[event] = this.events[event].filter(cb => cb !== callback);
        }
    }
}

// 全局事件总线
const eventBus = new EventBus();

// 使用示例
eventBus.on('plugin:uploaded', (plugin) => {
    console.log('Plugin uploaded:', plugin);
    // 刷新插件列表
    app.loadPlugins();
});

eventBus.on('user:login', (user) => {
    console.log('User logged in:', user);
    // 更新UI状态
    app.updateAuthUI();
});
```

## API集成开发

### 1. HTTP客户端封装

```javascript
class ApiClient {
    constructor(baseURL) {
        this.baseURL = baseURL;
        this.defaultHeaders = {
            'Content-Type': 'application/json'
        };
    }
    
    // 认证请求
    async authenticatedRequest(url, options = {}) {
        const token = localStorage.getItem('auth_token');
        const headers = {
            ...this.defaultHeaders,
            ...(token && { 'Authorization': `Bearer ${token}` }),
            ...options.headers
        };
        
        const response = await fetch(`${this.baseURL}${url}`, {
            ...options,
            headers
        });
        
        // 处理认证失败
        if (response.status === 401) {
            this.handleAuthError();
            throw new Error('Authentication required');
        }
        
        return response;
    }
    
    // 处理认证错误
    handleAuthError() {
        localStorage.removeItem('auth_token');
        localStorage.removeItem('current_user');
        eventBus.emit('auth:expired');
    }
    
    // GET请求
    async get(url, params = {}) {
        const searchParams = new URLSearchParams(params);
        const queryString = searchParams.toString();
        const fullUrl = queryString ? `${url}?${queryString}` : url;
        
        const response = await this.authenticatedRequest(fullUrl);
        return this.handleResponse(response);
    }
    
    // POST请求
    async post(url, data = {}) {
        const response = await this.authenticatedRequest(url, {
            method: 'POST',
            body: JSON.stringify(data)
        });
        return this.handleResponse(response);
    }
    
    // 响应处理
    async handleResponse(response) {
        const data = await response.json();
        
        if (!response.ok) {
            throw new Error(data.error || 'Request failed');
        }
        
        return data;
    }
}

// 全局API客户端
const api = new ApiClient('/api/v1');
```

### 2. API服务层

```javascript
// 插件服务
class PluginService {
    static async getPlugins(params = {}) {
        return await api.get('/plugins', {
            page: params.page || 1,
            limit: params.limit || 20,
            search: params.search || '',
            sort: params.sort || 'downloads',
            order: params.order || 'desc'
        });
    }
    
    static async getPlugin(id) {
        return await api.get(`/plugins/${id}`);
    }
    
    static async uploadPlugin(formData) {
        // 文件上传需要特殊处理
        const token = localStorage.getItem('auth_token');
        const response = await fetch('/api/v1/plugins/upload', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            },
            body: formData  // 不设置Content-Type，让浏览器自动设置
        });
        
        return await response.json();
    }
    
    static async downloadPlugin(id) {
        const token = localStorage.getItem('auth_token');
        const response = await fetch(`/api/v1/plugins/${id}/download`, {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        
        if (!response.ok) {
            throw new Error('Download failed');
        }
        
        // 处理文件下载
        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `plugin-${id}.tar.gz`;
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
        document.body.removeChild(a);
    }
}

// 认证服务
class AuthService {
    static async sendVerificationCode(email) {
        return await api.post('/auth/send-code', { email });
    }
    
    static async verifyCode(email, code) {
        const response = await api.post('/auth/verify-code', { email, code });
        
        if (response.success) {
            // 保存认证信息
            localStorage.setItem('auth_token', response.data.access_token);
            localStorage.setItem('current_user', JSON.stringify(response.data.user));
            
            // 触发登录事件
            eventBus.emit('user:login', response.data.user);
        }
        
        return response;
    }
    
    static async refreshToken() {
        const refreshToken = localStorage.getItem('refresh_token');
        if (!refreshToken) {
            throw new Error('No refresh token available');
        }
        
        const response = await api.post('/auth/refresh', { 
            refresh_token: refreshToken 
        });
        
        if (response.success) {
            localStorage.setItem('auth_token', response.data.access_token);
        }
        
        return response;
    }
    
    static logout() {
        localStorage.removeItem('auth_token');
        localStorage.removeItem('refresh_token');
        localStorage.removeItem('current_user');
        eventBus.emit('user:logout');
    }
}
```

## 用户界面开发

### 1. 组件化开发

```javascript
// 基础组件类
class Component {
    constructor(element) {
        this.element = element;
        this.state = {};
        this.initialize();
    }
    
    initialize() {
        this.bindEvents();
        this.render();
    }
    
    setState(newState) {
        this.state = { ...this.state, ...newState };
        this.render();
    }
    
    bindEvents() {
        // 子类实现
    }
    
    render() {
        // 子类实现
    }
}

// 插件卡片组件
class PluginCard extends Component {
    constructor(element, plugin) {
        super(element);
        this.plugin = plugin;
    }
    
    render() {
        this.element.innerHTML = `
            <div class="plugin-card bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow duration-300">
                <div class="p-6">
                    <div class="flex items-start justify-between mb-3">
                        <h3 class="text-xl font-semibold text-claude-text">${this.escapeHtml(this.plugin.name)}</h3>
                        <span class="status-badge ${this.getStatusClass()}">${this.plugin.status}</span>
                    </div>
                    
                    <p class="text-gray-600 mb-4">${this.escapeHtml(this.plugin.description)}</p>
                    
                    <div class="flex items-center justify-between">
                        <div class="plugin-stats">
                            <span class="stat-item">
                                <i class="fas fa-star text-yellow-500"></i>
                                ${this.plugin.rating}
                            </span>
                            <span class="stat-item">
                                <i class="fas fa-download text-blue-500"></i>
                                ${this.formatNumber(this.plugin.downloads)}
                            </span>
                        </div>
                        
                        <div class="plugin-actions">
                            <button class="btn-download" data-plugin-id="${this.plugin.id}">
                                <i class="fas fa-download"></i> 下载
                            </button>
                            <button class="btn-details" data-plugin-id="${this.plugin.id}">
                                详情
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        `;
    }
    
    bindEvents() {
        this.element.querySelector('.btn-download').addEventListener('click', (e) => {
            this.handleDownload(e.target.dataset.pluginId);
        });
        
        this.element.querySelector('.btn-details').addEventListener('click', (e) => {
            this.handleShowDetails(e.target.dataset.pluginId);
        });
    }
    
    async handleDownload(pluginId) {
        try {
            await PluginService.downloadPlugin(pluginId);
            eventBus.emit('plugin:downloaded', { id: pluginId });
        } catch (error) {
            eventBus.emit('error', { message: '下载失败: ' + error.message });
        }
    }
    
    handleShowDetails(pluginId) {
        eventBus.emit('plugin:showDetails', { id: pluginId });
    }
}

// 搜索组件
class SearchComponent extends Component {
    constructor(element) {
        super(element);
        this.debounceTimer = null;
    }
    
    render() {
        this.element.innerHTML = `
            <div class="search-container">
                <div class="search-input-group">
                    <input type="text" 
                           id="searchInput" 
                           placeholder="搜索插件..." 
                           class="search-input">
                    <i class="fas fa-search search-icon"></i>
                </div>
                
                <div class="search-filters">
                    <select id="sortSelect" class="filter-select">
                        <option value="downloads">按下载量排序</option>
                        <option value="rating">按评分排序</option>
                        <option value="created_at">按创建时间排序</option>
                    </select>
                    
                    <button id="advancedFiltersBtn" class="filter-button">
                        <i class="fas fa-filter"></i> 高级筛选
                    </button>
                </div>
            </div>
        `;
    }
    
    bindEvents() {
        const searchInput = this.element.querySelector('#searchInput');
        const sortSelect = this.element.querySelector('#sortSelect');
        
        searchInput.addEventListener('input', (e) => {
            this.debounceSearch(e.target.value);
        });
        
        sortSelect.addEventListener('change', (e) => {
            eventBus.emit('search:sortChanged', { sort: e.target.value });
        });
    }
    
    debounceSearch(query) {
        clearTimeout(this.debounceTimer);
        this.debounceTimer = setTimeout(() => {
            eventBus.emit('search:queryChanged', { query });
        }, 300);
    }
}
```

### 2. 响应式处理

```javascript
class ResponsiveManager {
    constructor() {
        this.breakpoints = {
            mobile: '(max-width: 767px)',
            tablet: '(min-width: 768px) and (max-width: 1023px)',
            desktop: '(min-width: 1024px)'
        };
        
        this.mediaQueries = {};
        this.initialize();
    }
    
    initialize() {
        Object.keys(this.breakpoints).forEach(breakpoint => {
            const mq = window.matchMedia(this.breakpoints[breakpoint]);
            this.mediaQueries[breakpoint] = mq;
            
            mq.addListener((e) => {
                if (e.matches) {
                    this.onBreakpointChange(breakpoint);
                }
            });
            
            // 初始检查
            if (mq.matches) {
                this.onBreakpointChange(breakpoint);
            }
        });
    }
    
    onBreakpointChange(breakpoint) {
        eventBus.emit('responsive:breakpointChanged', { breakpoint });
        
        switch (breakpoint) {
            case 'mobile':
                this.handleMobileLayout();
                break;
            case 'tablet':
                this.handleTabletLayout();
                break;
            case 'desktop':
                this.handleDesktopLayout();
                break;
        }
    }
    
    handleMobileLayout() {
        // 移动端布局调整
        document.body.classList.add('mobile');
        document.body.classList.remove('tablet', 'desktop');
        
        // 调整插件网格
        const grid = document.getElementById('pluginsGrid');
        if (grid) {
            grid.className = 'grid grid-cols-1 gap-4';
        }
        
        // 隐藏某些桌面功能
        document.querySelectorAll('.desktop-only').forEach(el => {
            el.style.display = 'none';
        });
    }
    
    handleTabletLayout() {
        document.body.classList.add('tablet');
        document.body.classList.remove('mobile', 'desktop');
        
        const grid = document.getElementById('pluginsGrid');
        if (grid) {
            grid.className = 'grid grid-cols-2 gap-6';
        }
    }
    
    handleDesktopLayout() {
        document.body.classList.add('desktop');
        document.body.classList.remove('mobile', 'tablet');
        
        const grid = document.getElementById('pluginsGrid');
        if (grid) {
            grid.className = 'grid grid-cols-3 xl:grid-cols-4 gap-6';
        }
        
        // 显示桌面功能
        document.querySelectorAll('.desktop-only').forEach(el => {
            el.style.display = '';
        });
    }
    
    isMobile() {
        return this.mediaQueries.mobile.matches;
    }
    
    isTablet() {
        return this.mediaQueries.tablet.matches;
    }
    
    isDesktop() {
        return this.mediaQueries.desktop.matches;
    }
}

// 初始化响应式管理器
const responsiveManager = new ResponsiveManager();
```

## 性能优化

### 1. 代码分割和懒加载

```javascript
// 动态导入组件
class ComponentLoader {
    static async loadComponent(componentName) {
        try {
            const module = await import(`./components/${componentName}.js`);
            return module.default;
        } catch (error) {
            console.error(`Failed to load component ${componentName}:`, error);
            throw error;
        }
    }
}

// 懒加载示例
class LazyLoadManager {
    constructor() {
        this.observers = new Map();
        this.initialize();
    }
    
    initialize() {
        // 图片懒加载
        this.setupImageLazyLoading();
        
        // 组件懒加载
        this.setupComponentLazyLoading();
    }
    
    setupImageLazyLoading() {
        const imageObserver = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    const img = entry.target;
                    img.src = img.dataset.src;
                    img.classList.remove('lazy');
                    imageObserver.unobserve(img);
                }
            });
        });
        
        document.querySelectorAll('img[data-src]').forEach(img => {
            imageObserver.observe(img);
        });
    }
    
    setupComponentLazyLoading() {
        const componentObserver = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    this.loadComponent(entry.target);
                    componentObserver.unobserve(entry.target);
                }
            });
        });
        
        document.querySelectorAll('[data-lazy-component]').forEach(el => {
            componentObserver.observe(el);
        });
    }
    
    async loadComponent(element) {
        const componentName = element.dataset.lazyComponent;
        try {
            const Component = await ComponentLoader.loadComponent(componentName);
            new Component(element);
        } catch (error) {
            element.innerHTML = '<div class="error">组件加载失败</div>';
        }
    }
}
```

### 2. 缓存策略

```javascript
class CacheManager {
    constructor() {
        this.cache = new Map();
        this.ttl = new Map(); // Time to live
        this.defaultTTL = 5 * 60 * 1000; // 5分钟
    }
    
    set(key, value, ttl = this.defaultTTL) {
        this.cache.set(key, value);
        this.ttl.set(key, Date.now() + ttl);
    }
    
    get(key) {
        if (!this.cache.has(key)) {
            return null;
        }
        
        const expiry = this.ttl.get(key);
        if (Date.now() > expiry) {
            this.cache.delete(key);
            this.ttl.delete(key);
            return null;
        }
        
        return this.cache.get(key);
    }
    
    clear() {
        this.cache.clear();
        this.ttl.clear();
    }
    
    // 清理过期缓存
    cleanup() {
        const now = Date.now();
        for (const [key, expiry] of this.ttl.entries()) {
            if (now > expiry) {
                this.cache.delete(key);
                this.ttl.delete(key);
            }
        }
    }
}

// 全局缓存实例
const cacheManager = new CacheManager();

// 定期清理过期缓存
setInterval(() => {
    cacheManager.cleanup();
}, 60000); // 每分钟清理一次

// 缓存装饰器
function cached(ttl = 5 * 60 * 1000) {
    return function(target, propertyName, descriptor) {
        const method = descriptor.value;
        
        descriptor.value = async function(...args) {
            const cacheKey = `${propertyName}_${JSON.stringify(args)}`;
            
            // 尝试从缓存获取
            const cached = cacheManager.get(cacheKey);
            if (cached) {
                return cached;
            }
            
            // 执行原方法
            const result = await method.apply(this, args);
            
            // 缓存结果
            cacheManager.set(cacheKey, result, ttl);
            
            return result;
        };
        
        return descriptor;
    };
}

// 使用缓存装饰器
class PluginService {
    @cached(10 * 60 * 1000) // 10分钟缓存
    static async getPlugins(params) {
        return await api.get('/plugins', params);
    }
}
```

### 3. 防抖和节流

```javascript
// 防抖工具函数
function debounce(func, wait, immediate = false) {
    let timeout;
    
    return function executedFunction(...args) {
        const later = () => {
            timeout = null;
            if (!immediate) func.apply(this, args);
        };
        
        const callNow = immediate && !timeout;
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
        
        if (callNow) func.apply(this, args);
    };
}

// 节流工具函数
function throttle(func, limit) {
    let inThrottle;
    
    return function executedFunction(...args) {
        if (!inThrottle) {
            func.apply(this, args);
            inThrottle = true;
            setTimeout(() => inThrottle = false, limit);
        }
    };
}

// 使用示例
class SearchHandler {
    constructor() {
        this.handleSearch = debounce(this.performSearch, 300);
        this.handleScroll = throttle(this.loadMorePlugins, 100);
    }
    
    performSearch(query) {
        // 执行搜索逻辑
    }
    
    loadMorePlugins() {
        // 加载更多插件
    }
}
```

## 测试和调试

### 1. 调试工具

```javascript
class DebugManager {
    constructor() {
        this.enabled = window.GeekToolsConfig?.frontend?.debug || false;
        this.logs = [];
        
        if (this.enabled) {
            this.setupDebugTools();
        }
    }
    
    setupDebugTools() {
        // 全局调试对象
        window.DEBUG = {
            app: window.app,
            api: api,
            cache: cacheManager,
            events: eventBus,
            logs: this.logs,
            
            // 调试方法
            clearCache: () => cacheManager.clear(),
            dumpState: () => console.log('App State:', window.app?.state),
            toggleDebug: () => this.toggle()
        };
        
        // 拦截console.log
        this.interceptConsole();
        
        // 添加调试面板
        this.addDebugPanel();
    }
    
    interceptConsole() {
        const originalLog = console.log;
        console.log = (...args) => {
            this.logs.push({
                timestamp: new Date().toISOString(),
                type: 'log',
                args: args
            });
            originalLog.apply(console, args);
        };
    }
    
    addDebugPanel() {
        const panel = document.createElement('div');
        panel.id = 'debugPanel';
        panel.style.cssText = `
            position: fixed;
            top: 10px;
            right: 10px;
            width: 300px;
            background: rgba(0,0,0,0.8);
            color: white;
            padding: 10px;
            border-radius: 5px;
            font-family: monospace;
            font-size: 12px;
            z-index: 9999;
            max-height: 400px;
            overflow-y: auto;
        `;
        
        panel.innerHTML = `
            <div style="margin-bottom: 10px;">
                <strong>Debug Panel</strong>
                <button onclick="document.getElementById('debugPanel').style.display='none'" 
                        style="float: right; background: red; color: white; border: none; padding: 2px 5px;">×</button>
            </div>
            <div id="debugContent">
                <div>App loaded: ${new Date().toLocaleTimeString()}</div>
                <div>Debug mode: ON</div>
            </div>
        `;
        
        document.body.appendChild(panel);
        
        // 每秒更新调试信息
        setInterval(() => {
            this.updateDebugPanel();
        }, 1000);
    }
    
    updateDebugPanel() {
        const content = document.getElementById('debugContent');
        if (!content) return;
        
        content.innerHTML = `
            <div>Time: ${new Date().toLocaleTimeString()}</div>
            <div>Cache entries: ${cacheManager.cache.size}</div>
            <div>Event listeners: ${Object.keys(eventBus.events).length}</div>
            <div>Memory: ${this.getMemoryUsage()}</div>
            <div>Logs: ${this.logs.length}</div>
        `;
    }
    
    getMemoryUsage() {
        if (performance.memory) {
            const used = Math.round(performance.memory.usedJSHeapSize / 1024 / 1024);
            return `${used}MB`;
        }
        return 'N/A';
    }
    
    toggle() {
        this.enabled = !this.enabled;
        const panel = document.getElementById('debugPanel');
        if (panel) {
            panel.style.display = this.enabled ? 'block' : 'none';
        }
    }
}

// 初始化调试管理器
const debugManager = new DebugManager();
```

### 2. 单元测试

```javascript
// 简单的测试框架
class TestFramework {
    constructor() {
        this.tests = [];
        this.results = [];
    }
    
    describe(name, fn) {
        console.group(`Testing: ${name}`);
        fn();
        console.groupEnd();
    }
    
    it(description, testFn) {
        try {
            testFn();
            console.log(`✅ ${description}`);
            this.results.push({ description, passed: true });
        } catch (error) {
            console.error(`❌ ${description}`, error);
            this.results.push({ description, passed: false, error });
        }
    }
    
    expect(actual) {
        return {
            toBe: (expected) => {
                if (actual !== expected) {
                    throw new Error(`Expected ${expected}, got ${actual}`);
                }
            },
            toEqual: (expected) => {
                if (JSON.stringify(actual) !== JSON.stringify(expected)) {
                    throw new Error(`Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
                }
            },
            toBeTruthy: () => {
                if (!actual) {
                    throw new Error(`Expected truthy value, got ${actual}`);
                }
            },
            toBeFalsy: () => {
                if (actual) {
                    throw new Error(`Expected falsy value, got ${actual}`);
                }
            }
        };
    }
    
    run() {
        const passed = this.results.filter(r => r.passed).length;
        const total = this.results.length;
        console.log(`\nTest Results: ${passed}/${total} passed`);
        
        if (passed === total) {
            console.log('🎉 All tests passed!');
        } else {
            console.log('❌ Some tests failed');
        }
    }
}

// 测试示例
if (window.GeekToolsConfig?.frontend?.debug) {
    const test = new TestFramework();
    
    test.describe('PluginService', () => {
        test.it('should format numbers correctly', () => {
            const service = new PluginService();
            test.expect(service.formatNumber(1000)).toBe('1K');
            test.expect(service.formatNumber(1500)).toBe('1.5K');
            test.expect(service.formatNumber(1000000)).toBe('1M');
        });
        
        test.it('should escape HTML correctly', () => {
            const service = new PluginService();
            test.expect(service.escapeHtml('<script>')).toBe('&lt;script&gt;');
            test.expect(service.escapeHtml('Tom & Jerry')).toBe('Tom &amp; Jerry');
        });
    });
    
    test.describe('CacheManager', () => {
        test.it('should store and retrieve values', () => {
            const cache = new CacheManager();
            cache.set('test', 'value');
            test.expect(cache.get('test')).toBe('value');
        });
        
        test.it('should expire values after TTL', (done) => {
            const cache = new CacheManager();
            cache.set('test', 'value', 100); // 100ms TTL
            
            setTimeout(() => {
                test.expect(cache.get('test')).toBe(null);
                done();
            }, 150);
        });
    });
    
    test.run();
}
```

### 3. 错误处理

```javascript
class ErrorHandler {
    constructor() {
        this.setupGlobalErrorHandling();
    }
    
    setupGlobalErrorHandling() {
        // 捕获未处理的错误
        window.addEventListener('error', (event) => {
            this.handleError({
                type: 'javascript',
                message: event.message,
                filename: event.filename,
                lineno: event.lineno,
                colno: event.colno,
                stack: event.error?.stack
            });
        });
        
        // 捕获未处理的Promise拒绝
        window.addEventListener('unhandledrejection', (event) => {
            this.handleError({
                type: 'promise',
                message: event.reason?.message || 'Unhandled promise rejection',
                stack: event.reason?.stack
            });
        });
        
        // 捕获网络错误
        window.addEventListener('offline', () => {
            this.showNetworkError('网络连接已断开');
        });
        
        window.addEventListener('online', () => {
            this.hideNetworkError();
        });
    }
    
    handleError(error) {
        console.error('Application Error:', error);
        
        // 发送错误报告（如果需要）
        this.reportError(error);
        
        // 显示用户友好的错误消息
        this.showUserError(error);
    }
    
    reportError(error) {
        // 发送到错误监控服务
        if (window.GeekToolsConfig?.frontend?.errorReporting) {
            fetch('/api/v1/errors', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    ...error,
                    url: window.location.href,
                    userAgent: navigator.userAgent,
                    timestamp: new Date().toISOString()
                })
            }).catch(() => {
                // 静默失败
            });
        }
    }
    
    showUserError(error) {
        const message = this.getUserFriendlyMessage(error);
        eventBus.emit('error', { message });
    }
    
    getUserFriendlyMessage(error) {
        switch (error.type) {
            case 'network':
                return '网络连接错误，请检查网络设置';
            case 'authentication':
                return '登录已过期，请重新登录';
            case 'permission':
                return '您没有权限执行此操作';
            case 'validation':
                return '输入数据格式不正确';
            default:
                return '系统出现错误，请稍后重试';
        }
    }
    
    showNetworkError(message) {
        const banner = document.createElement('div');
        banner.id = 'networkErrorBanner';
        banner.className = 'fixed top-0 left-0 right-0 bg-red-500 text-white text-center py-2 z-50';
        banner.textContent = message;
        document.body.appendChild(banner);
    }
    
    hideNetworkError() {
        const banner = document.getElementById('networkErrorBanner');
        if (banner) {
            banner.remove();
        }
    }
}

// 初始化错误处理器
const errorHandler = new ErrorHandler();
```

这个前端开发指南涵盖了从环境搭建到性能优化的全面内容，为开发者提供了详细的开发指导和最佳实践。