// GeekTools Configuration History Manager Frontend
class ConfigHistoryManager {
    constructor() {
        this.config = window.GeekToolsConfig || {};
        this.baseURL = this.config.apiBaseUrl || '/api/v1';
        this.authToken = localStorage.getItem('auth_token');
        this.currentUser = JSON.parse(localStorage.getItem('current_user') || 'null');
        this.configHistory = [];
        this.pagination = { page: 1, limit: 20 };
        this.filters = { configType: '', timeRange: '' };
        
        this.init();
    }

    init() {
        this.checkAuth();
        this.bindEvents();
        this.loadConfigHistory();
        this.loadStatistics();
        this.generateSnapshotName();
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

        // Action buttons
        document.getElementById('createSnapshotBtn').addEventListener('click', () => {
            this.showCreateSnapshotModal();
        });

        document.getElementById('exportHistoryBtn').addEventListener('click', () => {
            this.exportHistory();
        });

        document.getElementById('importHistoryBtn').addEventListener('click', () => {
            this.importHistory();
        });

        document.getElementById('refreshHistoryBtn').addEventListener('click', () => {
            this.loadConfigHistory();
        });

        // Filters
        document.getElementById('configTypeFilter').addEventListener('change', () => {
            this.filters.configType = document.getElementById('configTypeFilter').value;
            this.pagination.page = 1;
            this.loadConfigHistory();
        });

        document.getElementById('timeRangeFilter').addEventListener('change', () => {
            this.filters.timeRange = document.getElementById('timeRangeFilter').value;
            this.pagination.page = 1;
            this.loadConfigHistory();
        });

        // Quick actions
        document.getElementById('compareConfigsBtn').addEventListener('click', () => {
            this.showCompareModal();
        });

        document.getElementById('rollbackToStableBtn').addEventListener('click', () => {
            this.rollbackToStable();
        });

        document.getElementById('scheduleRollbackBtn').addEventListener('click', () => {
            this.scheduleRollback();
        });

        document.getElementById('cleanupOldVersionsBtn').addEventListener('click', () => {
            this.cleanupOldVersions();
        });

        document.getElementById('downloadCurrentConfigBtn').addEventListener('click', () => {
            this.downloadCurrentConfig();
        });

        // Pagination
        document.getElementById('prevPage').addEventListener('click', () => {
            if (this.pagination.page > 1) {
                this.pagination.page--;
                this.loadConfigHistory();
            }
        });

        document.getElementById('nextPage').addEventListener('click', () => {
            this.pagination.page++;
            this.loadConfigHistory();
        });

        // Create Snapshot Modal
        document.getElementById('closeCreateSnapshotModal').addEventListener('click', () => {
            this.hideCreateSnapshotModal();
        });

        document.getElementById('cancelCreateSnapshot').addEventListener('click', () => {
            this.hideCreateSnapshotModal();
        });

        document.getElementById('createSnapshotForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.createSnapshot();
        });

        // Rollback Modal
        document.getElementById('closeRollbackModal').addEventListener('click', () => {
            this.hideRollbackModal();
        });

        document.getElementById('cancelRollback').addEventListener('click', () => {
            this.hideRollbackModal();
        });

        document.getElementById('confirmRollback').addEventListener('click', () => {
            this.performRollback();
        });

        // Compare Modal
        document.getElementById('closeCompareModal').addEventListener('click', () => {
            this.hideCompareModal();
        });

        document.getElementById('compareVersionA').addEventListener('change', () => {
            this.performVersionComparison();
        });

