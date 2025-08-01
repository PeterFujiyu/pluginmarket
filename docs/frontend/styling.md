# 样式系统设计

## 概述

GeekTools 插件市场采用 Tailwind CSS 作为主要的样式框架，结合自定义CSS实现了现代化、一致性和响应式的视觉设计。样式系统遵循原子化CSS理念，提供高度可定制的设计语言。

## 技术架构

### 1. Tailwind CSS 配置

```javascript
// Tailwind CSS 自定义配置
tailwind.config = {
    theme: {
        extend: {
            // 自定义颜色系统
            colors: {
                'claude-orange': '#FF8C47',      // 主品牌色
                'claude-bg': '#F9F9F8',          // 背景色
                'claude-text': '#2F2F2F',        // 主文字色
                'claude-light': '#FEFEFE',       // 浅色背景
                'claude-gray': {
                    50: '#FAFAFA',
                    100: '#F4F4F5',
                    200: '#E4E4E7',
                    300: '#D1D5DB',
                    400: '#9CA3AF',
                    500: '#6B7280',
                    600: '#4B5563',
                    700: '#374151',
                    800: '#1F2937',
                    900: '#111827',
                }
            },
            
            // 自定义字体系统
            fontFamily: {
                'claude': [
                    'system-ui', 
                    '-apple-system', 
                    'BlinkMacSystemFont', 
                    'Segoe UI', 
                    'Roboto', 
                    'sans-serif'
                ],
                'mono': [
                    'SFMono-Regular',
                    'Consolas',
                    'Liberation Mono',
                    'Menlo',
                    'Courier',
                    'monospace'
                ]
            },
            
            // 自定义间距
            spacing: {
                '18': '4.5rem',   // 72px
                '22': '5.5rem',   // 88px
                '88': '22rem',    // 352px
                '128': '32rem',   // 512px
            },
            
            // 自定义圆角
            borderRadius: {
                'xl': '1rem',     // 16px
                '2xl': '1.5rem',  // 24px
                '3xl': '2rem',    // 32px
            },
            
            // 自定义阴影
            boxShadow: {
                'soft': '0 2px 15px 0 rgba(0, 0, 0, 0.08)',
                'medium': '0 4px 25px 0 rgba(0, 0, 0, 0.1)',
                'strong': '0 8px 40px 0 rgba(0, 0, 0, 0.12)',
                'claude': '0 4px 20px 0 rgba(255, 140, 71, 0.15)',
            },
            
            // 自定义动画
            animation: {
                'fade-in': 'fadeIn 0.3s ease-in-out',
                'slide-up': 'slideUp 0.3s ease-out',
                'slide-down': 'slideDown 0.3s ease-out',
                'bounce-in': 'bounceIn 0.5s ease-out',
                'pulse-slow': 'pulse 3s ease-in-out infinite',
            },
            
            // 自定义关键帧
            keyframes: {
                fadeIn: {
                    '0%': { opacity: '0' },
                    '100%': { opacity: '1' },
                },
                slideUp: {
                    '0%': { transform: 'translateY(10px)', opacity: '0' },
                    '100%': { transform: 'translateY(0)', opacity: '1' },
                },
                slideDown: {
                    '0%': { transform: 'translateY(-10px)', opacity: '0' },
                    '100%': { transform: 'translateY(0)', opacity: '1' },
                },
                bounceIn: {
                    '0%': { transform: 'scale(0.3)', opacity: '0' },
                    '50%': { transform: 'scale(1.05)' },
                    '70%': { transform: 'scale(0.9)' },
                    '100%': { transform: 'scale(1)', opacity: '1' },
                }
            }
        }
    },
    
    // 插件配置
    plugins: [
        // 添加自定义组件类
        function({ addComponents }) {
            addComponents({
                '.btn-primary': {
                    '@apply bg-claude-orange hover:bg-orange-600 text-white font-medium px-4 py-2 rounded-lg transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-claude-orange focus:ring-opacity-50': {},
                },
                '.btn-secondary': {
                    '@apply bg-white hover:bg-gray-50 text-claude-text border border-gray-300 hover:border-gray-400 font-medium px-4 py-2 rounded-lg transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-claude-orange focus:ring-opacity-50': {},
                },
                '.btn-ghost': {
                    '@apply text-claude-text hover:text-claude-orange hover:bg-orange-50 font-medium px-4 py-2 rounded-lg transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-claude-orange focus:ring-opacity-50': {},
                },
                '.input-field': {
                    '@apply w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-claude-orange focus:border-transparent transition-all duration-200 placeholder-gray-400': {},
                }
            });
        },
        
        // 添加自定义工具类
        function({ addUtilities }) {
            addUtilities({
                '.text-gradient': {
                    'background': 'linear-gradient(135deg, #FF8C47 0%, #FF6B35 100%)',
                    'background-clip': 'text',
                    '-webkit-background-clip': 'text',
                    '-webkit-text-fill-color': 'transparent',
                },
                '.bg-gradient-primary': {
                    'background': 'linear-gradient(135deg, #FF8C47 0%, #FF6B35 100%)',
                },
                '.bg-gradient-subtle': {
                    'background': 'linear-gradient(135deg, #F9F9F8 0%, #EFEFED 100%)',
                },
                '.shadow-glow': {
                    'box-shadow': '0 0 20px rgba(255, 140, 71, 0.3)',
                },
                '.line-clamp-2': {
                    'display': '-webkit-box',
                    '-webkit-line-clamp': '2',
                    '-webkit-box-orient': 'vertical',
                    'overflow': 'hidden',
                },
                '.line-clamp-3': {
                    'display': '-webkit-box',
                    '-webkit-line-clamp': '3',
                    '-webkit-box-orient': 'vertical',
                    'overflow': 'hidden',
                }
            });
        }
    ]
}
```

