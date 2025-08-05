// GeekTools Configuration Manager Frontend
class ConfigManager {
    constructor() {
        this.config = window.GeekToolsConfig || {};
        this.baseURL = this.config.apiBaseUrl || '/api/v1';
        this.authToken = localStorage.getItem('auth_token');
        this.currentUser = JSON.parse(localStorage.getItem('current_user') || 'null');
        this.currentTab = 'smtp';
        this.originalConfigs = {};
        this.pendingChanges = {};
        
        this.init();
    }

    init() {
        this.checkAuth();
        this.bindEvents();
        this.loadCurrentConfigs();
        this.updateLastUpdateTime();
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
        // Tab switching
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const tabId = e.target.id.replace('Tab', '');
                this.switchTab(tabId);
            });
        });

        // Logout
        document.getElementById('logoutBtn').addEventListener('click', () => {
            this.logout();
        });

        // Header buttons
        document.getElementById('reloadConfigBtn').addEventListener('click', () => {
            this.reloadConfiguration();
        });

        document.getElementById('backupConfigBtn').addEventListener('click', () => {
            this.createConfigBackup();
        });

        // SMTP Configuration
        document.getElementById('smtpConfigForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.saveSmtpConfig();
        });

        document.getElementById('testSmtpBtn').addEventListener('click', () => {
            this.showTestEmailModal();
        });

        document.getElementById('resetSmtpBtn').addEventListener('click', () => {
            this.resetSmtpConfig();
        });

        document.getElementById('previewSmtpChangesBtn').addEventListener('click', () => {
            this.previewSmtpChanges();
        });

        // Database Configuration
        document.getElementById('databaseConfigForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.saveDatabaseConfig();
        });

        document.getElementById('testDatabaseBtn').addEventListener('click', () => {
            this.testDatabaseConnection();
        });

        document.getElementById('resetDatabaseBtn').addEventListener('click', () => {
            this.resetDatabaseConfig();
        });

        document.getElementById('previewDatabaseChangesBtn').addEventListener('click', () => {
            this.previewDatabaseChanges();
        });

        // Server Configuration
        document.getElementById('serverConfigForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.saveServerConfig();
        });

        document.getElementById('generateJwtSecretBtn').addEventListener('click', () => {
            this.generateJwtSecret();
        });

        document.getElementById('resetServerBtn').addEventListener('click', () => {
            this.resetServerConfig();
        });

        document.getElementById('previewServerChangesBtn').addEventListener('click', () => {
            this.previewServerChanges();
        });

        // Storage Configuration
        document.getElementById('storageConfigForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.saveStorageConfig();
        });

        document.getElementById('testStorageBtn').addEventListener('click', () => {
            this.testStorageUpload();
        });

        document.getElementById('resetStorageBtn').addEventListener('click', () => {
            this.resetStorageConfig();
        });

        document.getElementById('previewStorageChangesBtn').addEventListener('click', () => {
            this.previewStorageChanges();
        });

        // CDN toggle
        document.getElementById('storageUseCdn').addEventListener('change', (e) => {
            const cdnSettings = document.getElementById('cdnSettings');
            if (e.target.checked) {
                cdnSettings.classList.remove('hidden');
            } else {
                cdnSettings.classList.add('hidden');
            }
        });

        // SMTP toggle
        document.getElementById('smtpEnabled').addEventListener('change', (e) => {
            const smtpSettings = document.getElementById('smtpSettings');
            if (e.target.checked) {
                smtpSettings.classList.remove('opacity-50');
                smtpSettings.querySelectorAll('input, select').forEach(input => input.disabled = false);
            } else {
                smtpSettings.classList.add('opacity-50');
                smtpSettings.querySelectorAll('input, select').forEach(input => input.disabled = true);
            }
        });

        // Modal events
        document.getElementById('closePreviewModal').addEventListener('click', () => {
            this.hidePreviewModal();
        });

        document.getElementById('cancelPreview').addEventListener('click', () => {
            this.hidePreviewModal();
        });

        document.getElementById('confirmApplyChanges').addEventListener('click', () => {
            this.applyPendingChanges();
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
    }

    switchTab(tabName) {
        // Remove active class from all tabs
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.classList.remove('tab-active');
        });

        // Hide all tab content
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.add('hidden');
        });

        // Activate selected tab
        document.getElementById(tabName + 'Tab').classList.add('tab-active');
        document.getElementById(tabName + 'Content').classList.remove('hidden');

        this.currentTab = tabName;

        // Load specific tab content
        switch (tabName) {
            case 'backup':
                this.loadBackupManager();
                break;
            case 'history':
                this.loadConfigHistory();
                break;
        }
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

        if (!response.ok) {
            // Try to parse error message from response
            let errorMessage = `HTTP ${response.status}`;
            try {
                const errorData = await response.json();
                errorMessage = errorData.message || errorData.error || errorMessage;
            } catch (e) {
                // If we can't parse JSON, use the status text
                errorMessage = response.statusText || errorMessage;
            }
            throw new Error(errorMessage);
        }

        return response;
    }

    async loadCurrentConfigs() {
        try {
            this.showLoading('加载配置中...');
            
            const response = await this.makeAuthenticatedRequest(`${this.baseURL}/admin/config`);
            if (!response) return;

            const result = await response.json();
            if (result.success && result.data) {
                this.originalConfigs = result.data;
                this.populateConfigForms(result.data);
                this.hideLoading();
            } else {
                throw new Error(result.message || '获取配置失败');
            }
            
        } catch (error) {
            console.error('Failed to load configs:', error);
            this.showError('加载配置失败: ' + error.message);
            this.hideLoading();
        }
    }

    populateConfigForms(config) {
        // Populate SMTP form
        if (config.smtp) {
            document.getElementById('smtpEnabled').checked = config.smtp.enabled;
            document.getElementById('smtpHost').value = config.smtp.host;
            document.getElementById('smtpPort').value = config.smtp.port;
            document.getElementById('smtpUsername').value = config.smtp.username;
            document.getElementById('smtpPassword').value = config.smtp.password;
            document.getElementById('smtpFromAddress').value = config.smtp.from_address;
            document.getElementById('smtpFromName').value = config.smtp.from_name;
            document.getElementById('smtpUseTls').checked = config.smtp.use_tls;
            
            // Trigger SMTP enable/disable
            document.getElementById('smtpEnabled').dispatchEvent(new Event('change'));
        }

        // Populate Database form
        if (config.database) {
            document.getElementById('databaseUrl').value = config.database.url;
            document.getElementById('databaseMaxConnections').value = config.database.max_connections;
            document.getElementById('databaseConnectTimeout').value = config.database.connect_timeout;
        }

        // Populate Server form
        if (config.server) {
            document.getElementById('serverHost').value = config.server.host;
            document.getElementById('serverPort').value = config.server.port;
            document.getElementById('jwtSecret').value = config.server.jwt_secret;
            document.getElementById('jwtAccessTokenExpiry').value = config.server.jwt_access_token_expires_in;
            document.getElementById('jwtRefreshTokenExpiry').value = config.server.jwt_refresh_token_expires_in;
            document.getElementById('corsOrigins').value = config.server.cors_origins;
        }

        // Populate Storage form
        if (config.storage) {
            document.getElementById('storageUploadPath').value = config.storage.upload_path;
            document.getElementById('storageMaxFileSize').value = config.storage.max_file_size;
            document.getElementById('storageUseCdn').checked = config.storage.use_cdn;
            document.getElementById('storageCdnBaseUrl').value = config.storage.cdn_base_url;
            
            // Trigger CDN enable/disable
            document.getElementById('storageUseCdn').dispatchEvent(new Event('change'));
        }
    }

    // SMTP Configuration Methods
    async saveSmtpConfig() {
        const smtpConfig = this.getSmtpConfigFromForm();
        
        if (!this.validateSmtpConfig(smtpConfig)) {
            return;
        }

        try {
            this.showButtonLoading('saveSmtpBtn', '保存中...');
            
            const response = await this.makeAuthenticatedRequest(`${this.baseURL}/admin/config/update`, {
                method: 'POST',
                body: JSON.stringify({
                    config_type: 'smtp',
                    config_data: smtpConfig
                })
            });

            if (!response) return;

            const result = await response.json();
            if (result.success) {
                this.originalConfigs.smtp = smtpConfig;
                this.showSuccess('SMTP配置保存成功');
                this.updateLastUpdateTime();
            } else {
                throw new Error(result.message || '保存失败');
            }
            
        } catch (error) {
            console.error('Failed to save SMTP config:', error);
            this.showError('保存SMTP配置失败: ' + error.message);
        } finally {
            this.hideButtonLoading('saveSmtpBtn', '保存并应用');
        }
    }

    getSmtpConfigFromForm() {
        return {
            enabled: document.getElementById('smtpEnabled').checked,
            host: document.getElementById('smtpHost').value.trim(),
            port: parseInt(document.getElementById('smtpPort').value),
            username: document.getElementById('smtpUsername').value.trim(),
            password: document.getElementById('smtpPassword').value.trim(),
            from_address: document.getElementById('smtpFromAddress').value.trim(),
            from_name: document.getElementById('smtpFromName').value.trim(),
            use_tls: document.getElementById('smtpUseTls').checked
        };
    }

    validateSmtpConfig(config) {
        if (config.enabled) {
            if (!config.host) {
                this.showError('请输入SMTP服务器地址');
                return false;
            }
            if (!config.username) {
                this.showError('请输入SMTP用户名');
                return false;
            }
            if (!config.password) {
                this.showError('请输入SMTP密码');
                return false;
            }
            if (!config.from_address) {
                this.showError('请输入发件人邮箱');
                return false;
            }
            if (!this.isValidEmail(config.from_address)) {
                this.showError('发件人邮箱格式不正确');
                return false;
            }
            if (config.username && !this.isValidEmail(config.username)) {
                this.showError('用户名邮箱格式不正确');
                return false;
            }
        }
        return true;
    }

    resetSmtpConfig() {
        if (confirm('确定要重置SMTP配置吗？这将恢复到上次保存的状态。')) {
            if (this.originalConfigs.smtp) {
                this.populateSmtpForm(this.originalConfigs.smtp);
                this.showSuccess('SMTP配置已重置');
            }
        }
    }

    populateSmtpForm(config) {
        document.getElementById('smtpEnabled').checked = config.enabled;
        document.getElementById('smtpHost').value = config.host;
        document.getElementById('smtpPort').value = config.port;
        document.getElementById('smtpUsername').value = config.username;
        document.getElementById('smtpPassword').value = config.password;
        document.getElementById('smtpFromAddress').value = config.from_address;
        document.getElementById('smtpFromName').value = config.from_name;
        document.getElementById('smtpUseTls').checked = config.use_tls;
        
        document.getElementById('smtpEnabled').dispatchEvent(new Event('change'));
    }

    previewSmtpChanges() {
        const currentConfig = this.getSmtpConfigFromForm();
        const originalConfig = this.originalConfigs.smtp;
        
        this.pendingChanges = { smtp: currentConfig };
        this.showPreviewModal('SMTP配置', originalConfig, currentConfig);
    }

    // Database Configuration Methods
    async saveDatabaseConfig() {
        const dbConfig = this.getDatabaseConfigFromForm();
        
        if (!this.validateDatabaseConfig(dbConfig)) {
            return;
        }

        try {
            this.showButtonLoading('saveDatabaseBtn', '保存中...');
            
            const response = await this.makeAuthenticatedRequest(`${this.baseURL}/admin/config/update`, {
                method: 'POST',
                body: JSON.stringify({
                    config_type: 'database',
                    config_data: {
                        max_connections: dbConfig.max_connections,
                        connect_timeout: dbConfig.connect_timeout
                    }
                })
            });

            if (!response) return;

            const result = await response.json();
            if (result.success) {
                this.originalConfigs.database = dbConfig;
                this.showSuccess('数据库配置保存成功');
                this.updateLastUpdateTime();
            } else {
                throw new Error(result.message || '保存失败');
            }
            
        } catch (error) {
            console.error('Failed to save database config:', error);
            this.showError('保存数据库配置失败: ' + error.message);
        } finally {
            this.hideButtonLoading('saveDatabaseBtn', '保存并应用');
        }
    }

    getDatabaseConfigFromForm() {
        return {
            url: document.getElementById('databaseUrl').value.trim(),
            max_connections: parseInt(document.getElementById('databaseMaxConnections').value),
            connect_timeout: parseInt(document.getElementById('databaseConnectTimeout').value)
        };
    }

    validateDatabaseConfig(config) {
        if (!config.url) {
            this.showError('请输入数据库连接URL');
            return false;
        }
        if (config.max_connections < 1 || config.max_connections > 100) {
            this.showError('最大连接数必须在1-100之间');
            return false;
        }
        if (config.connect_timeout < 5 || config.connect_timeout > 300) {
            this.showError('连接超时必须在5-300秒之间');
            return false;
        }
        return true;
    }

    async testDatabaseConnection() {
        try {
            this.showButtonLoading('testDatabaseBtn', '测试中...');
            
            const response = await this.makeAuthenticatedRequest(`${this.baseURL}/admin/config/test`, {
                method: 'POST',
                body: JSON.stringify({
                    config_type: 'database'
                })
            });

            if (!response) return;

            const result = await response.json();
            if (result.success && result.data) {
                if (result.data.status === 'success') {
                    this.showSuccess('数据库连接测试成功');
                } else {
                    this.showError(`数据库连接测试失败: ${result.data.message}`);
                }
            } else {
                throw new Error(result.message || '测试失败');
            }
            
        } catch (error) {
            console.error('Database connection test failed:', error);
            this.showError('数据库连接测试失败: ' + error.message);
        } finally {
            this.hideButtonLoading('testDatabaseBtn', '测试连接');
        }
    }

    resetDatabaseConfig() {
        if (confirm('确定要重置数据库配置吗？')) {
            if (this.originalConfigs.database) {
                this.populateDatabaseForm(this.originalConfigs.database);
                this.showSuccess('数据库配置已重置');
            }
        }
    }

    populateDatabaseForm(config) {
        document.getElementById('databaseUrl').value = config.url;
        document.getElementById('databaseMaxConnections').value = config.max_connections;
        document.getElementById('databaseConnectTimeout').value = config.connect_timeout;
    }

    previewDatabaseChanges() {
        const currentConfig = this.getDatabaseConfigFromForm();
        const originalConfig = this.originalConfigs.database;
        
        this.pendingChanges = { database: currentConfig };
        this.showPreviewModal('数据库配置', originalConfig, currentConfig);
    }

    // Server Configuration Methods
    async saveServerConfig() {
        const serverConfig = this.getServerConfigFromForm();
        
        if (!this.validateServerConfig(serverConfig)) {
            return;
        }

        try {
            this.showButtonLoading('saveServerBtn', '保存中...');
            
            const response = await this.makeAuthenticatedRequest(`${this.baseURL}/admin/config/update`, {
                method: 'POST',
                body: JSON.stringify({
                    config_type: 'server',
                    config_data: {
                        host: serverConfig.host,
                        port: serverConfig.port,
                        jwt_secret: serverConfig.jwt_secret,
                        jwt_access_token_expires_in: serverConfig.jwt_access_token_expires_in,
                        jwt_refresh_token_expires_in: serverConfig.jwt_refresh_token_expires_in,
                        cors_origins: serverConfig.cors_origins
                    }
                })
            });

            if (!response) return;

            const result = await response.json();
            if (result.success) {
                this.originalConfigs.server = serverConfig;
                this.showSuccess('服务器配置保存成功');
                this.updateLastUpdateTime();
            } else {
                throw new Error(result.message || '保存失败');
            }
            
        } catch (error) {
            console.error('Failed to save server config:', error);
            this.showError('保存服务器配置失败: ' + error.message);
        } finally {
            this.hideButtonLoading('saveServerBtn', '保存并应用');
        }
    }

    getServerConfigFromForm() {
        return {
            host: document.getElementById('serverHost').value.trim(),
            port: parseInt(document.getElementById('serverPort').value),
            jwt_secret: document.getElementById('jwtSecret').value.trim(),
            jwt_access_token_expires_in: parseInt(document.getElementById('jwtAccessTokenExpiry').value),
            jwt_refresh_token_expires_in: parseInt(document.getElementById('jwtRefreshTokenExpiry').value),
            cors_origins: document.getElementById('corsOrigins').value.trim()
        };
    }

    validateServerConfig(config) {
        if (!config.host) {
            this.showError('请输入服务器绑定地址');
            return false;
        }
        if (config.port < 1024 || config.port > 65535) {
            this.showError('端口号必须在1024-65535之间');
            return false;
        }
        if (!config.jwt_secret || config.jwt_secret.length < 32) {
            this.showError('JWT密钥长度不能少于32个字符');
            return false;
        }
        if (config.jwt_access_token_expires_in < 300 || config.jwt_access_token_expires_in > 86400) {
            this.showError('访问令牌过期时间必须在300-86400秒之间');
            return false;
        }
        return true;
    }

    generateJwtSecret() {
        const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*';
        let result = '';
        for (let i = 0; i < 64; i++) {
            result += characters.charAt(Math.floor(Math.random() * characters.length));
        }
        
        document.getElementById('jwtSecret').value = result;
        this.showSuccess('JWT密钥已生成');
    }

    resetServerConfig() {
        if (confirm('确定要重置服务器配置吗？')) {
            if (this.originalConfigs.server) {
                this.populateServerForm(this.originalConfigs.server);
                this.showSuccess('服务器配置已重置');
            }
        }
    }

    populateServerForm(config) {
        document.getElementById('serverHost').value = config.host;
        document.getElementById('serverPort').value = config.port;
        document.getElementById('jwtSecret').value = config.jwt_secret;
        document.getElementById('jwtAccessTokenExpiry').value = config.jwt_access_token_expires_in;
        document.getElementById('jwtRefreshTokenExpiry').value = config.jwt_refresh_token_expires_in;
        document.getElementById('corsOrigins').value = config.cors_origins;
    }

    previewServerChanges() {
        const currentConfig = this.getServerConfigFromForm();
        const originalConfig = this.originalConfigs.server;
        
        this.pendingChanges = { server: currentConfig };
        this.showPreviewModal('服务器配置', originalConfig, currentConfig);
    }

    // Storage Configuration Methods
    async saveStorageConfig() {
        const storageConfig = this.getStorageConfigFromForm();
        
        if (!this.validateStorageConfig(storageConfig)) {
            return;
        }

        try {
            this.showButtonLoading('saveStorageBtn', '保存中...');
            
            const response = await this.makeAuthenticatedRequest(`${this.baseURL}/admin/config/update`, {
                method: 'POST',
                body: JSON.stringify({
                    config_type: 'storage',
                    config_data: storageConfig
                })
            });

            if (!response) return;

            const result = await response.json();
            if (result.success) {
                this.originalConfigs.storage = storageConfig;
                this.showSuccess('存储配置保存成功');
                this.updateLastUpdateTime();
            } else {
                throw new Error(result.message || '保存失败');
            }
            
        } catch (error) {
            console.error('Failed to save storage config:', error);
            this.showError('保存存储配置失败: ' + error.message);
        } finally {
            this.hideButtonLoading('saveStorageBtn', '保存并应用');
        }
    }

    getStorageConfigFromForm() {
        return {
            upload_path: document.getElementById('storageUploadPath').value.trim(),
            max_file_size: parseInt(document.getElementById('storageMaxFileSize').value),
            use_cdn: document.getElementById('storageUseCdn').checked,
            cdn_base_url: document.getElementById('storageCdnBaseUrl').value.trim()
        };
    }

    validateStorageConfig(config) {
        if (!config.upload_path) {
            this.showError('请输入上传文件路径');
            return false;
        }
        if (config.max_file_size < 1 || config.max_file_size > 1000) {
            this.showError('最大文件大小必须在1-1000MB之间');
            return false;
        }
        if (config.use_cdn && !config.cdn_base_url) {
            this.showError('启用CDN时必须输入CDN基础URL');
            return false;
        }
        if (config.use_cdn && !this.isValidUrl(config.cdn_base_url)) {
            this.showError('CDN基础URL格式不正确');
            return false;
        }
        return true;
    }

    async testStorageUpload() {
        try {
            this.showButtonLoading('testStorageBtn', '测试中...');
            
            const response = await this.makeAuthenticatedRequest(`${this.baseURL}/admin/config/test`, {
                method: 'POST',
                body: JSON.stringify({
                    config_type: 'storage'
                })
            });

            if (!response) return;

            const result = await response.json();
            if (result.success && result.data) {
                if (result.data.status === 'success') {
                    this.showSuccess('存储上传测试成功');
                } else {
                    this.showError(`存储上传测试失败: ${result.data.message}`);
                }
            } else {
                throw new Error(result.message || '测试失败');
            }
            
        } catch (error) {
            console.error('Storage upload test failed:', error);
            this.showError('存储上传测试失败: ' + error.message);
        } finally {
            this.hideButtonLoading('testStorageBtn', '测试上传');
        }
    }

    resetStorageConfig() {
        if (confirm('确定要重置存储配置吗？')) {
            if (this.originalConfigs.storage) {
                this.populateStorageForm(this.originalConfigs.storage);
                this.showSuccess('存储配置已重置');
            }
        }
    }

    populateStorageForm(config) {
        document.getElementById('storageUploadPath').value = config.upload_path;
        document.getElementById('storageMaxFileSize').value = config.max_file_size;
        document.getElementById('storageUseCdn').checked = config.use_cdn;
        document.getElementById('storageCdnBaseUrl').value = config.cdn_base_url;
        
        document.getElementById('storageUseCdn').dispatchEvent(new Event('change'));
    }

    previewStorageChanges() {
        const currentConfig = this.getStorageConfigFromForm();
        const originalConfig = this.originalConfigs.storage;
        
        this.pendingChanges = { storage: currentConfig };
        this.showPreviewModal('存储配置', originalConfig, currentConfig);
    }

    // Test Email Methods
    showTestEmailModal() {
        const smtpConfig = this.getSmtpConfigFromForm();
        
        if (!this.validateSmtpConfig(smtpConfig)) {
            return;
        }

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
        const subject = document.getElementById('testEmailSubject').value.trim() || 'GeekTools 配置测试邮件';
        const body = document.getElementById('testEmailBody')?.value?.trim() || '这是来自 GeekTools 插件市场配置系统的测试邮件。';
        
        if (!recipient || !this.isValidEmail(recipient)) {
            this.showError('请输入有效的收件人邮箱');
            return;
        }

        try {
            this.showButtonLoading('sendTestEmailBtn', '发送中...');
            
            const response = await this.makeAuthenticatedRequest(`${this.baseURL}/admin/config/test/email`, {
                method: 'POST',
                body: JSON.stringify({
                    recipient: recipient,
                    subject: subject,
                    body: body
                })
            });

            if (!response) return;

            const result = await response.json();
            if (result.success) {
                this.showSuccess(`测试邮件已发送到 ${recipient}`);
                this.hideTestEmailModal();
            } else {
                throw new Error(result.message || '发送失败');
            }
            
        } catch (error) {
            console.error('Failed to send test email:', error);
            this.showError('发送测试邮件失败: ' + error.message);
        } finally {
            this.hideButtonLoading('sendTestEmailBtn', '发送邮件');
        }
    }

    // Preview and Apply Changes
    showPreviewModal(configType, originalConfig, currentConfig) {
        const changes = this.generateConfigDiff(originalConfig, currentConfig);
        
        if (changes.length === 0) {
            this.showInfo('没有检测到配置变更');
            return;
        }

        const previewContent = document.getElementById('previewContent');
        previewContent.innerHTML = `
            <h4 class="font-semibold mb-3">${configType}变更预览</h4>
            <div class="space-y-2">
                ${changes.map(change => `
                    <div class="flex items-center space-x-2 text-sm">
                        <span class="font-medium">${change.field}:</span>
                        ${change.type === 'modified' ? `
                            <span class="diff-removed px-2 py-1 rounded">${change.oldValue}</span>
                            <span class="text-gray-500">→</span>
                            <span class="diff-added px-2 py-1 rounded">${change.newValue}</span>
                        ` : change.type === 'added' ? `
                            <span class="diff-added px-2 py-1 rounded">+ ${change.newValue}</span>
                        ` : `
                            <span class="diff-removed px-2 py-1 rounded">- ${change.oldValue}</span>
                        `}
                    </div>
                `).join('')}
            </div>
        `;

        document.getElementById('previewModal').classList.remove('hidden');
        document.body.style.overflow = 'hidden';
    }

    hidePreviewModal() {
        document.getElementById('previewModal').classList.add('hidden');
        document.body.style.overflow = 'auto';
        this.pendingChanges = {};
    }

    async applyPendingChanges() {
        if (Object.keys(this.pendingChanges).length === 0) {
            this.hidePreviewModal();
            return;
        }

        try {
            this.showButtonLoading('confirmApplyChanges', '应用中...');
            
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 1500));
            
            // Apply changes to original configs
            Object.assign(this.originalConfigs, this.pendingChanges);
            
            this.showSuccess('配置变更已应用');
            this.hidePreviewModal();
            this.updateLastUpdateTime();
            
        } catch (error) {
            console.error('Failed to apply changes:', error);
            this.showError('应用配置变更失败');
        } finally {
            this.hideButtonLoading('confirmApplyChanges', '确认应用');
        }
    }

    generateConfigDiff(original, current) {
        const changes = [];
        
        // Compare all fields
        const allKeys = new Set([...Object.keys(original || {}), ...Object.keys(current || {})]);
        
        allKeys.forEach(key => {
            const oldValue = original?.[key];
            const newValue = current?.[key];
            
            if (oldValue !== newValue) {
                if (oldValue === undefined) {
                    changes.push({
                        field: this.getFieldDisplayName(key),
                        type: 'added',
                        newValue: this.formatConfigValue(newValue)
                    });
                } else if (newValue === undefined) {
                    changes.push({
                        field: this.getFieldDisplayName(key),
                        type: 'removed',
                        oldValue: this.formatConfigValue(oldValue)
                    });
                } else {
                    changes.push({
                        field: this.getFieldDisplayName(key),
                        type: 'modified',
                        oldValue: this.formatConfigValue(oldValue),
                        newValue: this.formatConfigValue(newValue)
                    });
                }
            }
        });
        
        return changes;
    }

    getFieldDisplayName(fieldName) {
        const fieldNames = {
            'enabled': '启用状态',
            'host': '服务器地址',
            'port': '端口',
            'username': '用户名',
            'password': '密码',
            'from_address': '发件人邮箱',
            'from_name': '发件人名称',
            'use_tls': 'TLS加密',
            'url': '连接URL',
            'max_connections': '最大连接数',
            'connect_timeout': '连接超时',
            'jwt_secret': 'JWT密钥',
            'jwt_access_token_expires_in': '访问令牌过期时间',
            'jwt_refresh_token_expires_in': '刷新令牌过期时间',
            'cors_origins': 'CORS源',
            'upload_path': '上传路径',
            'max_file_size': '最大文件大小',
            'use_cdn': 'CDN启用',
            'cdn_base_url': 'CDN基础URL'
        };
        
        return fieldNames[fieldName] || fieldName;
    }

    formatConfigValue(value) {
        if (typeof value === 'boolean') {
            return value ? '是' : '否';
        }
        if (typeof value === 'string' && (value.includes('password') || value.includes('secret'))) {
            return '***';
        }
        if (typeof value === 'string' && value.length > 50) {
            return value.substring(0, 47) + '...';
        }
        return String(value);
    }

    // Configuration Management
    async reloadConfiguration() {
        if (confirm('确定要重新加载配置吗？未保存的变更将丢失。')) {
            try {
                this.showLoading('重新加载配置中...');
                await this.loadCurrentConfigs();
                this.showSuccess('配置已重新加载');
            } catch (error) {
                this.showError('重新加载配置失败');
            }
        }
    }

    async createConfigBackup() {
        try {
            this.showButtonLoading('backupConfigBtn', '备份中...');
            
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 1000));
            
            const backupData = JSON.stringify(this.originalConfigs, null, 2);
            const blob = new Blob([backupData], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            
            const a = document.createElement('a');
            a.href = url;
            a.download = `config-backup-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.json`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            
            this.showSuccess('配置备份已下载');
            
        } catch (error) {
            console.error('Failed to create backup:', error);
            this.showError('创建配置备份失败');
        } finally {
            this.hideButtonLoading('backupConfigBtn', '备份配置');
        }
    }

    updateLastUpdateTime() {
        const now = new Date();
        document.getElementById('lastUpdateTime').textContent = now.toLocaleString('zh-CN');
    }

    // Utility Methods
    togglePassword(fieldId) {
        const field = document.getElementById(fieldId);
        const toggle = field.nextElementSibling;
        
        if (field.type === 'password') {
            field.type = 'text';
            toggle.classList.remove('fa-eye');
            toggle.classList.add('fa-eye-slash');
        } else {
            field.type = 'password';
            toggle.classList.remove('fa-eye-slash');
            toggle.classList.add('fa-eye');
        }
    }

    isValidEmail(email) {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(email);
    }

    isValidUrl(url) {
        try {
            new URL(url);
            return true;
        } catch {
            return false;
        }
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

    showLoading(message) {
        // Implementation depends on your loading component
        console.log('Loading:', message);
    }

    hideLoading() {
        // Implementation depends on your loading component
        console.log('Loading hidden');
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
                document.body.removeChild(notification);
            }, 300);
        }, 3000);
    }

    loadBackupManager() {
        // Load backup manager content
        document.getElementById('backupContent').innerHTML = `
            <div class="gradient-card rounded-2xl p-6 border border-white/20">
                <div class="config-section mb-6">
                    <h3 class="text-xl font-semibold mb-2">数据库备份管理</h3>
                    <p class="text-gray-600 text-sm">管理数据库备份和恢复操作</p>
                </div>
                <div class="text-center py-8">
                    <i class="fas fa-archive text-gray-400 text-4xl mb-4"></i>
                    <p class="text-gray-500">备份管理功能开发中...</p>
                </div>
            </div>
        `;
    }

    loadConfigHistory() {
        // Load configuration history content
        document.getElementById('historyContent').innerHTML = `
            <div class="gradient-card rounded-2xl p-6 border border-white/20">
                <div class="config-section mb-6">
                    <h3 class="text-xl font-semibold mb-2">配置变更历史</h3>
                    <p class="text-gray-600 text-sm">查看和回滚配置变更记录</p>
                </div>
                <div class="text-center py-8">
                    <i class="fas fa-history text-gray-400 text-4xl mb-4"></i>
                    <p class="text-gray-500">配置历史功能开发中...</p>
                </div>
            </div>
        `;
    }

    logout() {
        localStorage.removeItem('auth_token');
        localStorage.removeItem('current_user');
        window.location.href = 'index.html';
    }
}

// Initialize configuration manager when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.configManager = new ConfigManager();
});