        document.getElementById('compareVersionB').addEventListener('change', () => {
            this.performVersionComparison();
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

    async loadConfigHistory() {
        try {
            document.getElementById('timelineLoading').classList.remove('hidden');
            document.getElementById('timelineEmpty').classList.add('hidden');
            
            // Mock configuration history data
            const mockHistory = this.generateMockHistory();
            
            this.configHistory = mockHistory;
            this.renderTimeline(mockHistory);
            this.updatePagination(mockHistory.length);
            
        } catch (error) {
            console.error('Failed to load config history:', error);
            this.showError('加载配置历史失败');
            document.getElementById('timelineEmpty').classList.remove('hidden');
        } finally {
            document.getElementById('timelineLoading').classList.add('hidden');
        }
    }

    generateMockHistory() {
        const history = [];
        const now = new Date();
        const configTypes = ['smtp', 'database', 'server', 'storage'];
        const changeTypes = ['create', 'update', 'rollback', 'snapshot'];
        const authors = [this.currentUser.email, 'admin@geektools.dev', 'system@geektools.dev'];
        
        for (let i = 0; i < 25; i++) {
            const timestamp = new Date(now.getTime() - (i * 6 * 60 * 60 * 1000)); // Every 6 hours
            const configType = configTypes[Math.floor(Math.random() * configTypes.length)];
            const changeType = changeTypes[Math.floor(Math.random() * changeTypes.length)];
            const author = authors[Math.floor(Math.random() * authors.length)];
            
            history.push({
                id: `config_${i + 1}`,
                version: `v2.${Math.floor((25 - i) / 5)}.${(25 - i) % 5}`,
                config_type: configType,
                change_type: changeType,
                title: this.getChangeTitle(configType, changeType),
                description: this.getChangeDescription(configType, changeType),
                author: author,
                timestamp: timestamp.toISOString(),
                is_current: i === 0,
                is_stable: changeType === 'snapshot' || (i % 8 === 0),
                file_size: Math.floor(Math.random() * 50 + 10), // 10-60 KB
                changes_count: Math.floor(Math.random() * 15 + 1),
                checksum: this.generateChecksum()
            });
        }
        
        return history;
    }

    getChangeTitle(configType, changeType) {
        const titles = {
            smtp: {
                create: '初始化SMTP配置',
                update: '更新SMTP服务器设置',
                rollback: '回滚SMTP配置',
                snapshot: 'SMTP配置快照'
            },
            database: {
                create: '初始化数据库配置',
                update: '更新数据库连接池',
                rollback: '回滚数据库配置',
                snapshot: '数据库配置快照'
            },
            server: {
                create: '初始化服务器配置',
                update: '更新服务器参数',
                rollback: '回滚服务器配置',
                snapshot: '服务器配置快照'
            },
            storage: {
                create: '初始化存储配置',
                update: '更新存储设置',
                rollback: '回滚存储配置',
                snapshot: '存储配置快照'
            }
        };
        
        return titles[configType]?.[changeType] || '配置变更';
    }

    getChangeDescription(configType, changeType) {
        const descriptions = {
            smtp: {
                create: '设置SMTP服务器、端口和认证信息',
                update: '修改SMTP服务器地址和TLS设置',
                rollback: '由于连接问题回滚到之前版本',
                snapshot: '发布前创建SMTP配置备份'
            },
            database: {
                create: '配置PostgreSQL连接参数',
                update: '调整连接池大小和超时设置',
                rollback: '性能问题回滚数据库配置',
                snapshot: '维护前创建数据库配置备份'
            },
            server: {
                create: '设置服务器端口和CORS策略',
                update: '更新JWT过期时间和安全设置',
                rollback: '安全问题回滚服务器配置',
                snapshot: '升级前创建服务器配置备份'
            },
            storage: {
                create: '配置文件上传路径和大小限制',
                update: '启用CDN和调整存储策略',
                rollback: '存储问题回滚配置',
                snapshot: '迁移前创建存储配置备份'
            }
        };
        
        return descriptions[configType]?.[changeType] || '配置文件变更';
    }

    generateChecksum() {
        return Math.random().toString(36).substring(2, 10);
    }

    renderTimeline(history) {
        const container = document.getElementById('configTimeline');
        
        if (history.length === 0) {
            document.getElementById('timelineEmpty').classList.remove('hidden');
            container.innerHTML = '';
            return;
        }

        container.innerHTML = history.map((item, index) => `
            <div class="timeline-item ${item.is_current ? 'current' : ''} ${item.change_type === 'rollback' ? 'rollback' : ''}">
                <div class="history-card gradient-card rounded-xl p-4 border border-white/20">
                    <div class="flex items-start justify-between mb-3">
                        <div class="flex-1">
                            <div class="flex items-center space-x-3 mb-2">
                                <h4 class="font-semibold text-claude-text">${this.escapeHtml(item.title)}</h4>
                                <span class="version-tag ${this.getVersionTypeClass(item.version)} px-2 py-1 text-xs text-white rounded-full">
                                    ${item.version}
                                </span>
                                <span class="px-2 py-1 text-xs rounded-full ${this.getChangeTypeStyle(item.change_type)}">
                                    ${this.getChangeTypeText(item.change_type)}
                                </span>
                                ${item.is_current ? '<span class="px-2 py-1 text-xs bg-claude-orange text-white rounded-full">当前</span>' : ''}
                                ${item.is_stable ? '<span class="px-2 py-1 text-xs bg-green-100 text-green-700 rounded-full">稳定</span>' : ''}
                            </div>
                            <p class="text-sm text-gray-600 mb-3">${this.escapeHtml(item.description)}</p>
                            <div class="flex items-center space-x-4 text-xs text-gray-500">
                                <span><i class="fas fa-user mr-1"></i>${this.escapeHtml(item.author)}</span>
                                <span><i class="fas fa-clock mr-1"></i>${this.formatDateTime(item.timestamp)}</span>
                                <span><i class="fas fa-edit mr-1"></i>${item.changes_count}处变更</span>
                                <span><i class="fas fa-file mr-1"></i>${item.file_size} KB</span>
                                <span><i class="fas fa-fingerprint mr-1"></i>${item.checksum}</span>
                            </div>
                        </div>
                        <div class="flex items-center space-x-2 ml-4">
                            ${!item.is_current ? `
                                <button onclick="configHistoryManager.showRollbackModal('${item.id}')" 
                                        class="px-3 py-2 bg-blue-500 text-white text-sm rounded-lg hover:bg-blue-600 transition-colors"
                                        title="回滚到此版本">
                                    <i class="fas fa-undo"></i>
                                </button>
                            ` : ''}
                            <button onclick="configHistoryManager.downloadVersion('${item.id}')" 
                                    class="px-3 py-2 bg-green-500 text-white text-sm rounded-lg hover:bg-green-600 transition-colors"
                                    title="下载此版本">
                                <i class="fas fa-download"></i>
                            </button>
                            <button onclick="configHistoryManager.viewVersionDetails('${item.id}')" 
                                    class="px-3 py-2 bg-purple-500 text-white text-sm rounded-lg hover:bg-purple-600 transition-colors"
                                    title="查看详情">
                                <i class="fas fa-eye"></i>
                            </button>
                            ${!item.is_current && !item.is_stable ? `
                                <button onclick="configHistoryManager.deleteVersion('${item.id}')" 
                                        class="px-3 py-2 bg-red-500 text-white text-sm rounded-lg hover:bg-red-600 transition-colors"
                                        title="删除版本">
                                    <i class="fas fa-trash"></i>
                                </button>
                            ` : ''}
                        </div>
                    </div>
                </div>
            </div>
        `).join('');
    }

    getVersionTypeClass(version) {
        const parts = version.replace('v', '').split('.');
        const minor = parseInt(parts[1] || 0);
        const patch = parseInt(parts[2] || 0);
        
        if (patch === 0 && minor === 0) return 'major';
        if (patch === 0) return 'minor';
        return 'patch';
    }

    getChangeTypeStyle(changeType) {
        switch (changeType) {
            case 'create': return 'bg-green-100 text-green-700';
            case 'update': return 'bg-blue-100 text-blue-700';
            case 'rollback': return 'bg-red-100 text-red-700';
            case 'snapshot': return 'bg-purple-100 text-purple-700';
            default: return 'bg-gray-100 text-gray-700';
        }
    }

    getChangeTypeText(changeType) {
        switch (changeType) {
            case 'create': return '创建';
            case 'update': return '更新';
            case 'rollback': return '回滚';
            case 'snapshot': return '快照';
            default: return changeType;
        }
    }

    updatePagination(totalCount) {
        const totalPages = Math.ceil(totalCount / this.pagination.limit);
        
        document.getElementById('pageInfo').textContent = `第 ${this.pagination.page} 页，共 ${totalPages} 页`;
        
        const prevBtn = document.getElementById('prevPage');
        const nextBtn = document.getElementById('nextPage');
        
        prevBtn.disabled = this.pagination.page <= 1;
        nextBtn.disabled = this.pagination.page >= totalPages;
        
        if (totalCount > 0) {
            document.getElementById('timelinePagination').classList.remove('hidden');
        } else {
            document.getElementById('timelinePagination').classList.add('hidden');
        }
    }

    async loadStatistics() {
        try {
            // Mock statistics data
            const mockStats = {
                total_versions: 25,
                current_version: 'v2.1.3',
                last_modified: '2024-01-20 14:30:15',
                last_updated_by: this.currentUser.email,
                config_file_count: 4,
                config_size: 12.3,
                monthly_changes: 15,
                smtp_changes: 5,
                database_changes: 3,
                server_changes: 4,
                storage_changes: 3
            };

            document.getElementById('totalVersions').textContent = mockStats.total_versions;
            document.getElementById('currentVersion').textContent = mockStats.current_version;
            document.getElementById('lastModified').textContent = this.timeAgo(mockStats.last_modified);
            document.getElementById('currentConfigVersion').textContent = mockStats.current_version;
            document.getElementById('lastUpdateTime').textContent = this.formatDateTime(mockStats.last_modified);
            document.getElementById('lastUpdatedBy').textContent = mockStats.last_updated_by;
            document.getElementById('configFileCount').textContent = `${mockStats.config_file_count}个`;
            document.getElementById('configSize').textContent = `${mockStats.config_size} KB`;
            
            // Update change statistics
            document.getElementById('monthlyChanges').textContent = `${mockStats.monthly_changes}次`;
            document.getElementById('smtpChanges').textContent = `${mockStats.smtp_changes}次`;
            document.getElementById('databaseChanges').textContent = `${mockStats.database_changes}次`;
            document.getElementById('serverChanges').textContent = `${mockStats.server_changes}次`;
            document.getElementById('storageChanges').textContent = `${mockStats.storage_changes}次`;
            
            // Update progress bar
            const changePercentage = (mockStats.monthly_changes / 30) * 100; // Assuming 30 is max
            document.getElementById('monthlyChangesBar').style.width = `${Math.min(changePercentage, 100)}%`;

        } catch (error) {
            console.error('Failed to load statistics:', error);
            this.showError('加载统计数据失败');
        }
    }

    // Modal Methods
    showCreateSnapshotModal() {
        this.generateSnapshotName();
        document.getElementById('createSnapshotModal').classList.remove('hidden');
        document.body.style.overflow = 'hidden';
    }

    hideCreateSnapshotModal() {
        document.getElementById('createSnapshotModal').classList.add('hidden');
        document.body.style.overflow = 'auto';
    }

    showRollbackModal(versionId) {
        const version = this.configHistory.find(v => v.id === versionId);
        if (!version) {
            this.showError('版本不存在');
            return;
        }

        document.getElementById('rollbackVersionName').textContent = `${version.title} (${version.version})`;
        document.getElementById('rollbackVersionInfo').textContent = version.description;
        document.getElementById('rollbackVersionDate').textContent = this.formatDateTime(version.timestamp);
        document.getElementById('rollbackVersionAuthor').textContent = version.author;
        
        document.getElementById('rollbackModal').dataset.versionId = versionId;
        document.getElementById('rollbackModal').classList.remove('hidden');
        document.body.style.overflow = 'hidden';
    }

    hideRollbackModal() {
        document.getElementById('rollbackModal').classList.add('hidden');
        document.body.style.overflow = 'auto';
        document.getElementById('rollbackReason').value = '';
    }

    showCompareModal() {
        this.populateCompareVersions();
        document.getElementById('compareModal').classList.remove('hidden');
        document.body.style.overflow = 'hidden';
    }

    hideCompareModal() {
        document.getElementById('compareModal').classList.add('hidden');
        document.body.style.overflow = 'auto';
    }

    populateCompareVersions() {
        const versionA = document.getElementById('compareVersionA');
        const versionB = document.getElementById('compareVersionB');
        
        const options = this.configHistory.map(item => 
            `<option value="${item.id}">${item.version} - ${item.title} (${this.formatDateTime(item.timestamp)})</option>`
        ).join('');
        
        versionA.innerHTML = '<option value="">选择版本A</option>' + options;
        versionB.innerHTML = '<option value="">选择版本B</option>' + options;
    }

    // Action Methods
    async createSnapshot() {
        const name = document.getElementById('snapshotName').value.trim() || this.generateSnapshotName(false);
        const description = document.getElementById('snapshotDescription').value.trim();
        const type = document.getElementById('snapshotType').value;
        const includeSecrets = document.getElementById('includeSecrets').checked;

        try {
            this.showButtonLoading('confirmCreateSnapshot', '创建中...');
            
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 1500));
            
            this.showSuccess('配置快照创建成功');
            this.hideCreateSnapshotModal();
            this.loadConfigHistory();
            this.loadStatistics();
            
        } catch (error) {
            console.error('Failed to create snapshot:', error);
            this.showError('创建配置快照失败');
        } finally {
            this.hideButtonLoading('confirmCreateSnapshot', '创建快照');
        }
    }