### 2. 自定义CSS样式

```css
/* 全局样式定义 */
:root {
    /* 颜色变量 */
    --claude-orange: #FF8C47;
    --claude-orange-dark: #E67E22;
    --claude-bg: #F9F9F8;
    --claude-text: #2F2F2F;
    --claude-light: #FEFEFE;
    
    /* 间距变量 */
    --space-xs: 0.25rem;    /* 4px */
    --space-sm: 0.5rem;     /* 8px */
    --space-md: 1rem;       /* 16px */
    --space-lg: 1.5rem;     /* 24px */
    --space-xl: 2rem;       /* 32px */
    
    /* 字体大小变量 */
    --text-xs: 0.75rem;     /* 12px */
    --text-sm: 0.875rem;    /* 14px */
    --text-base: 1rem;      /* 16px */
    --text-lg: 1.125rem;    /* 18px */
    --text-xl: 1.25rem;     /* 20px */
    --text-2xl: 1.5rem;     /* 24px */
    
    /* 圆角变量 */
    --radius-sm: 0.25rem;   /* 4px */
    --radius-md: 0.5rem;    /* 8px */
    --radius-lg: 0.75rem;   /* 12px */
    --radius-xl: 1rem;      /* 16px */
    
    /* 阴影变量 */
    --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
    --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
    --shadow-xl: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
    
    /* 过渡变量 */
    --transition-fast: 150ms ease-in-out;
    --transition-normal: 200ms ease-in-out;
    --transition-slow: 300ms ease-in-out;
}

/* 全局基础样式 */
body {
    font-family: var(--font-claude);
    color: var(--claude-text);
    background: var(--claude-bg);
    line-height: 1.6;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
}

/* 滚动条样式 */
::-webkit-scrollbar {
    width: 8px;
    height: 8px;
}

::-webkit-scrollbar-track {
    background: #f1f1f1;
    border-radius: 4px;
}

::-webkit-scrollbar-thumb {
    background: #c1c1c1;
    border-radius: 4px;
    transition: background var(--transition-normal);
}

::-webkit-scrollbar-thumb:hover {
    background: #a8a8a8;
}

/* 选择文本样式 */
::selection {
    background-color: rgba(255, 140, 71, 0.2);
    color: var(--claude-text);
}

::-moz-selection {
    background-color: rgba(255, 140, 71, 0.2);
    color: var(--claude-text);
}

/* 焦点样式 */
:focus {
    outline: 2px solid var(--claude-orange);
    outline-offset: 2px;
}

:focus:not(:focus-visible) {
    outline: none;
}

/* 无障碍隐藏类 */
.sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
}
```

