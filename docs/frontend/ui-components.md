# 前端UI组件设计

## 概述

GeekTools 插件市场前端采用原生JavaScript + Tailwind CSS的技术栈，实现了现代化、响应式的用户界面。前端架构基于组件化思想，通过JavaScript类和模块化设计实现代码复用和维护性。

## 技术栈

### 核心技术

| 技术 | 版本 | 用途 | 说明 |
|------|------|------|------|
| **HTML5** | - | 结构标记 | 语义化HTML结构 |
| **CSS3** | - | 样式设计 | 现代CSS特性 |
| **JavaScript** | ES6+ | 交互逻辑 | 原生JavaScript，无框架依赖 |
| **Tailwind CSS** | 3.x | CSS框架 | 原子化CSS框架 |
| **Font Awesome** | 6.4.0 | 图标库 | 矢量图标库 |

### CDN依赖

```html
<!-- Tailwind CSS -->
<script src="https://cdn.tailwindcss.com"></script>

<!-- Font Awesome -->
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css">
```

## 主要组件架构

### 1. 主应用类 (PluginMarketplace)

主插件市场应用的核心控制器。

```javascript
class PluginMarketplace {
    constructor() {
        // 配置初始化
        const config = window.GeekToolsConfig || {};
        this.baseURL = config.apiBaseUrl || '/api/v1';
        this.pageSize = config.frontend?.pageSize || 20;
        
        // 状态管理
        this.currentPage = 1;
        this.currentQuery = '';
        this.currentCategory = '';
        this.currentSort = 'downloads';
        this.plugins = [];
        this.totalPages = 0;
        
        // 认证状态
        this.authToken = localStorage.getItem('auth_token');
        this.currentUser = JSON.parse(localStorage.getItem('current_user') || 'null');
        
        this.init();
    }
}
```

**主要职责**:
- 应用初始化和配置管理
- 全局状态管理
- 用户认证状态管理
- 插件数据的加载和显示
- 事件绑定和处理

**核心方法**:
```javascript
// 初始化应用
init() {
    this.bindEvents();
    this.initAuth();
    this.loadStats();
    this.loadPlugins();
    this.setupFileUpload();
}

// 绑定事件监听器
bindEvents() {
    // 搜索功能
    document.getElementById('searchInput').addEventListener('input', 
        this.debounce(() => this.performSearch(), 300));
    
    // 排序和筛选
    document.getElementById('sortSelect').addEventListener('change', 
        () => this.changeSorting());
    
    // 分页控制
    document.getElementById('prevPageBtn').addEventListener('click', 
        () => this.changePage(this.currentPage - 1));
}

// 加载插件列表
async loadPlugins() {
    try {
        const params = new URLSearchParams({
            page: this.currentPage,
            limit: this.pageSize,
            search: this.currentQuery,
            sort: this.currentSort,
            order: 'desc'
        });

        const response = await fetch(`${this.baseURL}/plugins?${params}`);
        const data = await response.json();
        
        if (data.success) {
            this.plugins = data.data.plugins;
            this.totalPages = data.data.pagination.pages;
            this.renderPlugins();
            this.updatePagination();
        }
    } catch (error) {
        this.showError('加载插件失败: ' + error.message);
    }
}
```

### 2. 管理面板类 (AdminPanel)

管理后台的核心控制器。

```javascript
class AdminPanel {
    constructor() {
        this.baseURL = '/api/v1';
        this.authToken = localStorage.getItem('auth_token');
        this.currentUser = JSON.parse(localStorage.getItem('current_user') || 'null');
        this.currentTab = 'dashboard';
        
        // 分页状态管理
        this.usersPagination = { page: 1, limit: 20 };
        this.pluginsPagination = { page: 1, limit: 20 };
        this.activitiesPagination = { page: 1, limit: 20 };
        
        this.init();
    }
}
```

**主要功能模块**:
- **仪表板**: 系统统计数据展示
- **用户管理**: 用户列表、编辑、权限管理
- **插件管理**: 插件审核、状态管理
- **活动监控**: 登录活动追踪
- **SQL控制台**: 数据库查询工具

### 3. 配置管理 (GeekToolsConfig)

全局配置管理系统。

