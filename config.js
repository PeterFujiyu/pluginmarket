// GeekTools Plugin Marketplace 配置文件
// 请根据您的部署环境修改以下配置

window.GeekToolsConfig = {
    // API 基础 URL
    // 开发环境：http://localhost:3000/api/v1
    // 生产环境：https://api.your-domain.com/v1
    apiBaseUrl: '/api/v1',
    
    // 管理面板配置
    admin: {
        // 自动刷新间隔 (毫秒)
        autoRefreshInterval: 30000,
        
        // 配置管理
        config: {
            // 配置变更备份
            autoBackupOnChange: true,
            // 配置版本保留数量
            maxVersionHistory: 50
        },
        
        // 备份管理
        backup: {
            // 默认保留天数
            defaultRetentionDays: 30,
            // 最大备份文件大小 (MB)
            maxBackupSize: 1024,
            // 定时备份默认时间
            defaultScheduleTime: '02:00'
        },
        
        // 系统监控
        monitor: {
            // 监控数据刷新间隔 (毫秒)
            refreshInterval: 30000,
            // 性能图表数据点数量
            chartDataPoints: 50,
            // 日志加载数量
            logLimit: 50
        }
    },
    
    // 前端配置
    frontend: {
        // 每页显示的插件数量
        pageSize: 20,
        
        // 支持的文件上传格式
        supportedFileTypes: ['.tar.gz'],
        
        // 最大文件上传大小 (100MB)
        maxFileSize: 100 * 1024 * 1024,
        
        // 搜索防抖延迟 (毫秒)
        searchDebounceDelay: 300,
        
        // 是否启用调试模式
        debug: false
    },
    
    // 主题配置
    theme: {
        // 主色调
        primaryColor: '#FF8C47',
        
        // 背景色
        backgroundColor: '#F9F9F8',
        
        // 文字颜色
        textColor: '#2F2F2F',
        
        // 启用深色模式
        darkMode: false
    },
    
    // 功能开关
    features: {
        // 是否显示用户注册按钮
        enableRegistration: true,
        
        // 是否显示插件上传功能
        enableUpload: true,
        
        // 是否显示评分功能
        enableRating: true,
        
        // 是否显示统计信息
        enableStats: true,
        
        // 是否显示管理员面板链接
        enableAdminPanel: true
    },
    
    // 文案配置
    text: {
        siteName: 'GeekTools 插件市场',
        siteDescription: '发现和分享强大的命令行工具',
        
        // 页脚版权信息
        copyright: 'Copyright © 2025 Github@PeterFujiyu, Claude Agent',
        
        // 空状态文案
        noPluginsFound: '未找到匹配的插件',
        noPluginsDescription: '尝试调整搜索关键词或清除筛选条件'
    },
    
    // 自动检测配置
    autoDetect: {
        // 自动检测API基础URL（适用于代理部署）
        apiBaseUrl: true,
        
        // 自动检测主题模式
        darkMode: true
    }
};

// 自动配置检测
(function() {
    const config = window.GeekToolsConfig;
    
    // 自动检测API基础URL
    if (config.autoDetect.apiBaseUrl) {
        // 如果当前页面通过代理访问，自动设置相对路径
        const isProxied = window.location.port === '8080' || 
                         window.location.hostname === 'localhost' ||
                         window.location.hostname === '127.0.0.1';
        
        if (isProxied) {
            config.apiBaseUrl = '/api/v1';
        } else {
            // 生产环境可能需要完整URL
            config.apiBaseUrl = window.location.origin + '/api/v1';
        }
    }
    
    // 自动检测深色模式
    if (config.autoDetect.darkMode) {
        config.theme.darkMode = window.matchMedia && 
                               window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    
    // 应用调试模式
    if (config.frontend.debug) {
        console.log('GeekTools Config:', config);
        window.GEEKTOOLS_DEBUG = true;
    }
})();