## 组件样式系统

### 1. 按钮组件样式

```css
/* 按钮基础样式 */
.btn {
    @apply inline-flex items-center justify-center font-medium rounded-lg transition-all duration-200;
    @apply focus:outline-none focus:ring-2 focus:ring-opacity-50;
    @apply disabled:opacity-50 disabled:cursor-not-allowed;
}

/* 按钮尺寸变体 */
.btn-xs {
    @apply px-2 py-1 text-xs;
}

.btn-sm {
    @apply px-3 py-1.5 text-sm;
}

.btn-md {
    @apply px-4 py-2 text-base;
}

.btn-lg {
    @apply px-6 py-3 text-lg;
}

.btn-xl {
    @apply px-8 py-4 text-xl;
}

/* 按钮颜色变体 */
.btn-primary {
    @apply bg-claude-orange hover:bg-orange-600 text-white;
    @apply focus:ring-claude-orange;
    @apply shadow-sm hover:shadow-md;
}

.btn-secondary {
    @apply bg-white hover:bg-gray-50 text-claude-text;
    @apply border border-gray-300 hover:border-gray-400;
    @apply focus:ring-claude-orange;
}

.btn-success {
    @apply bg-green-500 hover:bg-green-600 text-white;
    @apply focus:ring-green-500;
}

.btn-danger {
    @apply bg-red-500 hover:bg-red-600 text-white;
    @apply focus:ring-red-500;
}

.btn-warning {
    @apply bg-yellow-500 hover:bg-yellow-600 text-white;
    @apply focus:ring-yellow-500;
}

.btn-ghost {
    @apply text-claude-text hover:text-claude-orange hover:bg-orange-50;
    @apply focus:ring-claude-orange;
}

.btn-link {
    @apply text-claude-orange hover:text-orange-600 underline;
    @apply focus:ring-claude-orange;
    @apply p-0 bg-transparent;
}

/* 按钮状态样式 */
.btn-loading {
    @apply relative text-transparent;
}

.btn-loading::after {
    content: "";
    @apply absolute inset-0 flex items-center justify-center;
    @apply w-4 h-4 border-2 border-white border-t-transparent rounded-full;
    animation: spin 1s linear infinite;
}

/* 按钮组样式 */
.btn-group {
    @apply inline-flex rounded-lg shadow-sm isolate;
}

.btn-group > .btn {
    @apply relative rounded-none border-r-0;
}

.btn-group > .btn:first-child {
    @apply rounded-l-lg;
}

.btn-group > .btn:last-child {
    @apply rounded-r-lg border-r;
}

.btn-group > .btn:hover {
    @apply z-10;
}
```

### 2. 卡片组件样式

```css
/* 卡片基础样式 */
.card {
    @apply bg-white rounded-lg shadow-sm border border-gray-200;
    @apply transition-all duration-200;
}

.card-hover {
    @apply hover:shadow-md hover:shadow-claude/10;
    @apply hover:-translate-y-0.5;
}

.card-interactive {
    @apply cursor-pointer;
    @apply hover:shadow-lg hover:shadow-claude/15;
    @apply hover:-translate-y-1;
    @apply active:translate-y-0 active:shadow-md;
}

/* 卡片内容区域 */
.card-header {
    @apply px-6 py-4 border-b border-gray-200;
}

.card-body {
    @apply px-6 py-4;
}

.card-footer {
    @apply px-6 py-4 border-t border-gray-200;
    @apply bg-gray-50 rounded-b-lg;
}

/* 卡片变体 */
.card-outlined {
    @apply border-2 border-gray-200 shadow-none;
}

.card-elevated {
    @apply shadow-lg border-0;
}

.card-flat {
    @apply shadow-none border-0 bg-transparent;
}

/* 插件卡片特殊样式 */
.plugin-card {
    @apply card card-hover;
    @apply overflow-hidden;
}

.plugin-card .plugin-header {
    @apply flex items-start justify-between mb-3;
}

.plugin-card .plugin-title {
    @apply text-xl font-semibold text-claude-text truncate flex-1 mr-4;
}

.plugin-card .plugin-description {
    @apply text-gray-600 mb-4 line-clamp-2;
}

.plugin-card .plugin-meta {
    @apply flex items-center justify-between text-sm text-gray-500 mb-4;
}

.plugin-card .plugin-stats {
    @apply flex items-center space-x-4;
}

.plugin-card .plugin-actions {
    @apply flex space-x-2;
}

/* 状态徽章样式 */
.status-badge {
    @apply inline-flex items-center px-2 py-1 rounded-full text-sm font-medium;
}

.status-badge.active {
    @apply bg-green-100 text-green-800;
}

.status-badge.deprecated {
    @apply bg-yellow-100 text-yellow-800;
}

.status-badge.banned {
    @apply bg-red-100 text-red-800;
}
```