```javascript
window.GeekToolsConfig = {
    // API配置
    apiBaseUrl: '/api/v1',
    
    // 前端配置
    frontend: {
        pageSize: 20,
        supportedFileTypes: ['.tar.gz'],
        maxFileSize: 100 * 1024 * 1024,
        searchDebounceDelay: 300,
        debug: false
    },
    
    // 主题配置
    theme: {
        primaryColor: '#FF8C47',
        backgroundColor: '#F9F9F8',
        textColor: '#2F2F2F',
        darkMode: false
    },
    
    // 功能开关
    features: {
        enableRegistration: true,
        enableUpload: true,
        enableRating: true,
        enableStats: true,
        enableAdminPanel: true
    }
};
```

## UI组件详解

### 1. 导航栏组件

```html
<nav class="bg-white shadow-lg sticky top-0 z-50">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between h-16">
            <!-- Logo和标题 -->
            <div class="flex items-center">
                <h1 class="text-2xl font-bold text-claude-text">
                    <i class="fas fa-plug text-claude-orange mr-2"></i>
                    GeekTools 插件市场
                </h1>
            </div>
            
            <!-- 右侧用户菜单 -->
            <div class="flex items-center space-x-4">
                <!-- 未登录状态 -->
                <div id="loggedOut" class="flex items-center space-x-2">
                    <button onclick="app.showLoginModal()" 
                            class="bg-claude-orange hover:bg-orange-600 text-white px-4 py-2 rounded-lg transition-colors">
                        <i class="fas fa-sign-in-alt mr-2"></i>登录
                    </button>
                </div>
                
                <!-- 已登录状态 -->
                <div id="loggedIn" class="hidden flex items-center space-x-4">
                    <span class="text-claude-text">
                        <i class="fas fa-user mr-1"></i>
                        <span id="userEmail"></span>
                    </span>
                    <button onclick="app.logout()" 
                            class="text-gray-600 hover:text-gray-800 transition-colors">
                        <i class="fas fa-sign-out-alt mr-1"></i>登出
                    </button>
                </div>
            </div>
        </div>
    </div>
</nav>
```

**特性**:
- 响应式设计，支持移动端
- 粘性定位，滚动时保持在顶部
- 动态显示登录/登出状态
- 管理员权限显示管理面板入口

### 2. 插件卡片组件

```html
<div class="plugin-card bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow duration-300 overflow-hidden">
    <!-- 插件头部 -->
    <div class="p-6">
        <div class="flex items-start justify-between mb-3">
            <h3 class="text-xl font-semibold text-claude-text truncate flex-1 mr-4">
                {{plugin.name}}
            </h3>
            <div class="flex items-center bg-green-100 text-green-800 px-2 py-1 rounded-full text-sm font-medium">
                <i class="fas fa-check-circle mr-1"></i>
                {{plugin.status}}
            </div>
        </div>
        
        <!-- 插件描述 -->
        <p class="text-gray-600 mb-4 line-clamp-2">
            {{plugin.description}}
        </p>
        
        <!-- 插件元信息 -->
        <div class="flex items-center justify-between text-sm text-gray-500 mb-4">
            <span>
                <i class="fas fa-user mr-1"></i>
                {{plugin.author}}
            </span>
            <span>
                <i class="fas fa-tag mr-1"></i>
                v{{plugin.current_version}}
            </span>
        </div>
        
        <!-- 统计信息 -->
        <div class="flex items-center justify-between">
            <div class="flex items-center space-x-4">
                <span class="flex items-center text-yellow-600">
                    <i class="fas fa-star mr-1"></i>
                    {{plugin.rating}}
                </span>
                <span class="flex items-center text-blue-600">
                    <i class="fas fa-download mr-1"></i>
                    {{plugin.downloads}}
                </span>
            </div>
            
            <!-- 操作按钮 -->
            <div class="flex space-x-2">
                <button onclick="app.downloadPlugin('{{plugin.id}}')" 
                        class="bg-claude-orange hover:bg-orange-600 text-white px-3 py-1 rounded text-sm transition-colors">
                    <i class="fas fa-download mr-1"></i>下载
                </button>
                <button onclick="app.showPluginDetails('{{plugin.id}}')" 
                        class="border border-gray-300 hover:border-gray-400 text-gray-700 px-3 py-1 rounded text-sm transition-colors">
                    详情
                </button>
            </div>
        </div>
    </div>
</div>
```

