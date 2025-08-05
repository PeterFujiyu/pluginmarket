// GeekTools Backup Manager Frontend
class BackupManager {
    constructor() {
        this.config = window.GeekToolsConfig || {};
        this.baseURL = this.config.apiBaseUrl || '/api/v1';
        this.authToken = localStorage.getItem('auth_token');
        this.currentUser = JSON.parse(localStorage.getItem('current_user') || 'null');
        this.backups = [];
        this.schedules = [];
        this.currentOperation = null;
        this.operationPollingInterval = null;
        this.pagination = { page: 1, limit: 10 };
        
        this.init();
    }

    init() {
        this.checkAuth();
        this.bindEvents();
        this.loadBackups();
        this.loadSchedules();
        this.loadStats();
        this.startOperationPolling();
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
        document.getElementById('createBackupBtn').addEventListener('click', () => {
            this.showCreateBackupModal();
        });

        document.getElementById('scheduleBackupBtn').addEventListener('click', () => {
            this.showScheduleBackupModal();
        });

        document.getElementById('importBackupBtn').addEventListener('click', () => {
            this.importBackup();
        });

        document.getElementById('refreshBackupsBtn').addEventListener('click', () => {
            this.loadBackups();
        });

        // Filter
        document.getElementById('backupFilter').addEventListener('change', () => {
            this.loadBackups();
        });

        // Create Backup Modal
        document.getElementById('closeCreateBackupModal').addEventListener('click', () => {
            this.hideCreateBackupModal();
        });

        document.getElementById('cancelCreateBackup').addEventListener('click', () => {
            this.hideCreateBackupModal();
        });

        document.getElementById('createBackupForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.createBackup();
        });

        // Restore Backup Modal
        document.getElementById('closeRestoreBackupModal').addEventListener('click', () => {
            this.hideRestoreBackupModal();
        });

        document.getElementById('cancelRestoreBackup').addEventListener('click', () => {
            this.hideRestoreBackupModal();
        });