    async performRollback() {
        const versionId = document.getElementById('rollbackModal').dataset.versionId;
        const reason = document.getElementById('rollbackReason').value.trim();
        const createBackup = document.getElementById('createBackupBeforeRollback').checked;

        if (!reason) {
            this.showError('请填写回滚原因');
            return;
        }

        try {
            this.showButtonLoading('confirmRollback', '回滚中...');
            
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 2000));
            
            this.showSuccess('配置回滚成功');
            this.hideRollbackModal();
            this.loadConfigHistory();
            this.loadStatistics();
            
        } catch (error) {
            console.error('Failed to rollback:', error);
            this.showError('配置回滚失败');
        } finally {
            this.hideButtonLoading('confirmRollback', '确认回滚');
        }
    }

    async performVersionComparison() {
        const versionAId = document.getElementById('compareVersionA').value;
        const versionBId = document.getElementById('compareVersionB').value;
        
        if (!versionAId || !versionBId) {
            document.getElementById('compareResult').innerHTML = `
                <div class="text-center text-gray-500 py-8">
                    <i class="fas fa-code-branch text-4xl mb-4"></i>
                    <p>选择两个版本进行比较</p>
                </div>
            `;
            return;
        }

        if (versionAId === versionBId) {
            document.getElementById('compareResult').innerHTML = `
                <div class="text-center text-gray-500 py-8">
                    <i class="fas fa-exclamation-triangle text-4xl mb-4"></i>
                    <p>请选择不同的版本进行比较</p>
                </div>
            `;
            return;
        }

        try {
            // Mock comparison result
            const mockDiff = this.generateMockDiff();
            
            document.getElementById('compareResult').innerHTML = `
                <div class="space-y-4">
                    <div class="text-sm text-gray-600 mb-4">
                        <p>共发现 <strong>${mockDiff.length}</strong> 处差异</p>
                    </div>
                    ${mockDiff.map(diff => `
                        <div class="border border-gray-200 rounded-lg overflow-hidden">
                            <div class="bg-gray-50 px-4 py-2 text-sm font-medium text-gray-700">
                                ${diff.section}
                            </div>
                            <div class="p-0">
                                ${diff.changes.map(change => `
                                    <div class="diff-line ${change.type === 'added' ? 'diff-added' : change.type === 'removed' ? 'diff-removed' : 'diff-unchanged'}">
                                        ${change.type === 'added' ? '+ ' : change.type === 'removed' ? '- ' : '  '}${change.content}
                                    </div>
                                `).join('')}
                            </div>
                        </div>
                    `).join('')}
                </div>
            `;
            
        } catch (error) {
            console.error('Failed to compare versions:', error);
            this.showError('版本比较失败');
        }
    }

    generateMockDiff() {
        return [
            {
                section: 'SMTP配置',
                changes: [
                    { type: 'removed', content: 'host: smtp.gmail.com' },
                    { type: 'added', content: 'host: smtp.office365.com' },
                    { type: 'removed', content: 'port: 587' },
                    { type: 'added', content: 'port: 993' },
                    { type: 'unchanged', content: 'use_tls: true' }
                ]
            },
            {
                section: '数据库配置',
                changes: [
                    { type: 'removed', content: 'max_connections: 10' },
                    { type: 'added', content: 'max_connections: 20' },
                    { type: 'unchanged', content: 'connect_timeout: 30' }
                ]
            }
        ];
    }

    async exportHistory() {
        try {
            this.showButtonLoading('exportHistoryBtn', '导出中...');
            
            // Mock export data
            const exportData = {
                exported_at: new Date().toISOString(),
                version: '1.0',
                total_versions: this.configHistory.length,
                history: this.configHistory
            };
            
            const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            
            const a = document.createElement('a');
            a.href = url;
            a.download = `config-history-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.json`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            
            this.showSuccess('配置历史已导出');
            
        } catch (error) {
            console.error('Failed to export history:', error);
            this.showError('导出配置历史失败');
        } finally {
            this.hideButtonLoading('exportHistoryBtn', '导出历史');
        }
    }

    async importHistory() {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.json';
        
        input.onchange = async (e) => {
            const file = e.target.files[0];
            if (!file) return;

            try {
                const text = await file.text();
                const data = JSON.parse(text);
                
                if (!data.history || !Array.isArray(data.history)) {
                    throw new Error('无效的配置历史文件格式');
                }
                
                this.showSuccess(`成功导入 ${data.history.length} 个配置版本`);
                this.loadConfigHistory();
                
            } catch (error) {
                console.error('Failed to import history:', error);
                this.showError('导入配置历史失败：' + error.message);
            }
        };
        
        input.click();
    }

    async rollbackToStable() {
        const stableVersion = this.configHistory.find(v => v.is_stable && !v.is_current);
        
        if (!stableVersion) {
            this.showError('未找到稳定版本');
            return;
        }

        if (confirm(`确定要回滚到稳定版本 ${stableVersion.version} 吗？`)) {
            this.showRollbackModal(stableVersion.id);
            document.getElementById('rollbackReason').value = '回滚到最近的稳定版本';
        }
    }

    async scheduleRollback() {
        this.showInfo('定时回滚功能开发中...');
    }

    async cleanupOldVersions() {
        const oldVersions = this.configHistory.filter(v => !v.is_current && !v.is_stable).length;
        
        if (oldVersions === 0) {
            this.showInfo('没有可清理的旧版本');
            return;
        }

        if (confirm(`确定要清理 ${oldVersions} 个旧版本吗？此操作不可撤销。`)) {
            try {
                // Mock cleanup operation
                await new Promise(resolve => setTimeout(resolve, 1000));
                
                this.showSuccess(`已清理 ${oldVersions} 个旧版本`);
                this.loadConfigHistory();
                this.loadStatistics();
                
            } catch (error) {
                console.error('Failed to cleanup old versions:', error);
                this.showError('清理旧版本失败');
            }
        }
    }

    async downloadCurrentConfig() {
        try {
            // Mock current config data
            const configData = {
                version: 'v2.1.3',
                exported_at: new Date().toISOString(),
                smtp: {
                    enabled: false,
                    host: 'smtp.gmail.com',
                    port: 587,
                    // ... other config
                },
                database: {
                    max_connections: 10,
                    connect_timeout: 30,
                    // ... other config
                },
                // ... other sections
            };
            
            const blob = new Blob([JSON.stringify(configData, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            
            const a = document.createElement('a');
            a.href = url;
            a.download = `current-config-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.json`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            
            this.showSuccess('当前配置已下载');
            
        } catch (error) {
            console.error('Failed to download config:', error);
            this.showError('下载当前配置失败');
        }
    }

    async downloadVersion(versionId) {
        const version = this.configHistory.find(v => v.id === versionId);
        if (!version) {
            this.showError('版本不存在');
            return;
        }

        try {
            // Mock version config data
            const configData = {
                version: version.version,
                timestamp: version.timestamp,
                author: version.author,
                config_type: version.config_type,
                // ... mock config content
            };
            
            const blob = new Blob([JSON.stringify(configData, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            
            const a = document.createElement('a');
            a.href = url;
            a.download = `config-${version.version}-${new Date(version.timestamp).toISOString().slice(0, 19).replace(/:/g, '-')}.json`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            
            this.showSuccess(`版本 ${version.version} 已下载`);
            
        } catch (error) {
            console.error('Failed to download version:', error);
            this.showError('下载版本失败');
        }
    }

    viewVersionDetails(versionId) {
        const version = this.configHistory.find(v => v.id === versionId);
        if (!version) {
            this.showError('版本不存在');
            return;
        }

        // Show version details in a modal or expand inline
        this.showInfo(`查看版本详情: ${version.version} - ${version.title}`);
    }

    async deleteVersion(versionId) {
        const version = this.configHistory.find(v => v.id === versionId);
        if (!version) {
            this.showError('版本不存在');
            return;
        }

        if (!confirm(`确定要删除版本 ${version.version} 吗？此操作不可撤销。`)) {
            return;
        }

        try {
            // Mock delete operation
            await new Promise(resolve => setTimeout(resolve, 500));
            
            this.showSuccess(`版本 ${version.version} 已删除`);
            this.loadConfigHistory();
            this.loadStatistics();
            
        } catch (error) {
            console.error('Failed to delete version:', error);
            this.showError('删除版本失败');
        }
    }

    // Utility Methods
    generateSnapshotName(update = true) {
        const now = new Date();
        const timestamp = now.toISOString().slice(0, 19).replace(/[T:]/g, '_');
        const name = `snapshot_${timestamp}`;
        
        if (update) {
            document.getElementById('snapshotName').value = name;
        }
        
        return name;
    }

    formatDateTime(dateString) {
        const date = new Date(dateString);
        return date.toLocaleString('zh-CN', {
            year: 'numeric',
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
        if (diffInSeconds < 604800) return `${Math.floor(diffInSeconds / 86400)}天前`;
        return `${Math.floor(diffInSeconds / 604800)}周前`;
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

// Initialize config history manager when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.configHistoryManager = new ConfigHistoryManager();
});