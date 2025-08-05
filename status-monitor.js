// GeekTools Status Monitor Frontend
class StatusMonitor {
    constructor() {
        this.config = window.GeekToolsConfig || {};
        this.baseURL = this.config.apiBaseUrl || '/api/v1';
        this.authToken = localStorage.getItem('auth_token');
        this.currentUser = JSON.parse(localStorage.getItem('current_user') || 'null');
        this.autoRefreshEnabled = true;
        this.refreshInterval = null;
        this.refreshIntervalMs = this.config.admin?.monitor?.refreshInterval || 30000; // 30 seconds
        this.performanceChart = null;
        this.logOffset = 0;
        this.logLimit = this.config.admin?.monitor?.logLimit || 50;
        
        this.init();
    }

    init() {
        this.checkAuth();
        this.bindEvents();
        this.loadAllData();
        this.startAutoRefresh();
        this.updateServerTime();
    }

    checkAuth() {
        if (!this.authToken || !this.currentUser) {
            alert('请先登录');
            window.location.href = 'index.html';
            return;
        }

        document.getElementById('adminEmail').textContent = this.currentUser.email;
    }

    bindEvents() {
        // Logout
        document.getElementById('logoutBtn').addEventListener('click', () => {
            this.logout();
        });

        // Auto refresh toggle
        document.getElementById('autoRefreshToggle').addEventListener('change', (e) => {
            this.autoRefreshEnabled = e.target.checked;
            document.getElementById('autoRefreshStatus').textContent = e.target.checked ? '开启' : '关闭';
            
            if (e.target.checked) {
                this.startAutoRefresh();
            } else {
                this.stopAutoRefresh();
            }
        });

        // Refresh buttons
        document.getElementById('refreshServicesBtn').addEventListener('click', () => {
            this.loadServicesStatus();
        });

        document.getElementById('refreshLogsBtn').addEventListener('click', () => {
            this.loadSystemLogs(true);
        });

        // Chart time range
        document.getElementById('chartTimeRange').addEventListener('change', () => {
            this.loadPerformanceChart();
        });

        // Log level filter
        document.getElementById('logLevel').addEventListener('change', () => {
            this.loadSystemLogs(true);
        });

        // Load more logs
        document.getElementById('loadMoreLogsBtn').addEventListener('click', () => {
            this.loadSystemLogs(false);
        });

        // Test buttons
        document.getElementById('testDatabaseBtn').addEventListener('click', () => {
            this.testDatabaseConnection();
        });

        document.getElementById('testEmailBtn').addEventListener('click', () => {
            this.showTestEmailModal();
        });

        // Test Email Modal
        document.getElementById('closeTestEmailModal').addEventListener('click', () => {
            this.hideTestEmailModal();
        });

        document.getElementById('cancelTestEmail').addEventListener('click', () => {
            this.hideTestEmailModal();
        });

        document.getElementById('testEmailForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.sendTestEmail();
        });