### 3. 表单组件样式

```css
/* 表单基础样式 */
.form-group {
    @apply mb-4;
}

.form-label {
    @apply block text-sm font-medium text-gray-700 mb-2;
}

.form-input {
    @apply w-full px-3 py-2 border border-gray-300 rounded-lg;
    @apply focus:ring-2 focus:ring-claude-orange focus:border-transparent;
    @apply transition-all duration-200 placeholder-gray-400;
    @apply disabled:bg-gray-50 disabled:text-gray-500 disabled:cursor-not-allowed;
}

.form-input.error {
    @apply border-red-500 focus:ring-red-500;
}

.form-input.success {
    @apply border-green-500 focus:ring-green-500;
}

/* 输入框变体 */
.form-input-sm {
    @apply px-2 py-1 text-sm;
}

.form-input-lg {
    @apply px-4 py-3 text-lg;
}

/* 搜索输入框 */
.search-input {
    @apply form-input pl-10;
}

.search-input-icon {
    @apply absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none;
    @apply text-gray-400;
}

/* 选择框样式 */
.form-select {
    @apply form-input appearance-none bg-white;
    @apply bg-no-repeat bg-right bg-center;
    background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3e%3cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='m6 8 4 4 4-4'/%3e%3c/svg%3e");
    background-size: 1.5em 1.5em;
    padding-right: 2.5rem;
}

/* 复选框和单选框 */
.form-checkbox,
.form-radio {
    @apply w-4 h-4 text-claude-orange border-gray-300 rounded;
    @apply focus:ring-claude-orange focus:ring-2;
}

.form-radio {
    @apply rounded-full;
}

/* 文本域 */
.form-textarea {
    @apply form-input resize-vertical min-h-20;
}

/* 文件上传 */
.form-file {
    @apply form-input file:mr-4 file:py-2 file:px-4;
    @apply file:rounded-lg file:border-0;
    @apply file:text-sm file:font-medium;
    @apply file:bg-claude-orange file:text-white;
    @apply hover:file:bg-orange-600;
}

/* 表单验证样式 */
.form-error {
    @apply text-red-600 text-sm mt-1;
}

.form-success {
    @apply text-green-600 text-sm mt-1;
}

.form-help {
    @apply text-gray-500 text-sm mt-1;
}
```

### 4. 模态框样式

