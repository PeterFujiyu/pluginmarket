// GeekTools Plugin Marketplace Admin Panel - Minimalist Black-White-Gray Design
class AdminPanel {
    constructor() {
        // 从配置文件读取设置
        const config = window.GeekToolsConfig || {};
        this.baseURL = config.apiBaseUrl || '/api/v1';
        this.authToken = localStorage.getItem('auth_token');
        this.currentUser = JSON.parse(localStorage.getItem('current_user') || 'null');
        this.currentTab = 'dashboard';
        
        this.init();
    }

    init() {
        // 检查管理员权限
        if (!this.checkAdminAccess()) {
            window.location.href = 'index.html';
            return;
        }

        this.bindEvents();
        this.initTheme();
        this.updateUserInfo();
        this.updateClock();
        this.loadDashboard();
    }

    checkAdminAccess() {
        return this.authToken && 
               this.currentUser && 
               this.currentUser.role === 'admin';
    }

    initTheme() {
        // 主题切换系统
        const darkModeToggle = document.getElementById('darkModeToggle');
        const currentTheme = localStorage.getItem('theme') || 'light';
        
        this.applyTheme(currentTheme);
        
        darkModeToggle?.addEventListener('click', () => {
            const newTheme = document.documentElement.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
            this.applyTheme(newTheme);
            localStorage.setItem('theme', newTheme);
        });
    }

    applyTheme(theme) {
        document.documentElement.setAttribute('data-theme', theme);
        
        const toggle = document.getElementById('darkModeToggle');
        if (toggle) {
            const icon = toggle.querySelector('i');
            if (theme === 'dark') {
                icon.className = 'fas fa-sun';
                toggle.title = '切换到亮色模式';
            } else {
                icon.className = 'fas fa-moon';
                toggle.title = '切换到暗色模式';
            }
        }
    }

    bindEvents() {
        // 标签切换
        const navTabs = document.querySelectorAll('.nav-tab');
        navTabs.forEach(tab => {
            tab.addEventListener('click', () => {
                const tabName = tab.dataset.tab;
                this.switchTab(tabName);
            });
        });

        // 退出登录
        document.getElementById('logoutBtn')?.addEventListener('click', () => {
            this.logout();
        });

        // SQL表单
        document.getElementById('sqlForm')?.addEventListener('submit', (e) => {
            e.preventDefault();
            this.executeSql();
        });

        // 快速查询
        document.querySelectorAll('.quick-query').forEach(btn => {
            btn.addEventListener('click', () => {
                const query = btn.dataset.query;
                document.getElementById('sqlQuery').value = query;
            });
        });

        // 清空按钮
        document.getElementById('clearSql')?.addEventListener('click', () => {
            document.getElementById('sqlQuery').value = '';
        });

        document.getElementById('clearResults')?.addEventListener('click', () => {
            this.hideSqlResults();
        });

        // 搜索和筛选
        this.bindSearchEvents();
        
        // 刷新按钮
        document.getElementById('refreshUsers')?.addEventListener('click', () => {
            this.loadUsers();
        });

        document.getElementById('refreshPlugins')?.addEventListener('click', () => {
            this.loadPlugins();
        });

        // 模态框关闭
        this.bindModalEvents();
    }

    bindSearchEvents() {
        const userSearch = document.getElementById('userSearch');
        const pluginSearch = document.getElementById('pluginSearch');
        const statusFilter = document.getElementById('statusFilter');

        let searchTimeout;

        userSearch?.addEventListener('input', (e) => {
            clearTimeout(searchTimeout);
            searchTimeout = setTimeout(() => {
                this.filterUsers(e.target.value);
            }, 300);
        });

        pluginSearch?.addEventListener('input', (e) => {
            clearTimeout(searchTimeout);
            searchTimeout = setTimeout(() => {
                this.filterPlugins(e.target.value, statusFilter?.value);
            }, 300);
        });

        statusFilter?.addEventListener('change', (e) => {
            this.filterPlugins(pluginSearch?.value || '', e.target.value);
        });
    }