**动态渲染实现**:
```javascript
renderPlugins() {
    const container = document.getElementById('pluginsGrid');
    
    if (this.plugins.length === 0) {
        container.innerHTML = this.renderEmptyState();
        return;
    }
    
    container.innerHTML = this.plugins.map(plugin => 
        this.renderPluginCard(plugin)
    ).join('');
}

renderPluginCard(plugin) {
    return `
        <div class="plugin-card bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow duration-300 overflow-hidden">
            <div class="p-6">
                <div class="flex items-start justify-between mb-3">
                    <h3 class="text-xl font-semibold text-claude-text truncate flex-1 mr-4">
                        ${this.escapeHtml(plugin.name)}
                    </h3>
                    <div class="flex items-center ${this.getStatusBadgeClass(plugin.status)} px-2 py-1 rounded-full text-sm font-medium">
                        <i class="fas fa-check-circle mr-1"></i>
                        ${plugin.status}
                    </div>
                </div>
                
                <p class="text-gray-600 mb-4 line-clamp-2">
                    ${this.escapeHtml(plugin.description || '暂无描述')}
                </p>
                
                <div class="flex items-center justify-between text-sm text-gray-500 mb-4">
                    <span>
                        <i class="fas fa-user mr-1"></i>
                        ${this.escapeHtml(plugin.author)}
                    </span>
                    <span>
                        <i class="fas fa-tag mr-1"></i>
                        v${plugin.current_version}
                    </span>
                </div>
                
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                        <span class="flex items-center text-yellow-600">
                            <i class="fas fa-star mr-1"></i>
                            ${plugin.rating.toFixed(1)}
                        </span>
                        <span class="flex items-center text-blue-600">
                            <i class="fas fa-download mr-1"></i>
                            ${this.formatNumber(plugin.downloads)}
                        </span>
                    </div>
                    
                    <div class="flex space-x-2">
                        <button onclick="app.downloadPlugin('${plugin.id}')" 
                                class="bg-claude-orange hover:bg-orange-600 text-white px-3 py-1 rounded text-sm transition-colors">
                            <i class="fas fa-download mr-1"></i>下载
                        </button>
                        <button onclick="app.showPluginDetails('${plugin.id}')" 
                                class="border border-gray-300 hover:border-gray-400 text-gray-700 px-3 py-1 rounded text-sm transition-colors">
                            详情
                        </button>
                    </div>
                </div>
            </div>
        </div>
    `;
}
```

### 3. 搜索和筛选组件

```html
<div class="bg-white rounded-lg shadow-md p-6 mb-8">
    <div class="flex flex-col md:flex-row gap-4">
        <!-- 搜索输入框 -->
        <div class="flex-1">
            <div class="relative">
                <input type="text" 
                       id="searchInput" 
                       placeholder="搜索插件..." 
                       class="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-claude-orange focus:border-transparent">
                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                    <i class="fas fa-search text-gray-400"></i>
                </div>
            </div>
        </div>
        
        <!-- 排序选择 -->
        <div class="md:w-48">
            <select id="sortSelect" 
                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-claude-orange focus:border-transparent">
                <option value="downloads">按下载量排序</option>
                <option value="rating">按评分排序</option>
                <option value="created_at">按创建时间排序</option>
                <option value="updated_at">按更新时间排序</option>
            </select>
        </div>
        
        <!-- 筛选按钮 -->
        <div class="flex space-x-2">
            <button id="filterBtn" 
                    class="px-4 py-2 border border-gray-300 rounded-lg hover:border-gray-400 transition-colors">
                <i class="fas fa-filter mr-2"></i>筛选
            </button>
            <button id="clearFiltersBtn" 
                    class="px-4 py-2 text-gray-600 hover:text-gray-800 transition-colors">
                <i class="fas fa-times mr-1"></i>清除
            </button>
        </div>
    </div>
    
    <!-- 高级筛选面板 -->
    <div id="advancedFilters" class="hidden mt-4 pt-4 border-t border-gray-200">
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <!-- 状态筛选 -->
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">状态</label>
                <select id="statusFilter" class="w-full px-3 py-2 border border-gray-300 rounded-lg">
                    <option value="">全部状态</option>
                    <option value="active">正常</option>
                    <option value="deprecated">已弃用</option>
                </select>
            </div>
            
            <!-- 评分筛选 -->
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">最低评分</label>
                <select id="ratingFilter" class="w-full px-3 py-2 border border-gray-300 rounded-lg">
                    <option value="">不限</option>
                    <option value="4">4星及以上</option>
                    <option value="3">3星及以上</option>
                    <option value="2">2星及以上</option>
                </select>
            </div>
            
            <!-- 标签筛选 -->
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">标签</label>
                <input type="text" 
                       id="tagFilter" 
                       placeholder="输入标签..." 
                       class="w-full px-3 py-2 border border-gray-300 rounded-lg">
            </div>
        </div>
    </div>
</div>
```