```css
/* 模态框基础样式 */
.modal {
    @apply fixed inset-0 z-50 overflow-y-auto;
}

.modal-backdrop {
    @apply fixed inset-0 bg-black bg-opacity-50 transition-opacity;
}

.modal-container {
    @apply flex min-h-full items-center justify-center p-4;
}

.modal-content {
    @apply relative bg-white rounded-lg shadow-xl;
    @apply w-full max-w-md transform transition-all;
    @apply max-h-screen overflow-y-auto;
}

/* 模态框尺寸变体 */
.modal-sm {
    @apply max-w-sm;
}

.modal-md {
    @apply max-w-md;
}

.modal-lg {
    @apply max-w-2xl;
}

.modal-xl {
    @apply max-w-4xl;
}

.modal-full {
    @apply max-w-none w-full h-full m-0 rounded-none;
}

/* 模态框内容区域 */
.modal-header {
    @apply flex items-center justify-between p-6 border-b border-gray-200;
}

.modal-title {
    @apply text-lg font-semibold text-claude-text;
}

.modal-close {
    @apply text-gray-400 hover:text-gray-600 transition-colors;
    @apply p-1 rounded-lg hover:bg-gray-100;
}

.modal-body {
    @apply p-6;
}

.modal-footer {
    @apply flex items-center justify-end space-x-2 p-6 border-t border-gray-200;
    @apply bg-gray-50 rounded-b-lg;
}

/* 模态框动画 */
.modal-enter {
    @apply opacity-0 scale-95;
}

.modal-enter-active {
    @apply opacity-100 scale-100 transition-all duration-200;
}

.modal-exit {
    @apply opacity-100 scale-100;
}

.modal-exit-active {
    @apply opacity-0 scale-95 transition-all duration-200;
}

/* 移动端适配 */
@media (max-width: 640px) {
    .modal-content {
        @apply m-2 max-h-none;
    }
    
    .modal-body {
        @apply p-4;
    }
    
    .modal-header,
    .modal-footer {
        @apply p-4;
    }
}
```

### 5. 导航组件样式

```css
/* 导航栏样式 */
.navbar {
    @apply bg-white shadow-lg sticky top-0 z-40;
}

.navbar-container {
    @apply max-w-7xl mx-auto px-4 sm:px-6 lg:px-8;
}

.navbar-content {
    @apply flex justify-between items-center h-16;
}

.navbar-brand {
    @apply flex items-center text-2xl font-bold text-claude-text;
}

.navbar-brand-icon {
    @apply text-claude-orange mr-2;
}

.navbar-menu {
    @apply hidden md:flex items-center space-x-4;
}

.navbar-item {
    @apply text-claude-text hover:text-claude-orange;
    @apply px-3 py-2 rounded-lg transition-colors;
}

.navbar-item.active {
    @apply text-claude-orange bg-orange-50;
}

/* 移动端菜单 */
.navbar-mobile-menu {
    @apply md:hidden absolute top-full left-0 right-0;
    @apply bg-white border-t border-gray-200 shadow-lg;
}

.navbar-mobile-item {
    @apply block px-4 py-3 text-claude-text hover:bg-gray-50;
    @apply border-b border-gray-100 last:border-b-0;
}

/* 用户菜单 */
.user-menu {
    @apply relative;
}

.user-menu-trigger {
    @apply flex items-center space-x-2 px-3 py-2 rounded-lg;
    @apply text-claude-text hover:bg-gray-50 transition-colors;
}

.user-menu-dropdown {
    @apply absolute right-0 mt-2 w-48 bg-white rounded-lg shadow-lg;
    @apply border border-gray-200 py-1 z-50;
}

.user-menu-item {
    @apply block px-4 py-2 text-sm text-claude-text;
    @apply hover:bg-gray-50 transition-colors;
}

.user-menu-item.danger {
    @apply text-red-600 hover:bg-red-50;
}
```

## 主题系统

### 1. 深色模式支持

```css
/* 深色模式变量 */
@media (prefers-color-scheme: dark) {
    :root {
        --claude-bg: #1F2937;
        --claude-text: #F9FAFB;
        --claude-light: #374151;
    }
}

/* 深色模式类 */
.dark {
    --claude-bg: #1F2937;
    --claude-text: #F9FAFB;
    --claude-light: #374151;
}

/* 深色模式组件样式 */
.dark .card {
    @apply bg-gray-800 border-gray-700;
}

.dark .form-input {
    @apply bg-gray-800 border-gray-700 text-white;
    @apply placeholder-gray-400;
}

.dark .navbar {
    @apply bg-gray-900 border-gray-800;
}

.dark .modal-content {
    @apply bg-gray-800;
}

.dark .modal-header,
.dark .modal-footer {
    @apply border-gray-700;
}
```

### 2. 主题切换实现