        // Alert banner
        document.getElementById('dismissAlert').addEventListener('click', () => {
            this.hideAlert();
        });
    }

    async makeAuthenticatedRequest(url, options = {}) {
        const headers = {
            'Authorization': `Bearer ${this.authToken}`,
            'Content-Type': 'application/json',
            ...options.headers
        };

        const response = await fetch(url, {
            ...options,
            headers
        });

        if (response.status === 401 || response.status === 403) {
            alert('认证失败或权限不足');
            this.logout();
            return null;
        }

        return response;
    }

    async loadAllData() {
        try {
            await Promise.all([
                this.loadSystemMetrics(),
                this.loadServicesStatus(),
                this.loadSystemInfo(),
                this.loadDatabaseStatus(),
                this.loadEmailStatus(),
                this.loadSystemLogs(true),
                this.loadPerformanceChart()
            ]);
        } catch (error) {
            console.error('Failed to load data:', error);
            this.showError('加载监控数据失败');
        }
    }

    async loadSystemMetrics() {
        try {
            // Mock system metrics data
            const mockMetrics = {
                system_load: 0.65,
                memory_usage_percent: 72.3,
                disk_usage_percent: 45.8,
                network_traffic_mbps: 12.4,
                cpu_usage_percent: 28.5,
                response_time_ms: 125,
                active_connections: 47,
                uptime_seconds: 2847392,
                cpu_cores: 4,
                total_memory_gb: 16,
                operating_system: 'Ubuntu 22.04 LTS',
                performance_score: 85,
                trends: {
                    load: 'up',
                    memory: 'down',
                    disk: 'stable',
                    network: 'up'
                }
            };

            // Update main metrics
            document.getElementById('systemLoad').textContent = mockMetrics.system_load.toFixed(2);
            document.getElementById('memoryUsage').textContent = `${mockMetrics.memory_usage_percent.toFixed(1)}%`;
            document.getElementById('diskUsage').textContent = `${mockMetrics.disk_usage_percent.toFixed(1)}%`;
            document.getElementById('networkTraffic').textContent = `${mockMetrics.network_traffic_mbps.toFixed(1)} Mbps`;

            // Update trends
            this.updateTrend('loadTrend', 'loadTrendText', mockMetrics.trends.load);
            this.updateTrend('memoryTrend', 'memoryTrendText', mockMetrics.trends.memory);
            this.updateTrend('diskTrend', 'diskTrendText', mockMetrics.trends.disk);
            this.updateTrend('networkTrend', 'networkTrendText', mockMetrics.trends.network);

            // Update performance metrics
            document.getElementById('cpuUsage').textContent = `${mockMetrics.cpu_usage_percent.toFixed(1)}%`;
            document.getElementById('responseTime').textContent = `${mockMetrics.response_time_ms}ms`;
            document.getElementById('activeConnections').textContent = mockMetrics.active_connections;

            // Update system info
            document.getElementById('systemUptime').textContent = this.formatUptime(mockMetrics.uptime_seconds);
            document.getElementById('cpuCores').textContent = mockMetrics.cpu_cores;
            document.getElementById('totalMemory').textContent = `${mockMetrics.total_memory_gb} GB`;
            document.getElementById('operatingSystem').textContent = mockMetrics.operating_system;

            // Update performance score
            document.getElementById('performanceScore').textContent = mockMetrics.performance_score;
            this.updatePerformanceRing(mockMetrics.performance_score);

            // Update system health
            this.updateSystemHealth(mockMetrics);

        } catch (error) {
            console.error('Failed to load system metrics:', error);
            this.showError('加载系统指标失败');
        }
    }

    updateTrend(trendElementId, textElementId, direction) {
        const trendElement = document.getElementById(trendElementId);
        const textElement = document.getElementById(textElementId);
        
        trendElement.className = `fas text-xs mr-1 ${
            direction === 'up' ? 'fa-arrow-up metric-trend-up' :
            direction === 'down' ? 'fa-arrow-down metric-trend-down' :
            'fa-minus metric-trend-stable'
        }`;
        
        textElement.textContent = direction === 'up' ? '↑ 较上小时' : 
                                  direction === 'down' ? '↓ 较上小时' : 
                                  '— 较上小时';
    }

    updateSystemHealth(metrics) {
        const healthIndicator = document.getElementById('systemHealthIndicator');
        const healthText = document.getElementById('systemHealthText');
        
        let healthStatus = 'healthy';
        let healthMessage = '系统正常';
        
        // Determine health status based on metrics
        if (metrics.cpu_usage_percent > 90 || metrics.memory_usage_percent > 95 || metrics.disk_usage_percent > 90) {
            healthStatus = 'critical';
            healthMessage = '系统异常';
            this.showAlert('系统资源使用率过高，请检查系统状态');
        } else if (metrics.cpu_usage_percent > 75 || metrics.memory_usage_percent > 80 || metrics.disk_usage_percent > 75) {
            healthStatus = 'warning';
            healthMessage = '系统警告';
        }
        
        healthIndicator.className = `health-indicator ${healthStatus}`;
        healthText.textContent = healthMessage;
    }

    updatePerformanceRing(score) {
        const ring = document.getElementById('performanceRing');
        const circumference = 2 * Math.PI * 56; // radius = 56
        const offset = circumference - (score / 100) * circumference;
        
        ring.style.strokeDasharray = circumference;
        ring.style.strokeDashoffset = offset;
        
        // Update ring color based on score
        if (score >= 80) {
            ring.style.stroke = '#10B981'; // green
        } else if (score >= 60) {
            ring.style.stroke = '#F59E0B'; // yellow
        } else {
            ring.style.stroke = '#EF4444'; // red
        }
    }

    async loadServicesStatus() {
        try {
            // Mock services data
            const mockServices = [
                {
                    name: 'Web服务器',
                    description: 'Axum HTTP服务器',
                    status: 'healthy',
                    response_time: 12,
                    uptime: '99.9%',
                    last_check: new Date().toISOString(),
                    port: 3000,
                    cpu_usage: 15.2,
                    memory_usage: 245
                },
                {
                    name: '数据库服务',
                    description: 'PostgreSQL数据库',
                    status: 'healthy',
                    response_time: 8,
                    uptime: '99.8%',
                    last_check: new Date().toISOString(),
                    port: 5432,
                    cpu_usage: 8.7,
                    memory_usage: 1024
                },
                {
                    name: 'SMTP服务',
                    description: '邮件发送服务',
                    status: 'warning',
                    response_time: null,
                    uptime: '0%',
                    last_check: new Date().toISOString(),
                    port: 587,
                    cpu_usage: 0,
                    memory_usage: 0
                },
                {
                    name: '文件存储',
                    description: '插件文件存储服务',
                    status: 'healthy',
                    response_time: 5,
                    uptime: '100%',
                    last_check: new Date().toISOString(),
                    port: null,
                    cpu_usage: 2.1,
                    memory_usage: 128
                }
            ];

            this.renderServices(mockServices);

        } catch (error) {
            console.error('Failed to load services status:', error);
            this.showError('加载服务状态失败');
        }
    }

    renderServices(services) {
        const container = document.getElementById('servicesList');
        
        container.innerHTML = services.map(service => `
            <div class="service-card flex items-center justify-between p-4 bg-gray-50 rounded-lg">
                <div class="flex items-center flex-1">
                    <div class="health-indicator ${service.status}" title="${this.getStatusText(service.status)}"></div>
                    <div class="flex-1">
                        <div class="flex items-center justify-between mb-1">
                            <h4 class="font-medium text-sm">${this.escapeHtml(service.name)}</h4>
                            <span class="px-2 py-1 text-xs rounded-full ${this.getServiceStatusStyle(service.status)}">
                                ${this.getStatusText(service.status)}
                            </span>
                        </div>
                        <p class="text-xs text-gray-500 mb-2">${this.escapeHtml(service.description)}</p>
                        <div class="flex items-center space-x-4 text-xs text-gray-500">
                            ${service.response_time ? `<span><i class="fas fa-clock mr-1"></i>${service.response_time}ms</span>` : '<span><i class="fas fa-clock mr-1"></i>-</span>'}
                            <span><i class="fas fa-chart-line mr-1"></i>运行时间 ${service.uptime}</span>
                            ${service.port ? `<span><i class="fas fa-network-wired mr-1"></i>端口 ${service.port}</span>` : ''}
                        </div>
                    </div>
                </div>
                <div class="text-right text-xs space-y-1">
                    <div class="text-gray-600">CPU: <span class="font-medium">${service.cpu_usage.toFixed(1)}%</span></div>
                    <div class="text-gray-600">内存: <span class="font-medium">${this.formatBytes(service.memory_usage * 1024 * 1024)}</span></div>
                    <div class="text-gray-500">检查: ${this.timeAgo(service.last_check)}</div>
                </div>
            </div>
        `).join('');
    }

    getServiceStatusStyle(status) {
        switch (status) {
            case 'healthy': return 'bg-green-100 text-green-700';
            case 'warning': return 'bg-yellow-100 text-yellow-700';
            case 'critical': return 'bg-red-100 text-red-700';
            default: return 'bg-gray-100 text-gray-700';
        }
    }

    getStatusText(status) {
        switch (status) {
            case 'healthy': return '正常';
            case 'warning': return '警告';
            case 'critical': return '异常';
            default: return '未知';
        }
    }

    async loadSystemInfo() {
        // System info is loaded as part of loadSystemMetrics
        this.updateServerTime();
    }

    updateServerTime() {
        const now = new Date();
        document.getElementById('serverTime').textContent = now.toLocaleString('zh-CN');
        
        // Update every second
        setTimeout(() => this.updateServerTime(), 1000);
    }

    async loadDatabaseStatus() {
        try {
            // Mock database status
            const mockDbStatus = {
                status: 'healthy',
                response_time: 12,
                active_connections: 15,
                max_connections: 100,
                queries_per_second: 45.7,
                cache_hit_ratio: 98.5,
                connection_info: '连接正常'
            };

            document.getElementById('dbResponseTime').textContent = `${mockDbStatus.response_time}ms`;
            document.getElementById('dbActiveConnections').textContent = mockDbStatus.active_connections;
            document.getElementById('dbMaxConnections').textContent = mockDbStatus.max_connections;
            document.getElementById('dbQueriesPerSecond').textContent = mockDbStatus.queries_per_second.toFixed(1);
            document.getElementById('dbCacheHitRatio').textContent = `${mockDbStatus.cache_hit_ratio.toFixed(1)}%`;
            document.getElementById('dbConnectionInfo').textContent = mockDbStatus.connection_info;
            
            const dbHealthIndicator = document.getElementById('dbHealthIndicator');
            dbHealthIndicator.className = `health-indicator ${mockDbStatus.status}`;

        } catch (error) {
            console.error('Failed to load database status:', error);
            this.showError('加载数据库状态失败');
        }
    }

    async loadEmailStatus() {
        try {
            // Mock email status
            const mockEmailStatus = {
                status: 'warning',
                response_time: null,
                emails_sent_today: 0,
                success_rate: 0,
                queue_length: 0,
                last_email_sent: null,
                connection_info: '配置待完善'
            };

            document.getElementById('emailResponseTime').textContent = mockEmailStatus.response_time ? `${mockEmailStatus.response_time}ms` : '-';
            document.getElementById('emailsSentToday').textContent = mockEmailStatus.emails_sent_today;
            document.getElementById('emailSuccessRate').textContent = `${mockEmailStatus.success_rate}%`;
            document.getElementById('emailQueueLength').textContent = mockEmailStatus.queue_length;
            document.getElementById('lastEmailSent').textContent = mockEmailStatus.last_email_sent ? this.timeAgo(mockEmailStatus.last_email_sent) : '-';
            document.getElementById('emailConnectionInfo').textContent = mockEmailStatus.connection_info;
            
            const emailHealthIndicator = document.getElementById('emailHealthIndicator');
            emailHealthIndicator.className = `health-indicator ${mockEmailStatus.status}`;

        } catch (error) {
            console.error('Failed to load email status:', error);
            this.showError('加载邮件服务状态失败');
        }
    }

    async loadSystemLogs(reset = false) {
        try {
            if (reset) {
                this.logOffset = 0;
            }

            const logLevel = document.getElementById('logLevel').value;
            
            // Mock log data
            const mockLogs = this.generateMockLogs(this.logLimit, this.logOffset, logLevel);
            
            const container = document.getElementById('systemLogs');
            
            if (reset) {
                container.innerHTML = '';
            }
            
            mockLogs.forEach(log => {
                const logElement = this.createLogElement(log);
                container.appendChild(logElement);
            });
            
            this.logOffset += mockLogs.length;
            
            // Show/hide load more button
            document.getElementById('loadMoreLogsBtn').style.display = mockLogs.length < this.logLimit ? 'none' : 'block';

        } catch (error) {
            console.error('Failed to load system logs:', error);
            this.showError('加载系统日志失败');
        }
    }

    generateMockLogs(limit, offset, level) {
        const logs = [];
        const levels = level ? [level] : ['info', 'warning', 'error'];
        const messages = {
            info: [
                '用户登录成功',
                '插件上传成功',
                '数据库连接建立',
                '缓存更新完成',
                'API请求处理成功'
            ],
            warning: [
                '内存使用率较高',
                '磁盘空间不足警告',
                'SMTP服务未配置',
                '连接池接近上限',
                '响应时间较慢'
            ],
            error: [
                '数据库连接失败',
                '文件上传失败',
                '认证令牌过期',
                '邮件发送失败',
                '系统异常错误'
            ]
        };
        
        for (let i = 0; i < limit; i++) {
            const randomLevel = levels[Math.floor(Math.random() * levels.length)];
            const randomMessage = messages[randomLevel][Math.floor(Math.random() * messages[randomLevel].length)];
            const timestamp = new Date(Date.now() - (offset + i) * 60000); // Each log is 1 minute apart
            
            logs.push({
                id: `log_${offset + i}`,
                level: randomLevel,
                message: randomMessage,
                timestamp: timestamp.toISOString(),
                source: 'server',
                details: `详细信息 ${offset + i}`
            });
        }
        
        return logs;
    }

    createLogElement(log) {
        const logElement = document.createElement('div');
        logElement.className = `log-entry p-3 rounded-lg ${log.level}`;
        
        logElement.innerHTML = `
            <div class="flex items-start justify-between">
                <div class="flex-1">
                    <div class="flex items-center space-x-2 mb-1">
                        <span class="px-2 py-1 text-xs rounded-full ${this.getLogLevelStyle(log.level)}">
                            ${this.getLogLevelText(log.level)}
                        </span>
                        <span class="text-xs text-gray-500">${log.source}</span>
                        <span class="text-xs text-gray-500">${this.formatDateTime(log.timestamp)}</span>
                    </div>
                    <p class="text-sm text-gray-800">${this.escapeHtml(log.message)}</p>
                </div>
                <button onclick="statusMonitor.showLogDetails('${log.id}')" class="text-gray-400 hover:text-gray-600 text-xs">
                    <i class="fas fa-info-circle"></i>
                </button>
            </div>
        `;
        
        return logElement;
    }

    getLogLevelStyle(level) {
        switch (level) {
            case 'error': return 'bg-red-100 text-red-700';
            case 'warning': return 'bg-yellow-100 text-yellow-700';
            case 'info': return 'bg-blue-100 text-blue-700';
            default: return 'bg-gray-100 text-gray-700';
        }
    }

    getLogLevelText(level) {
        switch (level) {
            case 'error': return '错误';
            case 'warning': return '警告';
            case 'info': return '信息';
            default: return level;
        }
    }

    showLogDetails(logId) {
        // Show log details in a modal or expanded view
        this.showInfo(`查看日志详情: ${logId}`);
    }

    async loadPerformanceChart() {
        try {
            const timeRange = document.getElementById('chartTimeRange').value;
            
            // Mock chart data
            const mockChartData = this.generateMockChartData(timeRange);
            
            this.renderPerformanceChart(mockChartData);

        } catch (error) {
            console.error('Failed to load performance chart:', error);
            this.showError('加载性能图表失败');
        }
    }

    generateMockChartData(timeRange) {
        const now = new Date();
        const dataPoints = timeRange === '1h' ? 12 : timeRange === '6h' ? 36 : timeRange === '24h' ? 48 : 168;
        const intervalMs = timeRange === '1h' ? 5 * 60 * 1000 : // 5 minutes
                          timeRange === '6h' ? 10 * 60 * 1000 : // 10 minutes
                          timeRange === '24h' ? 30 * 60 * 1000 : // 30 minutes
                          60 * 60 * 1000; // 1 hour
        
        const data = {
            labels: [],
            cpu: [],
            memory: [],
            disk: []
        };
        
        for (let i = dataPoints - 1; i >= 0; i--) {
            const timestamp = new Date(now.getTime() - i * intervalMs);
            data.labels.push(timestamp.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }));
            
            // Generate realistic mock data with some variation
            data.cpu.push(Math.max(0, Math.min(100, 30 + Math.sin(i * 0.1) * 20 + Math.random() * 10)));
            data.memory.push(Math.max(0, Math.min(100, 70 + Math.sin(i * 0.05) * 15 + Math.random() * 5)));
            data.disk.push(Math.max(0, Math.min(100, 45 + Math.sin(i * 0.02) * 5 + Math.random() * 2)));
        }
        
        return data;
    }

    renderPerformanceChart(data) {
        const canvas = document.getElementById('performanceChart');
        const ctx = canvas.getContext('2d');
        
        // Clear canvas
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        
        // Simple chart implementation (in production, use Chart.js or similar)
        const padding = 40;
        const chartWidth = canvas.width - 2 * padding;
        const chartHeight = canvas.height - 2 * padding;
        
        // Draw grid
        ctx.strokeStyle = '#E5E7EB';
        ctx.lineWidth = 1;
        
        // Horizontal grid lines
        for (let i = 0; i <= 5; i++) {
            const y = padding + (chartHeight / 5) * i;
            ctx.beginPath();
            ctx.moveTo(padding, y);
            ctx.lineTo(padding + chartWidth, y);
            ctx.stroke();
        }
        
        // Vertical grid lines
        const stepX = chartWidth / (data.labels.length - 1);
        for (let i = 0; i < data.labels.length; i += Math.floor(data.labels.length / 6)) {
            const x = padding + stepX * i;
            ctx.beginPath();
            ctx.moveTo(x, padding);
            ctx.lineTo(x, padding + chartHeight);
            ctx.stroke();
        }
        
        // Draw lines
        const drawLine = (dataSet, color) => {
            ctx.strokeStyle = color;
            ctx.lineWidth = 2;
            ctx.beginPath();
            
            dataSet.forEach((value, index) => {
                const x = padding + (chartWidth / (dataSet.length - 1)) * index;
                const y = padding + chartHeight - (value / 100) * chartHeight;
                
                if (index === 0) {
                    ctx.moveTo(x, y);
                } else {
                    ctx.lineTo(x, y);
                }
            });
            
            ctx.stroke();
        };
        
        drawLine(data.cpu, '#3B82F6');    // Blue
        drawLine(data.memory, '#10B981'); // Green
        drawLine(data.disk, '#8B5CF6');   // Purple
        
        // Draw Y-axis labels
        ctx.fillStyle = '#6B7280';
        ctx.font = '12px system-ui';
        ctx.textAlign = 'right';
        
        for (let i = 0; i <= 5; i++) {
            const value = 100 - (i * 20);
            const y = padding + (chartHeight / 5) * i + 4;
            ctx.fillText(`${value}%`, padding - 10, y);
        }
    }

    // Test Methods
    async testDatabaseConnection() {
        try {
            this.showButtonLoading('testDatabaseBtn', '测试中...');
            
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 2000));
            
            this.showSuccess('数据库连接测试成功');
            
        } catch (error) {
            console.error('Database test failed:', error);
            this.showError('数据库连接测试失败');
        } finally {
            this.hideButtonLoading('testDatabaseBtn', '测试连接');
        }
    }

    showTestEmailModal() {
        document.getElementById('testEmailRecipient').value = this.currentUser.email;
        document.getElementById('testEmailModal').classList.remove('hidden');
        document.body.style.overflow = 'hidden';
    }

    hideTestEmailModal() {
        document.getElementById('testEmailModal').classList.add('hidden');
        document.body.style.overflow = 'auto';
    }

    async sendTestEmail() {
        const recipient = document.getElementById('testEmailRecipient').value.trim();
        
        if (!recipient || !this.isValidEmail(recipient)) {
            this.showError('请输入有效的收件人邮箱');
            return;
        }

        try {
            this.showButtonLoading('sendTestEmailBtn', '发送中...');
            
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 2000));
            
            this.showSuccess(`测试邮件已发送到 ${recipient}`);
            this.hideTestEmailModal();
            
        } catch (error) {
            console.error('Failed to send test email:', error);
            this.showError('发送测试邮件失败');
        } finally {
            this.hideButtonLoading('sendTestEmailBtn', '发送邮件');
        }
    }

    // Auto Refresh
    startAutoRefresh() {
        if (!this.autoRefreshEnabled) return;
        
        this.refreshInterval = setInterval(() => {
            this.loadAllData();
        }, this.refreshIntervalMs);
    }

    stopAutoRefresh() {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
            this.refreshInterval = null;
        }
    }

    // Alert Management
    showAlert(message, type = 'error') {
        const alertBanner = document.getElementById('alertBanner');
        const alertMessage = document.getElementById('alertMessage');
        
        alertMessage.textContent = message;
        alertBanner.className = `alert-banner ${
            type === 'error' ? 'bg-red-500' : 
            type === 'warning' ? 'bg-yellow-500' : 
            'bg-blue-500'
        } text-white px-4 py-3`;
        
        alertBanner.classList.remove('hidden');
        
        // Auto hide after 10 seconds
        setTimeout(() => {
            this.hideAlert();
        }, 10000);
    }

    hideAlert() {
        document.getElementById('alertBanner').classList.add('hidden');
    }

    // Utility Methods
    formatUptime(seconds) {
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        
        if (days > 0) {
            return `${days}天 ${hours}小时`;
        } else if (hours > 0) {
            return `${hours}小时 ${minutes}分钟`;
        } else {
            return `${minutes}分钟`;
        }
    }

    formatBytes(bytes) {
        const sizes = ['B', 'KB', 'MB', 'GB'];
        if (bytes === 0) return '0 B';
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
    }

    formatDateTime(dateString) {
        const date = new Date(dateString);
        return date.toLocaleString('zh-CN', {
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit'
        });
    }

    timeAgo(dateString) {
        const date = new Date(dateString);
        const now = new Date();
        const diffInSeconds = Math.floor((now - date) / 1000);

        if (diffInSeconds < 60) return '刚刚';
        if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)}分钟前`;
        if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)}小时前`;
        return `${Math.floor(diffInSeconds / 86400)}天前`;
    }

    isValidEmail(email) {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(email);
    }

    escapeHtml(text) {
        const map = {
            '&': '&amp;',
            '<': '&lt;',
            '>': '&gt;',
            '"': '&quot;',
            "'": '&#039;'
        };
        return String(text || '').replace(/[&<>"']/g, function(m) { return map[m]; });
    }

    showButtonLoading(buttonId, loadingText) {
        const button = document.getElementById(buttonId);
        button.disabled = true;
        button.innerHTML = `<div class="spinner inline-block mr-2"></div>${loadingText}`;
    }

    hideButtonLoading(buttonId, originalText) {
        const button = document.getElementById(buttonId);
        button.disabled = false;
        button.innerHTML = originalText;
    }

    showError(message) {
        this.showNotification(message, 'error');
    }

    showSuccess(message) {
        this.showNotification(message, 'success');
    }

    showInfo(message) {
        this.showNotification(message, 'info');
    }

    showNotification(message, type = 'info') {
        const notification = document.createElement('div');
        notification.className = `fixed top-4 right-4 px-6 py-3 rounded-lg text-white z-50 transform transition-all duration-300 ${
            type === 'error' ? 'bg-red-500' : 
            type === 'success' ? 'bg-green-500' : 
            'bg-blue-500'
        }`;
        notification.textContent = message;

        document.body.appendChild(notification);

        setTimeout(() => {
            notification.style.transform = 'translateX(0)';
        }, 100);

        setTimeout(() => {
            notification.style.transform = 'translateX(100%)';
            setTimeout(() => {
                if (document.body.contains(notification)) {
                    document.body.removeChild(notification);
                }
            }, 300);
        }, 3000);
    }

    logout() {
        localStorage.removeItem('auth_token');
        localStorage.removeItem('current_user');
        window.location.href = 'index.html';
    }
}

// Initialize status monitor when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.statusMonitor = new StatusMonitor();
});