**搜索功能实现**:
```javascript
// 防抖搜索
debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

// 执行搜索
async performSearch() {
    const searchInput = document.getElementById('searchInput');
    this.currentQuery = searchInput.value.trim();
    this.currentPage = 1;
    
    await this.loadPlugins();
}

// 高级搜索
async advancedSearch() {
    const filters = {
        status: document.getElementById('statusFilter').value,
        rating: document.getElementById('ratingFilter').value,
        tag: document.getElementById('tagFilter').value.trim()
    };
    
    // 构建搜索参数
    const searchParams = {
        query: this.currentQuery,
        filters: Object.fromEntries(
            Object.entries(filters).filter(([key, value]) => value !== '')
        ),
        sort: {
            field: this.currentSort,
            order: 'desc'
        },
        pagination: {
            page: this.currentPage,
            limit: this.pageSize
        }
    };
    
    try {
        const response = await fetch(`${this.baseURL}/search/advanced`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(searchParams)
        });
        
        const data = await response.json();
        
        if (data.success) {
            this.plugins = data.data.plugins;
            this.totalPages = data.data.pagination.pages;
            this.renderPlugins();
            this.updatePagination();
        }
    } catch (error) {
        this.showError('搜索失败: ' + error.message);
    }
}
```

### 4. 分页组件

```html
<div id="pagination" class="flex items-center justify-between mt-8">
    <div class="text-sm text-gray-700">
        显示第 <span id="pageStart">1</span> - <span id="pageEnd">20</span> 项，
        共 <span id="totalItems">0</span> 项
    </div>
    
    <div class="flex items-center space-x-2">
        <button id="prevPageBtn" 
                class="px-3 py-2 border border-gray-300 rounded-lg hover:border-gray-400 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
            <i class="fas fa-chevron-left"></i>
            上一页
        </button>
        
        <div id="pageNumbers" class="flex space-x-1">
            <!-- 页码按钮动态生成 -->
        </div>
        
        <button id="nextPageBtn" 
                class="px-3 py-2 border border-gray-300 rounded-lg hover:border-gray-400 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
            下一页
            <i class="fas fa-chevron-right ml-1"></i>
        </button>
    </div>
</div>
```

**分页逻辑实现**:
```javascript
updatePagination() {
    const prevBtn = document.getElementById('prevPageBtn');
    const nextBtn = document.getElementById('nextPageBtn');
    const pageNumbers = document.getElementById('pageNumbers');
    const pageStart = document.getElementById('pageStart');
    const pageEnd = document.getElementById('pageEnd');
    const totalItems = document.getElementById('totalItems');
    
    // 更新分页信息
    const start = (this.currentPage - 1) * this.pageSize + 1;
    const end = Math.min(this.currentPage * this.pageSize, this.plugins.length);
    
    pageStart.textContent = start;
    pageEnd.textContent = end;
    totalItems.textContent = this.totalPages * this.pageSize;
    
    // 更新按钮状态
    prevBtn.disabled = this.currentPage <= 1;
    nextBtn.disabled = this.currentPage >= this.totalPages;
    
    // 生成页码按钮
    pageNumbers.innerHTML = this.generatePageNumbers();
}

generatePageNumbers() {
    const maxVisible = 5;
    const pages = [];
    
    let startPage = Math.max(1, this.currentPage - Math.floor(maxVisible / 2));
    let endPage = Math.min(this.totalPages, startPage + maxVisible - 1);
    
    // 调整startPage以确保显示足够的页码
    if (endPage - startPage < maxVisible - 1) {
        startPage = Math.max(1, endPage - maxVisible + 1);
    }
    
    // 添加第一页和省略号
    if (startPage > 1) {
        pages.push(this.createPageButton(1));
        if (startPage > 2) {
            pages.push('<span class="px-3 py-2 text-gray-500">...</span>');
        }
    }
    
    // 添加中间页码
    for (let i = startPage; i <= endPage; i++) {
        pages.push(this.createPageButton(i));
    }
    
    // 添加最后一页和省略号
    if (endPage < this.totalPages) {
        if (endPage < this.totalPages - 1) {
            pages.push('<span class="px-3 py-2 text-gray-500">...</span>');
        }
        pages.push(this.createPageButton(this.totalPages));
    }
    
    return pages.join('');
}

createPageButton(pageNum) {
    const isActive = pageNum === this.currentPage;
    const classes = isActive 
        ? 'px-3 py-2 bg-claude-orange text-white rounded-lg'
        : 'px-3 py-2 border border-gray-300 rounded-lg hover:border-gray-400 transition-colors';
    
    return `
        <button onclick="app.changePage(${pageNum})" class="${classes}">
            ${pageNum}
        </button>
    `;
}

async changePage(page) {
    if (page < 1 || page > this.totalPages || page === this.currentPage) {
        return;
    }
    
    this.currentPage = page;
    await this.loadPlugins();
    
    // 滚动到顶部
    window.scrollTo({ top: 0, behavior: 'smooth' });
}
```