```javascript
class ThemeManager {
    constructor() {
        this.currentTheme = localStorage.getItem('theme') || 'auto';
        this.init();
    }
    
    init() {
        this.applyTheme();
        this.bindEvents();
    }
    
    applyTheme() {
        const html = document.documentElement;
        
        switch (this.currentTheme) {
            case 'dark':
                html.classList.add('dark');
                break;
            case 'light':
                html.classList.remove('dark');
                break;
            case 'auto':
            default:
                if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
                    html.classList.add('dark');
                } else {
                    html.classList.remove('dark');
                }
                break;
        }
    }
    
    setTheme(theme) {
        this.currentTheme = theme;
        localStorage.setItem('theme', theme);
        this.applyTheme();
        this.updateThemeIcon();
    }
    
    toggleTheme() {
        const themes = ['light', 'dark', 'auto'];
        const currentIndex = themes.indexOf(this.currentTheme);
        const nextIndex = (currentIndex + 1) % themes.length;
        this.setTheme(themes[nextIndex]);
    }
    
    bindEvents() {
        // 监听系统主题变化
        window.matchMedia('(prefers-color-scheme: dark)')
            .addEventListener('change', () => {
                if (this.currentTheme === 'auto') {
                    this.applyTheme();
                }
            });
    }
    
    updateThemeIcon() {
        const themeIcon = document.getElementById('themeIcon');
        if (!themeIcon) return;
        
        const icons = {
            light: 'fas fa-sun',
            dark: 'fas fa-moon',
            auto: 'fas fa-adjust'
        };
        
        themeIcon.className = icons[this.currentTheme] || icons.auto;
    }
}

// 初始化主题管理器
const themeManager = new ThemeManager();
```

## 响应式设计

### 1. 断点系统

```css
/* 自定义断点 */
@custom-media --mobile (max-width: 640px);
@custom-media --tablet (min-width: 641px) and (max-width: 1024px);
@custom-media --desktop (min-width: 1025px);

/* 响应式工具类 */
.container-responsive {
    @apply w-full max-w-7xl mx-auto px-4;
    
    @screen sm {
        @apply px-6;
    }
    
    @screen lg {
        @apply px-8;
    }
}

/* 响应式网格 */
.grid-responsive {
    @apply grid gap-6;
    @apply grid-cols-1;
    
    @screen md {
        @apply grid-cols-2;
    }
    
    @screen lg {
        @apply grid-cols-3;
    }
    
    @screen xl {
        @apply grid-cols-4;
    }
}

/* 响应式文字大小 */
.text-responsive {
    @apply text-base;
    
    @screen sm {
        @apply text-lg;
    }
    
    @screen lg {
        @apply text-xl;
    }
}

/* 响应式间距 */
.spacing-responsive {
    @apply p-4;
    
    @screen sm {
        @apply p-6;
    }
    
    @screen lg {
        @apply p-8;
    }
}
```

### 2. 移动端优化

```css
/* 移动端特定样式 */
@media (max-width: 640px) {
    /* 触摸友好的按钮尺寸 */
    .btn {
        @apply min-h-11 min-w-11;
    }
    
    /* 移动端导航 */
    .navbar-mobile {
        @apply block;
    }
    
    .navbar-desktop {
        @apply hidden;
    }
    
    /* 移动端模态框 */
    .modal-content {
        @apply w-full h-full max-w-none max-h-none rounded-none;
    }
    
    /* 移动端表单 */
    .form-input {
        @apply text-base; /* 防止iOS缩放 */
    }
    
    /* 移动端卡片 */
    .plugin-card {
        @apply mx-2;
    }
    
    /* 移动端分页 */
    .pagination-desktop {
        @apply hidden;
    }
    
    .pagination-mobile {
        @apply block;
    }
}

/* 平板端适配 */
@media (min-width: 641px) and (max-width: 1024px) {
    .plugin-card {
        @apply max-w-sm mx-auto;
    }
    
    .grid-responsive {
        @apply grid-cols-2;
    }
}
```

## 性能优化

### 1. CSS优化策略

```css
/* 使用CSS变量减少重复 */
.optimized-component {
    color: var(--claude-text);
    background-color: var(--claude-bg);
    transition: all var(--transition-normal);
}

/* 避免昂贵的属性 */
.performance-optimized {
    /* 使用transform而不是改变布局属性 */
    transform: translateY(0);
    transition: transform var(--transition-fast);
}

.performance-optimized:hover {
    transform: translateY(-2px);
}

/* 使用will-change提示浏览器优化 */
.will-animate {
    will-change: transform, opacity;
}

/* 完成动画后移除will-change */
.animation-complete {
    will-change: auto;
}
```