    bindModalEvents() {
        const actionModal = document.getElementById('actionModal');
        const closeBtn = document.getElementById('closeActionModal');
        const cancelBtn = document.getElementById('cancelAction');

        // 背景点击关闭
        actionModal?.addEventListener('click', (e) => {
            if (e.target === actionModal) {
                this.hideActionModal();
            }
        });

        // 关闭按钮
        closeBtn?.addEventListener('click', () => {
            this.hideActionModal();
        });

        cancelBtn?.addEventListener('click', () => {
            this.hideActionModal();
        });

        // ESC键关闭
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && !actionModal?.classList.contains('hidden')) {
                this.hideActionModal();
            }
        });
    }

    switchTab(tabName) {
        // 更新标签状态
        document.querySelectorAll('.nav-tab').forEach(tab => {
            if (tab.dataset.tab === tabName) {
                tab.classList.add('active');
            } else {
                tab.classList.remove('active');
            }
        });

        // 显示对应内容
        document.querySelectorAll('.tab-content').forEach(content => {
            if (content.id === tabName) {
                content.classList.remove('hidden');
                content.classList.add('fade-in');
            } else {
                content.classList.add('hidden');
                content.classList.remove('fade-in');
            }
        });

        this.currentTab = tabName;

        // 加载对应数据
        switch (tabName) {
            case 'dashboard':
                this.loadDashboard();
                break;
            case 'users':
                this.loadUsers();
                break;
            case 'plugins':
                this.loadPlugins();
                break;
            case 'sql':
                // SQL标签不需要加载数据
                break;
        }
    }

    updateUserInfo() {
        const adminUserEmail = document.getElementById('adminUserEmail');
        if (adminUserEmail && this.currentUser) {
            adminUserEmail.textContent = this.currentUser.email;
        }
    }

    updateClock() {
        const updateTime = () => {
            const now = new Date();
            const timeString = now.toLocaleTimeString('zh-CN');
            const clockElement = document.getElementById('currentTime');
            if (clockElement) {
                clockElement.textContent = timeString;
            }
        };

        updateTime();
        setInterval(updateTime, 1000);
    }

    async loadDashboard() {
        try {
            // 加载统计数据
            const [usersResponse, pluginsResponse] = await Promise.all([
                this.fetchWithAuth(`${this.baseURL}/admin/users`),
                this.fetchWithAuth(`${this.baseURL}/admin/plugins`)
            ]);

            if (usersResponse.ok && pluginsResponse.ok) {
                const usersData = await usersResponse.json();
                const pluginsData = await pluginsResponse.json();

                this.updateDashboardStats({
                    totalUsers: usersData.total || usersData.users?.length || 0,
                    totalPlugins: pluginsData.total || pluginsData.plugins?.length || 0,
                    todayVisits: Math.floor(Math.random() * 500) + 100, // 模拟数据
                });

                // 加载最近活动
                this.loadRecentActivities();
            }
        } catch (error) {
            console.error('Failed to load dashboard:', error);
            this.showAlert('加载仪表板失败', 'error');
        }
    }

    updateDashboardStats(stats) {
        const elements = {
            totalUsers: document.getElementById('totalUsers'),
            totalPluginsAdmin: document.getElementById('totalPluginsAdmin'),
            todayVisits: document.getElementById('todayVisits')
        };

        Object.entries(stats).forEach(([key, value]) => {
            const elementKey = key === 'totalPlugins' ? 'totalPluginsAdmin' : key;
            const element = elements[elementKey];
            if (element) {
                this.animateNumber(element, 0, value);
            }
        });
    }

    animateNumber(element, start, end, duration = 1000) {
        const startTime = performance.now();
        const range = end - start;
        
        const updateNumber = (currentTime) => {
            const elapsed = currentTime - startTime;
            const progress = Math.min(elapsed / duration, 1);
            
            const currentValue = Math.floor(start + (range * progress));
            element.textContent = currentValue.toLocaleString();
            
            if (progress < 1) {
                requestAnimationFrame(updateNumber);
            }
        };
        
        requestAnimationFrame(updateNumber);
    }

    async loadRecentActivities() {
        try {
            // 模拟最近活动数据
            const activities = [
                { type: 'user', message: '新用户注册', time: '2分钟前' },
                { type: 'plugin', message: '插件 "Git Helper" 被下载', time: '5分钟前' },
                { type: 'admin', message: '管理员执行SQL查询', time: '10分钟前' },
                { type: 'user', message: '用户上传新插件', time: '15分钟前' }
            ];

            this.renderRecentActivities(activities);
        } catch (error) {
            console.error('Failed to load recent activities:', error);
        }
    }

    renderRecentActivities(activities) {
        const container = document.getElementById('recentActivities');
        if (!container) return;

        container.innerHTML = activities.map(activity => `
            <div class="flex items-center space-x-3 p-2 border-b border-gray-100">
                <div class="w-2 h-2 bg-success rounded-full"></div>
                <div class="flex-1">
                    <span class="text-body">${this.escapeHtml(activity.message)}</span>
                    <span class="text-micro ml-2">${this.escapeHtml(activity.time)}</span>
                </div>
            </div>
        `).join('');
    }

    async loadUsers() {
        try {
            const response = await this.fetchWithAuth(`${this.baseURL}/admin/users`);
            
            if (!response.ok) {
                throw new Error('Failed to load users');
            }
            
            const data = await response.json();
            this.users = data.users || [];
            this.renderUsers(this.users);
            this.updateUserCount(this.users.length);
            
        } catch (error) {
            console.error('Failed to load users:', error);
            this.showAlert('加载用户失败', 'error');
        }
    }

    renderUsers(users) {
        const tbody = document.getElementById('usersTableBody');
        if (!tbody) return;

        tbody.innerHTML = users.map(user => `
            <tr>
                <td class="font-mono">${user.id}</td>
                <td class="font-mono">${this.escapeHtml(user.email)}</td>
                <td>
                    <span class="text-micro px-2 py-1 ${user.role === 'admin' ? 'bg-error text-white' : 'bg-gray-100'} border">
                        ${user.role === 'admin' ? '管理员' : '用户'}
                    </span>
                </td>
                <td class="font-mono">${this.formatDate(user.created_at)}</td>
                <td class="font-mono">${this.formatDate(user.last_login_at)}</td>
                <td>
                    <span class="status-dot ${user.is_active ? '' : 'inactive'} mr-2"></span>
                    ${user.is_active ? '活跃' : '非活跃'}
                </td>
                <td>
                    <div class="flex space-x-2">
                        <button onclick="adminPanel.toggleUserStatus(${user.id})" 
                                class="btn-ghost text-micro">
                            ${user.is_active ? '禁用' : '启用'}
                        </button>
                        ${user.role !== 'admin' ? `
                            <button onclick="adminPanel.deleteUser(${user.id})" 
                                    class="btn-ghost text-micro text-error">
                                删除
                            </button>
                        ` : ''}
                    </div>
                </td>
            </tr>
        `).join('');
    }

    async loadPlugins() {
        try {
            const response = await this.fetchWithAuth(`${this.baseURL}/admin/plugins`);
            
            if (!response.ok) {
                throw new Error('Failed to load plugins');
            }
            
            const data = await response.json();
            this.plugins = data.plugins || [];
            this.renderPlugins(this.plugins);
            this.updatePluginCount(this.plugins.length);
            
        } catch (error) {
            console.error('Failed to load plugins:', error);
            this.showAlert('加载插件失败', 'error');
        }
    }

    renderPlugins(plugins) {
        const tbody = document.getElementById('pluginsTableBody');
        if (!tbody) return;

        tbody.innerHTML = plugins.map(plugin => `
            <tr>
                <td class="font-mono">${plugin.id}</td>
                <td class="font-mono">${this.escapeHtml(plugin.name)}</td>
                <td class="font-mono">${this.escapeHtml(plugin.author)}</td>
                <td class="font-mono">${this.escapeHtml(plugin.version || 'v1.0.0')}</td>
                <td>${this.escapeHtml(plugin.category || '-')}</td>
                <td class="font-mono">${this.formatNumber(plugin.downloads || 0)}</td>
                <td class="font-mono">${plugin.rating || 0}</td>
                <td>
                    <span class="text-micro px-2 py-1 ${this.getStatusColor(plugin.status)} border">
                        ${this.getStatusText(plugin.status)}
                    </span>
                </td>
                <td>
                    <div class="flex space-x-2">
                        <button onclick="adminPanel.togglePluginStatus(${plugin.id})" 
                                class="btn-ghost text-micro">
                            ${plugin.status === 'active' ? '禁用' : '启用'}
                        </button>
                        <button onclick="adminPanel.deletePlugin(${plugin.id})" 
                                class="btn-ghost text-micro text-error">
                            删除
                        </button>
                    </div>
                </td>
            </tr>
        `).join('');
    }

    getStatusColor(status) {
        switch (status) {
            case 'active': return 'bg-success text-white';
            case 'disabled': return 'bg-error text-white';
            case 'pending': return 'bg-warning text-white';
            default: return 'bg-gray-100';
        }
    }

    getStatusText(status) {
        switch (status) {
            case 'active': return '已启用';
            case 'disabled': return '已禁用';
            case 'pending': return '待审核';
            default: return '未知';
        }
    }

    async executeSql() {
        const query = document.getElementById('sqlQuery')?.value?.trim();
        
        if (!query) {
            this.showAlert('请输入SQL查询语句', 'warning');
            return;
        }

        try {
            const response = await this.fetchWithAuth(`${this.baseURL}/admin/sql/execute`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ query })
            });

            if (!response.ok) {
                throw new Error('SQL execution failed');
            }

            const result = await response.json();
            this.showSqlResults(result);
            
        } catch (error) {
            console.error('SQL execution failed:', error);
            this.showAlert('SQL执行失败', 'error');
        }
    }

    showSqlResults(result) {
        const sqlResults = document.getElementById('sqlResults');
        const sqlResultsContent = document.getElementById('sqlResultsContent');
        
        if (!sqlResults || !sqlResultsContent) return;

        let output = '';
        
        if (result.rows && Array.isArray(result.rows)) {
            // SELECT查询结果
            if (result.rows.length === 0) {
                output = '查询返回0行结果';
            } else {
                output = `查询返回 ${result.rows.length} 行结果:\n\n`;
                output += JSON.stringify(result.rows, null, 2);
            }
        } else if (result.affected_rows !== undefined) {
            // INSERT/UPDATE/DELETE查询结果
            output = `查询执行成功，影响 ${result.affected_rows} 行`;
        } else {
            output = '查询执行成功';
        }

        sqlResultsContent.textContent = output;
        sqlResults.classList.remove('hidden');
        sqlResults.classList.add('fade-in');
    }

    hideSqlResults() {
        const sqlResults = document.getElementById('sqlResults');
        if (sqlResults) {
            sqlResults.classList.add('hidden');
            sqlResults.classList.remove('fade-in');
        }
    }

    // 用户管理方法
    filterUsers(searchTerm) {
        if (!this.users) return;
        
        const filtered = this.users.filter(user => 
            user.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
            user.id.toString().includes(searchTerm)
        );
        
        this.renderUsers(filtered);
        this.updateUserCount(filtered.length);
    }

    async toggleUserStatus(userId) {
        this.showActionModal(
            '切换用户状态',
            '确定要切换该用户的状态吗？',
            () => this.performUserStatusToggle(userId)
        );
    }

    async performUserStatusToggle(userId) {
        try {
            const response = await this.fetchWithAuth(`${this.baseURL}/admin/users/${userId}/toggle`, {
                method: 'POST'
            });

            if (!response.ok) {
                throw new Error('Failed to toggle user status');
            }

            this.showAlert('用户状态已更新', 'success');
            this.loadUsers();
            
        } catch (error) {
            console.error('Failed to toggle user status:', error);
            this.showAlert('操作失败', 'error');
        }
    }

    async deleteUser(userId) {
        this.showActionModal(
            '删除用户',
            '确定要删除该用户吗？此操作无法撤销。',
            () => this.performUserDelete(userId)
        );
    }

    async performUserDelete(userId) {
        try {
            const response = await this.fetchWithAuth(`${this.baseURL}/admin/users/${userId}`, {
                method: 'DELETE'
            });

            if (!response.ok) {
                throw new Error('Failed to delete user');
            }

            this.showAlert('用户已删除', 'success');
            this.loadUsers();
            
        } catch (error) {
            console.error('Failed to delete user:', error);
            this.showAlert('删除失败', 'error');
        }
    }

    // 插件管理方法
    filterPlugins(searchTerm, status) {
        if (!this.plugins) return;
        
        const filtered = this.plugins.filter(plugin => {
            const matchesSearch = !searchTerm || 
                plugin.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                plugin.author.toLowerCase().includes(searchTerm.toLowerCase()) ||
                plugin.id.toString().includes(searchTerm);
            
            const matchesStatus = !status || plugin.status === status;
            
            return matchesSearch && matchesStatus;
        });
        
        this.renderPlugins(filtered);
        this.updatePluginCount(filtered.length);
    }

    async togglePluginStatus(pluginId) {
        this.showActionModal(
            '切换插件状态',
            '确定要切换该插件的状态吗？',
            () => this.performPluginStatusToggle(pluginId)
        );
    }

    async performPluginStatusToggle(pluginId) {
        try {
            const response = await this.fetchWithAuth(`${this.baseURL}/admin/plugins/${pluginId}/toggle`, {
                method: 'POST'
            });

            if (!response.ok) {
                throw new Error('Failed to toggle plugin status');
            }

            this.showAlert('插件状态已更新', 'success');
            this.loadPlugins();
            
        } catch (error) {
            console.error('Failed to toggle plugin status:', error);
            this.showAlert('操作失败', 'error');
        }
    }

    async deletePlugin(pluginId) {
        this.showActionModal(
            '删除插件',
            '确定要删除该插件吗？此操作无法撤销。',
            () => this.performPluginDelete(pluginId)
        );
    }

    async performPluginDelete(pluginId) {
        try {
            const response = await this.fetchWithAuth(`${this.baseURL}/admin/plugins/${pluginId}`, {
                method: 'DELETE'
            });

            if (!response.ok) {
                throw new Error('Failed to delete plugin');
            }

            this.showAlert('插件已删除', 'success');
            this.loadPlugins();
            
        } catch (error) {
            console.error('Failed to delete plugin:', error);
            this.showAlert('删除失败', 'error');
        }
    }

    // 模态框相关
    showActionModal(title, message, onConfirm) {
        const modal = document.getElementById('actionModal');
        const titleElement = document.getElementById('modalTitle');
        const messageElement = document.getElementById('modalMessage');
        const confirmBtn = document.getElementById('confirmAction');
        
        if (titleElement) titleElement.textContent = title;
        if (messageElement) messageElement.textContent = message;
        
        // 移除之前的事件监听器
        const newConfirmBtn = confirmBtn.cloneNode(true);
        confirmBtn.parentNode.replaceChild(newConfirmBtn, confirmBtn);
        
        // 添加新的事件监听器
        newConfirmBtn.addEventListener('click', () => {
            this.hideActionModal();
            onConfirm();
        });
        
        modal.classList.remove('hidden');
        modal.classList.add('fade-in');
    }

    hideActionModal() {
        const modal = document.getElementById('actionModal');
        if (modal) {
            modal.classList.add('hidden');
            modal.classList.remove('fade-in');
        }
    }

    // 工具方法
    async fetchWithAuth(url, options = {}) {
        const defaultOptions = {
            headers: {
                'Authorization': `Bearer ${this.authToken}`,
                'Content-Type': 'application/json'
            }
        };
        
        return fetch(url, { ...defaultOptions, ...options });
    }

    updateUserCount(count) {
        const userCount = document.getElementById('userCount');
        if (userCount) {
            userCount.textContent = count;
        }
    }

    updatePluginCount(count) {
        const pluginCount = document.getElementById('pluginCountAdmin');
        if (pluginCount) {
            pluginCount.textContent = count;
        }
    }

    formatNumber(num) {
        if (num >= 1000000) {
            return (num / 1000000).toFixed(1) + 'M';
        }
        if (num >= 1000) {
            return (num / 1000).toFixed(1) + 'K';
        }
        return num.toString();
    }

    formatDate(dateString) {
        if (!dateString) return '--';
        
        const date = new Date(dateString);
        return date.toLocaleDateString('zh-CN') + ' ' + date.toLocaleTimeString('zh-CN');
    }

    escapeHtml(text) {
        if (!text) return '';
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    showAlert(message, type = 'info') {
        const alertDiv = document.createElement('div');
        alertDiv.className = `alert alert-${type} fixed top-4 right-4 z-50 max-w-sm`;
        alertDiv.textContent = message;
        
        document.body.appendChild(alertDiv);
        
        setTimeout(() => {
            alertDiv.remove();
        }, 3000);
    }

    logout() {
        localStorage.removeItem('auth_token');
        localStorage.removeItem('current_user');
        window.location.href = 'index.html';
    }
}

// 初始化应用
let adminPanel;
document.addEventListener('DOMContentLoaded', () => {
    adminPanel = new AdminPanel();
});

// 全局导出，供HTML中的onclick使用
window.adminPanel = adminPanel;