### 5. 模态框组件

```html
<!-- 登录模态框 -->
<div id="loginModal" class="hidden fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4">
    <div class="bg-white rounded-lg shadow-xl max-w-md w-full max-h-screen overflow-y-auto">
        <div class="p-6">
            <!-- 模态框头部 -->
            <div class="flex items-center justify-between mb-4">
                <h2 class="text-xl font-bold text-claude-text">用户登录</h2>
                <button onclick="app.hideLoginModal()" 
                        class="text-gray-400 hover:text-gray-600 transition-colors">
                    <i class="fas fa-times text-xl"></i>
                </button>
            </div>
            
            <!-- 登录表单 -->
            <div id="sendCodeStep">
                <p class="text-gray-600 mb-4">请输入您的邮箱地址，我们将发送验证码到您的邮箱</p>
                
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-2">邮箱地址</label>
                    <input type="email" 
                           id="emailInput" 
                           placeholder="请输入邮箱地址" 
                           class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-claude-orange focus:border-transparent">
                </div>
                
                <button id="sendCodeBtn" 
                        onclick="app.sendVerificationCode()" 
                        class="w-full bg-claude-orange hover:bg-orange-600 text-white py-2 rounded-lg transition-colors">
                    发送验证码
                </button>
            </div>
            
            <!-- 验证码输入步骤 -->
            <div id="verifyCodeStep" class="hidden">
                <p class="text-gray-600 mb-4">验证码已发送到您的邮箱，请查收并输入验证码</p>
                
                <!-- 验证码显示（开发环境） -->
                <div id="codeDisplay" class="hidden mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
                    <p class="text-sm text-yellow-800">
                        <i class="fas fa-info-circle mr-1"></i>
                        开发环境验证码：<span id="displayedCode" class="font-mono font-bold"></span>
                    </p>
                </div>
                
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-2">验证码</label>
                    <input type="text" 
                           id="codeInput" 
                           placeholder="请输入6位验证码" 
                           maxlength="6" 
                           class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-claude-orange focus:border-transparent">
                </div>
                
                <div class="flex space-x-2">
                    <button onclick="app.resetLoginForm()" 
                            class="flex-1 border border-gray-300 hover:border-gray-400 text-gray-700 py-2 rounded-lg transition-colors">
                        重新发送
                    </button>
                    <button id="verifyCodeBtn" 
                            onclick="app.verifyCode()" 
                            class="flex-1 bg-claude-orange hover:bg-orange-600 text-white py-2 rounded-lg transition-colors">
                        验证登录
                    </button>
                </div>
            </div>
        </div>
    </div>
</div>
```

**模态框控制逻辑**:
```javascript
showLoginModal() {
    document.getElementById('loginModal').classList.remove('hidden');
    document.body.style.overflow = 'hidden';
    this.resetLoginForm();
    
    // 聚焦到邮箱输入框
    setTimeout(() => {
        document.getElementById('emailInput').focus();
    }, 100);
}

hideLoginModal() {
    document.getElementById('loginModal').classList.add('hidden');
    document.body.style.overflow = 'auto';
}

resetLoginForm() {
    document.getElementById('emailInput').value = '';
    document.getElementById('codeInput').value = '';
    document.getElementById('sendCodeStep').classList.remove('hidden');
    document.getElementById('verifyCodeStep').classList.add('hidden');
    document.getElementById('codeDisplay').classList.add('hidden');
}

// ESC键关闭模态框
document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
        const modal = document.getElementById('loginModal');
        if (!modal.classList.contains('hidden')) {
            this.hideLoginModal();
        }
    }
});

// 点击背景关闭模态框
document.getElementById('loginModal').addEventListener('click', (e) => {
    if (e.target.id === 'loginModal') {
        this.hideLoginModal();
    }
});
```

### 6. 消息提示组件