        document.getElementById('restoreBackupForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.restoreBackup();
        });

        document.getElementById('confirmRestore').addEventListener('input', (e) => {
            const confirmBtn = document.getElementById('confirmRestoreBackup');
            confirmBtn.disabled = e.target.value.trim().toUpperCase() !== 'RESTORE';
        });

        // Schedule Backup Modal
        document.getElementById('closeScheduleBackupModal').addEventListener('click', () => {
            this.hideScheduleBackupModal();
        });

        document.getElementById('cancelScheduleBackup').addEventListener('click', () => {
            this.hideScheduleBackupModal();
        });

        document.getElementById('scheduleBackupForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.saveSchedule();
        });

        document.getElementById('scheduleFrequency').addEventListener('change', (e) => {
            this.updateScheduleTimeOptions(e.target.value);
        });

        // Current operation cancel
        document.getElementById('cancelOperationBtn').addEventListener('click', () => {
            this.cancelCurrentOperation();
        });

        // Pagination
        document.getElementById('backupPrevPage').addEventListener('click', () => {
            if (this.pagination.page > 1) {
                this.pagination.page--;
                this.loadBackups();
            }
        });

        document.getElementById('backupNextPage').addEventListener('click', () => {
            this.pagination.page++;
            this.loadBackups();
        });

        // Auto-generate backup name
        this.generateBackupName();
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

    async loadStats() {
        try {
            // Mock data - replace with actual API call
            const mockStats = {
                total_backups: 15,
                storage_used: 2.3,
                storage_total: 10,
                last_backup_time: '2024-01-20 14:30:15',
                backup_status: 'success',
                avg_backup_size: 150,
                max_backup_size: 280,
                retention_policy: 30
            };

            document.getElementById('totalBackups').textContent = mockStats.total_backups;
            document.getElementById('storageUsed').textContent = `${mockStats.storage_used} GB`;
            document.getElementById('lastBackupTime').textContent = this.timeAgo(mockStats.last_backup_time);
            
            // Update status indicator
            const statusIndicator = document.getElementById('backupStatusIndicator');
            const statusText = document.getElementById('backupStatusText');
            
            switch (mockStats.backup_status) {
                case 'success':
                    statusIndicator.className = 'w-3 h-3 bg-green-500 rounded-full';
                    statusText.textContent = '正常';
                    break;
                case 'warning':
                    statusIndicator.className = 'w-3 h-3 bg-yellow-500 rounded-full';
                    statusText.textContent = '警告';
                    break;
                case 'error':
                    statusIndicator.className = 'w-3 h-3 bg-red-500 rounded-full';
                    statusText.textContent = '异常';
                    break;
            }

            // Update storage info
            document.getElementById('storageUsedDetails').textContent = `${mockStats.storage_used} GB / ${mockStats.storage_total} GB`;
            document.getElementById('avgBackupSize').textContent = `${mockStats.avg_backup_size} MB`;
            document.getElementById('maxBackupSize').textContent = `${mockStats.max_backup_size} MB`;
            document.getElementById('retentionPolicy').textContent = `${mockStats.retention_policy}天`;
            
            const storagePercentage = (mockStats.storage_used / mockStats.storage_total) * 100;
            document.getElementById('storageProgressBar').style.width = `${storagePercentage}%`;

        } catch (error) {
            console.error('Failed to load stats:', error);
            this.showError('加载统计信息失败');
        }
    }

    async loadBackups() {
        try {
            document.getElementById('backupListLoading').classList.remove('hidden');
            document.getElementById('backupListEmpty').classList.add('hidden');
            
            const filter = document.getElementById('backupFilter').value;
            
            // Mock backup data - replace with actual API call
            const mockBackups = this.generateMockBackups();
            
            this.backups = mockBackups;
            this.renderBackups(mockBackups);
            this.updatePagination(mockBackups.length);
            
        } catch (error) {
            console.error('Failed to load backups:', error);
            this.showError('加载备份列表失败');
            document.getElementById('backupListEmpty').classList.remove('hidden');
        } finally {
            document.getElementById('backupListLoading').classList.add('hidden');
        }
    }

    generateMockBackups() {
        const backups = [];
        const now = new Date();
        
        for (let i = 0; i < 15; i++) {
            const date = new Date(now.getTime() - (i * 24 * 60 * 60 * 1000));
            const types = ['manual', 'scheduled'];
            const statuses = ['success', 'failed', 'running'];
            const type = types[Math.floor(Math.random() * types.length)];
            const status = i === 0 && Math.random() > 0.7 ? 'running' : statuses[Math.floor(Math.random() * statuses.length)];
            
            backups.push({
                id: `backup_${i + 1}`,
                name: type === 'manual' ? `手动备份_${date.toISOString().slice(0, 10)}` : `定时备份_${date.toISOString().slice(0, 10)}`,
                description: type === 'manual' ? '手动创建的数据库备份' : '系统自动创建的定时备份',
                type: type,
                status: status,
                size: Math.floor(Math.random() * 200 + 50) * 1024 * 1024, // 50-250MB
                created_at: date.toISOString(),
                created_by: this.currentUser.email,
                file_path: `/backups/backup_${i + 1}.sql.gz`,
                backup_type: 'full',
                compressed: true,
                progress: status === 'running' ? Math.floor(Math.random() * 80 + 10) : 100
            });
        }
        
        return backups;
    }

    renderBackups(backups) {
        const container = document.getElementById('backupList');
        
        if (backups.length === 0) {
            document.getElementById('backupListEmpty').classList.remove('hidden');
            container.innerHTML = '';
            return;
        }

        container.innerHTML = backups.map(backup => `
            <div class="backup-card gradient-card rounded-xl p-4 border border-white/20">
                <div class="flex items-center justify-between">
                    <div class="flex-1">
                        <div class="flex items-center space-x-3 mb-2">
                            <h4 class="font-semibold text-claude-text">${this.escapeHtml(backup.name)}</h4>
                            <span class="px-2 py-1 text-xs rounded-full ${this.getTypeStyle(backup.type)}">
                                ${backup.type === 'manual' ? '手动' : '定时'}
                            </span>
                            <span class="px-2 py-1 text-xs rounded-full ${this.getStatusStyle(backup.status)}">
                                ${this.getStatusText(backup.status)}
                            </span>
                        </div>
                        <p class="text-sm text-gray-600 mb-2">${this.escapeHtml(backup.description || '无描述')}</p>
                        <div class="flex items-center space-x-4 text-xs text-gray-500">
                            <span><i class="fas fa-calendar mr-1"></i>${this.formatDateTime(backup.created_at)}</span>
                            <span><i class="fas fa-user mr-1"></i>${this.escapeHtml(backup.created_by)}</span>
                            <span><i class="fas fa-hdd mr-1"></i>${this.formatFileSize(backup.size)}</span>
                            <span><i class="fas fa-database mr-1"></i>${backup.backup_type}</span>
                            ${backup.compressed ? '<span><i class="fas fa-compress-alt mr-1"></i>已压缩</span>' : ''}
                        </div>
                        ${backup.status === 'running' ? `
                            <div class="mt-3">
                                <div class="flex justify-between text-xs text-gray-500 mb-1">
                                    <span>备份进度</span>
                                    <span>${backup.progress}%</span>
                                </div>
                                <div class="w-full bg-gray-200 rounded-full h-2">
                                    <div class="progress-bar h-2 rounded-full" style="width: ${backup.progress}%"></div>
                                </div>
                            </div>
                        ` : ''}
                    </div>
                    <div class="flex items-center space-x-2 ml-4">
                        ${backup.status === 'success' ? `
                            <button onclick="backupManager.downloadBackup('${backup.id}')" 
                                    class="px-3 py-2 bg-blue-500 text-white text-sm rounded-lg hover:bg-blue-600 transition-colors"
                                    title="下载备份">
                                <i class="fas fa-download"></i>
                            </button>
                            <button onclick="backupManager.showRestoreBackupModal('${backup.id}')" 
                                    class="px-3 py-2 bg-green-500 text-white text-sm rounded-lg hover:bg-green-600 transition-colors"
                                    title="恢复备份">
                                <i class="fas fa-upload"></i>
                            </button>
                        ` : backup.status === 'running' ? `
                            <button onclick="backupManager.cancelBackup('${backup.id}')" 
                                    class="px-3 py-2 bg-red-500 text-white text-sm rounded-lg hover:bg-red-600 transition-colors"
                                    title="取消备份">
                                <i class="fas fa-stop"></i>
                            </button>
                        ` : `
                            <button class="px-3 py-2 bg-gray-300 text-gray-500 text-sm rounded-lg cursor-not-allowed" 
                                    title="备份失败" disabled>
                                <i class="fas fa-exclamation-triangle"></i>
                            </button>
                        `}
                        <button onclick="backupManager.deleteBackup('${backup.id}')" 
                                class="px-3 py-2 bg-red-500 text-white text-sm rounded-lg hover:bg-red-600 transition-colors"
                                title="删除备份">
                            <i class="fas fa-trash"></i>
                        </button>
                    </div>
                </div>
            </div>
        `).join('');
    }

    async loadSchedules() {
        try {
            // Mock schedule data
            const mockSchedules = [
                {
                    id: 'schedule_1',
                    name: '每日备份',
                    frequency: 'daily',
                    time: '02:00',
                    enabled: true,
                    next_run: '2024-01-21 02:00:00',
                    last_run: '2024-01-20 02:00:00',
                    last_status: 'success'
                },
                {
                    id: 'schedule_2',
                    name: '周末备份',
                    frequency: 'weekly',
                    time: '03:00',
                    day: 0, // Sunday
                    enabled: false,
                    next_run: '2024-01-21 03:00:00',
                    last_run: '2024-01-14 03:00:00',
                    last_status: 'success'
                }
            ];

            this.schedules = mockSchedules;
            this.renderSchedules(mockSchedules);

        } catch (error) {
            console.error('Failed to load schedules:', error);
            this.showError('加载定时任务失败');
        }
    }

    renderSchedules(schedules) {
        const container = document.getElementById('scheduleList');
        
        if (schedules.length === 0) {
            container.innerHTML = `
                <div class="text-center py-4 text-gray-500 text-sm">
                    <i class="fas fa-calendar-plus text-2xl mb-2"></i>
                    <p>暂无定时备份任务</p>
                </div>
            `;
            return;
        }

        container.innerHTML = schedules.map(schedule => `
            <div class="schedule-item bg-gray-50 rounded-lg p-3">
                <div class="flex items-center justify-between mb-2">
                    <h5 class="font-medium text-sm">${this.escapeHtml(schedule.name)}</h5>
                    <div class="flex items-center space-x-2">
                        <label class="relative inline-flex items-center cursor-pointer">
                            <input type="checkbox" ${schedule.enabled ? 'checked' : ''} 
                                   onchange="backupManager.toggleSchedule('${schedule.id}')"
                                   class="sr-only peer">
                            <div class="w-6 h-3 bg-gray-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[1px] after:left-[1px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-2.5 after:w-2.5 after:transition-all peer-checked:bg-claude-orange"></div>
                        </label>
                        <button onclick="backupManager.editSchedule('${schedule.id}')" 
                                class="text-gray-400 hover:text-gray-600 text-xs">
                            <i class="fas fa-edit"></i>
                        </button>
                        <button onclick="backupManager.deleteSchedule('${schedule.id}')" 
                                class="text-gray-400 hover:text-red-600 text-xs">
                            <i class="fas fa-trash"></i>
                        </button>
                    </div>
                </div>
                <div class="text-xs text-gray-500 space-y-1">
                    <div>频率: ${this.getFrequencyText(schedule.frequency, schedule.day)}</div>
                    <div>时间: ${schedule.time}</div>
                    <div>下次运行: ${this.formatDateTime(schedule.next_run)}</div>
                    <div class="flex items-center">
                        <span>最后运行: ${this.formatDateTime(schedule.last_run)}</span>
                        <span class="ml-2 px-1 py-0.5 text-xs rounded ${schedule.last_status === 'success' ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'}">
                            ${schedule.last_status === 'success' ? '成功' : '失败'}
                        </span>
                    </div>
                </div>
            </div>
        `).join('');
    }

    getFrequencyText(frequency, day = null) {
        switch (frequency) {
            case 'daily': return '每日';
            case 'weekly': 
                const days = ['日', '一', '二', '三', '四', '五', '六'];
                return `每周${days[day]}`;
            case 'monthly': return '每月';
            default: return frequency;
        }
    }

    updatePagination(totalCount) {
        const totalPages = Math.ceil(totalCount / this.pagination.limit);
        const startItem = (this.pagination.page - 1) * this.pagination.limit + 1;
        const endItem = Math.min(this.pagination.page * this.pagination.limit, totalCount);

        document.getElementById('backupPageStart').textContent = startItem;
        document.getElementById('backupPageEnd').textContent = endItem;
        document.getElementById('backupTotalCount').textContent = totalCount;
        document.getElementById('backupCurrentPage').textContent = this.pagination.page;

        const prevBtn = document.getElementById('backupPrevPage');
        const nextBtn = document.getElementById('backupNextPage');

        prevBtn.disabled = this.pagination.page <= 1;
        nextBtn.disabled = this.pagination.page >= totalPages;

        if (totalCount > 0) {
            document.getElementById('backupPagination').classList.remove('hidden');
        } else {
            document.getElementById('backupPagination').classList.add('hidden');
        }
    }

    // Modal Methods
    showCreateBackupModal() {
        this.generateBackupName();
        document.getElementById('createBackupModal').classList.remove('hidden');
        document.body.style.overflow = 'hidden';
    }

    hideCreateBackupModal() {
        document.getElementById('createBackupModal').classList.add('hidden');
        document.body.style.overflow = 'auto';
    }

    showRestoreBackupModal(backupId) {
        const backup = this.backups.find(b => b.id === backupId);
        if (!backup) {
            this.showError('备份不存在');
            return;
        }

        document.getElementById('selectedBackupName').textContent = backup.name;
        document.getElementById('selectedBackupInfo').textContent = backup.description || '无描述';
        document.getElementById('selectedBackupSize').textContent = this.formatFileSize(backup.size);
        document.getElementById('selectedBackupDate').textContent = this.formatDateTime(backup.created_at);
        
        document.getElementById('restoreBackupModal').dataset.backupId = backupId;
        document.getElementById('restoreBackupModal').classList.remove('hidden');
        document.body.style.overflow = 'hidden';
    }

    hideRestoreBackupModal() {
        document.getElementById('restoreBackupModal').classList.add('hidden');
        document.body.style.overflow = 'auto';
        document.getElementById('confirmRestore').value = '';
        document.getElementById('confirmRestoreBackup').disabled = true;
    }

    showScheduleBackupModal() {
        document.getElementById('scheduleBackupModal').classList.remove('hidden');
        document.body.style.overflow = 'hidden';
        this.updateScheduleTimeOptions('daily');
    }

    hideScheduleBackupModal() {
        document.getElementById('scheduleBackupModal').classList.add('hidden');
        document.body.style.overflow = 'auto';
    }

    updateScheduleTimeOptions(frequency) {
        const weeklyDiv = document.getElementById('weeklyDaySelection');
        const monthlyDiv = document.getElementById('monthlyDateSelection');
        
        weeklyDiv.classList.add('hidden');
        monthlyDiv.classList.add('hidden');
        
        switch (frequency) {
            case 'weekly':
                weeklyDiv.classList.remove('hidden');
                break;
            case 'monthly':
                monthlyDiv.classList.remove('hidden');
                break;
        }
    }

    // Backup Operations
    async createBackup() {
        const name = document.getElementById('backupName').value.trim() || this.generateBackupName(false);
        const description = document.getElementById('backupDescription').value.trim();
        const type = document.getElementById('backupType').value;
        const compress = document.getElementById('compressBackup').checked;

        try {
            this.showButtonLoading('confirmCreateBackup', '创建中...');
            
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 1000));
            
            this.showSuccess('备份任务已启动');
            this.hideCreateBackupModal();
            this.loadBackups();
            
            // Start showing current operation
            this.showCurrentOperation('创建数据库备份', name);
            
        } catch (error) {
            console.error('Failed to create backup:', error);
            this.showError('创建备份失败');
        } finally {
            this.hideButtonLoading('confirmCreateBackup', '开始备份');
        }
    }

    async restoreBackup() {
        const backupId = document.getElementById('restoreBackupModal').dataset.backupId;
        const confirmText = document.getElementById('confirmRestore').value.trim();
        
        if (confirmText.toUpperCase() !== 'RESTORE') {
            this.showError('请输入 "RESTORE" 确认操作');
            return;
        }

        const dropTables = document.getElementById('dropExistingTables').checked;
        const createBackup = document.getElementById('createPreRestoreBackup').checked;

        try {
            this.showButtonLoading('confirmRestoreBackup', '恢复中...');
            
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 2000));
            
            this.showSuccess('数据库恢复成功');
            this.hideRestoreBackupModal();
            
        } catch (error) {
            console.error('Failed to restore backup:', error);
            this.showError('恢复备份失败');
        } finally {
            this.hideButtonLoading('confirmRestoreBackup', '开始恢复');
        }
    }

    async downloadBackup(backupId) {
        const backup = this.backups.find(b => b.id === backupId);
        if (!backup) {
            this.showError('备份不存在');
            return;
        }

        try {
            // Mock download - in production this would be a real file download
            this.showSuccess(`正在下载备份: ${backup.name}`);
            
            // Simulate download
            const blob = new Blob(['Mock backup data'], { type: 'application/gzip' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `${backup.name}.sql.gz`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            
        } catch (error) {
            console.error('Failed to download backup:', error);
            this.showError('下载备份失败');
        }
    }

    async deleteBackup(backupId) {
        const backup = this.backups.find(b => b.id === backupId);
        if (!backup) {
            this.showError('备份不存在');
            return;
        }

        if (!confirm(`确定要删除备份 "${backup.name}" 吗？此操作不可撤销。`)) {
            return;
        }

        try {
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 500));
            
            this.showSuccess('备份已删除');
            this.loadBackups();
            
        } catch (error) {
            console.error('Failed to delete backup:', error);
            this.showError('删除备份失败');
        }
    }

    async cancelBackup(backupId) {
        if (!confirm('确定要取消正在进行的备份吗？')) {
            return;
        }

        try {
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 500));
            
            this.showSuccess('备份已取消');
            this.loadBackups();
            
        } catch (error) {
            console.error('Failed to cancel backup:', error);
            this.showError('取消备份失败');
        }
    }

    // Schedule Operations
    async saveSchedule() {
        const frequency = document.getElementById('scheduleFrequency').value;
        const time = document.getElementById('scheduleTimeInput').value;
        const retention = parseInt(document.getElementById('scheduleRetention').value);
        const enabled = document.getElementById('scheduleEnabled').checked;
        
        let scheduleData = {
            frequency,
            time,
            retention,
            enabled
        };

        if (frequency === 'weekly') {
            scheduleData.day = parseInt(document.getElementById('weeklyDay').value);
        } else if (frequency === 'monthly') {
            scheduleData.date = parseInt(document.getElementById('monthlyDate').value);
        }

        try {
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 500));
            
            this.showSuccess('定时备份设置已保存');
            this.hideScheduleBackupModal();
            this.loadSchedules();
            
        } catch (error) {
            console.error('Failed to save schedule:', error);
            this.showError('保存定时备份设置失败');
        }
    }

    async toggleSchedule(scheduleId) {
        try {
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 200));
            
            const schedule = this.schedules.find(s => s.id === scheduleId);
            if (schedule) {
                schedule.enabled = !schedule.enabled;
                this.showSuccess(`定时备份已${schedule.enabled ? '启用' : '禁用'}`);
            }
            
        } catch (error) {
            console.error('Failed to toggle schedule:', error);
            this.showError('切换定时备份状态失败');
        }
    }

    async deleteSchedule(scheduleId) {
        if (!confirm('确定要删除此定时备份任务吗？')) {
            return;
        }

        try {
            // Mock API call
            await new Promise(resolve => setTimeout(resolve, 500));
            
            this.showSuccess('定时备份任务已删除');
            this.loadSchedules();
            
        } catch (error) {
            console.error('Failed to delete schedule:', error);
            this.showError('删除定时备份任务失败');
        }
    }

    editSchedule(scheduleId) {
        // Load schedule data into modal and show it
        const schedule = this.schedules.find(s => s.id === scheduleId);
        if (schedule) {
            document.getElementById('scheduleFrequency').value = schedule.frequency;
            document.getElementById('scheduleTimeInput').value = schedule.time;
            document.getElementById('scheduleRetention').value = schedule.retention || 7;
            document.getElementById('scheduleEnabled').checked = schedule.enabled;
            
            this.updateScheduleTimeOptions(schedule.frequency);
            
            if (schedule.frequency === 'weekly' && schedule.day !== undefined) {
                document.getElementById('weeklyDay').value = schedule.day;
            }
            if (schedule.frequency === 'monthly' && schedule.date !== undefined) {
                document.getElementById('monthlyDate').value = schedule.date;
            }
            
            this.showScheduleBackupModal();
        }
    }

    // Current Operation Management
    showCurrentOperation(description, name) {
        this.currentOperation = {
            description,
            name,
            startTime: new Date(),
            progress: 0
        };

        document.getElementById('operationDescription').textContent = `${description}: ${name}`;
        document.getElementById('operationStartTime').textContent = `开始时间: ${this.formatDateTime(this.currentOperation.startTime)}`;
        document.getElementById('currentOperation').classList.remove('hidden');
        
        // Start progress simulation
        this.simulateProgress();
    }

    simulateProgress() {
        if (!this.currentOperation) return;

        const interval = setInterval(() => {
            if (!this.currentOperation) {
                clearInterval(interval);
                return;
            }

            this.currentOperation.progress += Math.random() * 10;
            
            if (this.currentOperation.progress >= 100) {
                this.currentOperation.progress = 100;
                this.completeCurrentOperation();
                clearInterval(interval);
            }

            this.updateOperationProgress();
        }, 1000);
    }

    updateOperationProgress() {
        if (!this.currentOperation) return;

        const progress = Math.min(this.currentOperation.progress, 100);
        document.getElementById('operationProgress').textContent = `${Math.round(progress)}%`;
        document.getElementById('operationProgressBar').style.width = `${progress}%`;
        
        const elapsed = (new Date() - this.currentOperation.startTime) / 1000;
        const estimated = progress > 0 ? Math.round((elapsed * (100 - progress)) / progress) : 0;
        
        if (estimated > 0) {
            document.getElementById('operationEstimated').textContent = `预计剩余: ${this.formatDuration(estimated)}`;
        } else {
            document.getElementById('operationEstimated').textContent = '即将完成';
        }
    }

    completeCurrentOperation() {
        this.currentOperation = null;
        document.getElementById('currentOperation').classList.add('hidden');
        this.loadBackups();
        this.loadStats();
    }

    cancelCurrentOperation() {
        if (this.currentOperation && confirm('确定要取消当前操作吗？')) {
            this.currentOperation = null;
            document.getElementById('currentOperation').classList.add('hidden');
            this.showSuccess('操作已取消');
        }
    }

    startOperationPolling() {
        // In a real application, this would poll the server for operation status
        this.operationPollingInterval = setInterval(() => {
            // Check for any running operations
            // This is a mock implementation
        }, 5000);
    }

    async importBackup() {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.sql,.sql.gz,.dump';
        
        input.onchange = async (e) => {
            const file = e.target.files[0];
            if (!file) return;

            try {
                this.showSuccess(`正在导入备份文件: ${file.name}`);
                
                // Mock upload process
                await new Promise(resolve => setTimeout(resolve, 2000));
                
                this.showSuccess('备份文件导入成功');
                this.loadBackups();
                
            } catch (error) {
                console.error('Failed to import backup:', error);
                this.showError('导入备份文件失败');
            }
        };
        
        input.click();
    }

    // Utility Methods
    generateBackupName(update = true) {
        const now = new Date();
        const timestamp = now.toISOString().slice(0, 19).replace(/[T:]/g, '_');
        const name = `manual_backup_${timestamp}`;
        
        if (update) {
            document.getElementById('backupName').value = name;
        }
        
        return name;
    }

    getTypeStyle(type) {
        return type === 'manual' 
            ? 'bg-blue-100 text-blue-700' 
            : 'bg-purple-100 text-purple-700';
    }

    getStatusStyle(status) {
        switch (status) {
            case 'success': return 'bg-green-100 text-green-700';
            case 'failed': return 'bg-red-100 text-red-700';
            case 'running': return 'bg-yellow-100 text-yellow-700';
            default: return 'bg-gray-100 text-gray-700';
        }
    }

    getStatusText(status) {
        switch (status) {
            case 'success': return '成功';
            case 'failed': return '失败';
            case 'running': return '进行中';
            default: return '未知';
        }
    }

    formatFileSize(bytes) {
        const sizes = ['B', 'KB', 'MB', 'GB'];
        if (bytes === 0) return '0 B';
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
    }

    formatDuration(seconds) {
        if (seconds < 60) return `${seconds}秒`;
        if (seconds < 3600) return `${Math.floor(seconds / 60)}分钟`;
        return `${Math.floor(seconds / 3600)}小时${Math.floor((seconds % 3600) / 60)}分钟`;
    }

    formatDateTime(dateString) {
        if (!dateString) return '-';
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
        if (!dateString) return '-';
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

// Initialize backup manager when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.backupManager = new BackupManager();
});