### 2. 关键CSS内联

```html
<!-- 关键路径CSS内联 */
<style>
/* 页面布局关键样式 */
body { 
    font-family: system-ui, -apple-system, sans-serif; 
    margin: 0; 
    background: #F9F9F8; 
}

.navbar { 
    background: white; 
    box-shadow: 0 1px 3px rgba(0,0,0,0.1); 
    position: sticky; 
    top: 0; 
    z-index: 40; 
}

.container { 
    max-width: 1280px; 
    margin: 0 auto; 
    padding: 0 16px; 
}

/* 加载状态样式 */
.loading {
    opacity: 0.7;
    pointer-events: none;
}

.spinner {
    width: 20px;
    height: 20px;
    border: 2px solid #e5e5e5;
    border-top: 2px solid #FF8C47;
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
}
</style>
```

### 3. 动画性能优化

```css
/* 高性能动画 */
@keyframes slideInUp {
    from {
        transform: translate3d(0, 100%, 0);
        opacity: 0;
    }
    to {
        transform: translate3d(0, 0, 0);
        opacity: 1;
    }
}

.slide-in-up {
    animation: slideInUp 0.3s ease-out forwards;
    /* 使用GPU加速 */
    transform: translate3d(0, 0, 0);
}

/* 减少重排重绘的动画 */
.fade-transition {
    transition: opacity 0.2s ease-in-out, transform 0.2s ease-in-out;
}

.fade-transition.fade-enter {
    opacity: 0;
    transform: scale(0.95);
}

.fade-transition.fade-enter-active {
    opacity: 1;
    transform: scale(1);
}
```

## 可访问性 (A11y)

### 1. 颜色对比度

```css
/* 确保足够的颜色对比度 */
.text-primary {
    color: #2F2F2F; /* 对比度 > 4.5:1 */
}

.text-secondary {
    color: #4B5563; /* 对比度 > 4.5:1 */
}

.text-muted {
    color: #6B7280; /* 对比度 > 3:1 */
}

/* 链接样式 */
.link {
    color: #FF8C47;
    text-decoration: underline;
}

.link:hover {
    color: #E67E22;
    text-decoration: none;
}

.link:focus {
    outline: 2px solid #FF8C47;
    outline-offset: 2px;
}
```

### 2. 焦点指示器

```css
/* 键盘导航焦点样式 */
.focus-visible {
    outline: 2px solid var(--claude-orange);
    outline-offset: 2px;
}

/* 自定义焦点环 */
.custom-focus:focus-visible {
    box-shadow: 0 0 0 2px var(--claude-orange);
    outline: none;
}

/* 跳过链接 */
.skip-link {
    position: absolute;
    top: -40px;
    left: 6px;
    background: var(--claude-orange);
    color: white;
    padding: 8px;
    text-decoration: none;
    border-radius: 4px;
    z-index: 1000;
}

.skip-link:focus {
    top: 6px;
}
```

### 3. 屏幕阅读器支持

```css
/* 屏幕阅读器专用内容 */
.sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
}

/* 可选择性显示给屏幕阅读器 */
.sr-only-focusable:focus {
    position: static;
    width: auto;
    height: auto;
    padding: inherit;
    margin: inherit;
    overflow: visible;
    clip: auto;
    white-space: normal;
}

/* 高对比度模式支持 */
@media (prefers-contrast: high) {
    .card {
        border: 2px solid;
    }
    
    .btn {
        border: 2px solid;
    }
}

/* 减少动画偏好 */
@media (prefers-reduced-motion: reduce) {
    *,
    *::before,
    *::after {
        animation-duration: 0.01ms !important;
        animation-iteration-count: 1 !important;
        transition-duration: 0.01ms !important;
    }
}
```

通过这套完整的样式系统，GeekTools插件市场实现了现代化、一致性、可访问性和高性能的用户界面设计。