```javascript
// 消息提示系统
class MessageSystem {
    constructor() {
        this.container = null;
        this.createContainer();
    }
    
    createContainer() {
        this.container = document.createElement('div');
        this.container.id = 'messageContainer';
        this.container.className = 'fixed top-4 right-4 z-50 space-y-2';
        document.body.appendChild(this.container);
    }
    
    show(message, type = 'info', duration = 5000) {
        const messageEl = document.createElement('div');
        messageEl.className = `message-toast ${this.getTypeClass(type)} max-w-sm p-4 rounded-lg shadow-lg transform transition-all duration-300 translate-x-full opacity-0`;
        
        messageEl.innerHTML = `
            <div class="flex items-start">
                <div class="flex-shrink-0">
                    <i class="fas ${this.getTypeIcon(type)} mr-2"></i>
                </div>
                <div class="flex-1">
                    <p class="text-sm font-medium">${message}</p>
                </div>
                <div class="ml-4 flex-shrink-0 flex">
                    <button onclick="this.parentElement.parentElement.parentElement.remove()" 
                            class="text-gray-400 hover:text-gray-600 transition-colors">
                        <i class="fas fa-times"></i>
                    </button>
                </div>
            </div>
        `;
        
        this.container.appendChild(messageEl);
        
        // 显示动画
        setTimeout(() => {
            messageEl.classList.remove('translate-x-full', 'opacity-0');
        }, 100);
        
        // 自动消失
        if (duration > 0) {
            setTimeout(() => {
                this.hide(messageEl);
            }, duration);
        }
        
        return messageEl;
    }
    
    hide(messageEl) {
        messageEl.classList.add('translate-x-full', 'opacity-0');
        setTimeout(() => {
            if (messageEl.parentNode) {
                messageEl.parentNode.removeChild(messageEl);
            }
        }, 300);
    }
    
    getTypeClass(type) {
        const classes = {
            success: 'bg-green-50 border border-green-200 text-green-800',
            error: 'bg-red-50 border border-red-200 text-red-800',
            warning: 'bg-yellow-50 border border-yellow-200 text-yellow-800',
            info: 'bg-blue-50 border border-blue-200 text-blue-800'
        };
        return classes[type] || classes.info;
    }
    
    getTypeIcon(type) {
        const icons = {
            success: 'fa-check-circle text-green-500',
            error: 'fa-exclamation-circle text-red-500',
            warning: 'fa-exclamation-triangle text-yellow-500',
            info: 'fa-info-circle text-blue-500'
        };
        return icons[type] || icons.info;
    }
}

// 全局消息系统实例
const messageSystem = new MessageSystem();

// 便捷方法
window.showSuccess = (message) => messageSystem.show(message, 'success');
window.showError = (message) => messageSystem.show(message, 'error');
window.showWarning = (message) => messageSystem.show(message, 'warning');
window.showInfo = (message) => messageSystem.show(message, 'info');
```

## 响应式设计

### 1. 断点系统

使用Tailwind CSS的响应式断点：

```css
/* 移动设备优先 */
.container {
    @apply px-4;
}

/* 平板设备 (768px+) */
@screen md {
    .container {
        @apply px-6;
    }
}

/* 桌面设备 (1024px+) */
@screen lg {
    .container {
        @apply px-8;
    }
}

/* 大桌面设备 (1280px+) */
@screen xl {
    .container {
        @apply px-12;
    }
}
```

### 2. 网格布局

```html
<!-- 响应式插件网格 -->
<div id="pluginsGrid" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
    <!-- 插件卡片 -->
</div>

<!-- 响应式搜索栏 -->
<div class="flex flex-col md:flex-row gap-4">
    <div class="flex-1"><!-- 搜索框 --></div>
    <div class="md:w-48"><!-- 排序选择 --></div>
</div>
```

### 3. 移动端优化

```javascript
// 移动端检测
isMobile() {
    return window.innerWidth < 768;
}

// 移动端特定处理
handleMobileLayout() {
    if (this.isMobile()) {
        // 调整分页大小
        this.pageSize = 10;
        
        // 隐藏某些桌面端功能
        document.querySelectorAll('.desktop-only').forEach(el => {
            el.classList.add('hidden');
        });
        
        // 调整模态框大小
        document.querySelectorAll('.modal').forEach(modal => {
            modal.classList.add('mobile-modal');
        });
    }
}

// 监听窗口大小变化
window.addEventListener('resize', () => {
    this.handleMobileLayout();
});
```

通过这种组件化的UI设计，GeekTools插件市场实现了现代化、响应式、用户友好的界面，同时保持了代码的可维护性和扩展性。