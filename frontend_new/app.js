// GeekTools Plugin Marketplace Frontend - Minimalist Black-White-Gray Design
class PluginMarketplace {
    constructor() {
        // 从配置文件读取设置
        const config = window.GeekToolsConfig || {};
        this.baseURL = config.apiBaseUrl || '/api/v1';
        this.currentPage = 1;
        this.pageSize = config.frontend?.pageSize || 20;
        this.currentQuery = '';
        this.currentCategory = '';
        this.currentSort = 'downloads';
        this.plugins = [];
        this.totalPages = 0;
        this.authToken = localStorage.getItem('auth_token');
        this.currentUser = JSON.parse(localStorage.getItem('current_user') || 'null');
        
        this.init();
    }

    init() {
        this.bindEvents();
        this.initAuth();
        this.initTheme();
        this.loadStats();
        this.loadPlugins();
        this.setupFileUpload();
        this.initAnimations();
    }

    initAnimations() {
        // 简化的动画系统 - 移除复杂特效，专注于简洁过渡
        this.setupFadeInAnimation();
        this.setupNumberAnimations();
    }

    setupFadeInAnimation() {
        // 页面元素渐入动画
        const observer = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    entry.target.classList.add('fade-in');
                }
            });
        }, { threshold: 0.1 });

        // 观察需要动画的元素
        document.querySelectorAll('.card-minimal, .stats-card').forEach(el => {
            observer.observe(el);
        });
    }

    setupNumberAnimations() {
        // 数字计数动画
        this.animateNumbers = (element, start, end, duration = 1500) => {
            const startTime = performance.now();
            const range = end - start;
            
            const updateNumber = (currentTime) => {
                const elapsed = currentTime - startTime;
                const progress = Math.min(elapsed / duration, 1);
                
                // 使用简单的线性插值，符合极简美学
                const currentValue = Math.floor(start + (range * progress));
                element.textContent = currentValue.toLocaleString();
                
                if (progress < 1) {
                    requestAnimationFrame(updateNumber);
                } else {
                    element.classList.add('animate');
                }
            };
            
            requestAnimationFrame(updateNumber);
        };
    }

    initTheme() {
        // 极简主题切换系统
        const darkModeToggle = document.getElementById('darkModeToggle');
        const currentTheme = localStorage.getItem('theme') || 'light';
        
        // 应用主题
        this.applyTheme(currentTheme);
        
        // 绑定切换事件
        darkModeToggle?.addEventListener('click', () => {
            const newTheme = document.documentElement.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
            this.applyTheme(newTheme);
            localStorage.setItem('theme', newTheme);
        });
    }

    applyTheme(theme) {
        document.documentElement.setAttribute('data-theme', theme);
        
        // 更新切换按钮图标
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
        // 搜索功能
        const searchInput = document.getElementById('searchInput');
        let searchTimeout;
        
        searchInput?.addEventListener('input', (e) => {
            clearTimeout(searchTimeout);
            searchTimeout = setTimeout(() => {
                this.currentQuery = e.target.value.trim();
                this.currentPage = 1;
                this.loadPlugins();
            }, 300);
        });

        // 筛选器
        const categoryFilter = document.getElementById('categoryFilter');
        const sortFilter = document.getElementById('sortFilter');
        
        categoryFilter?.addEventListener('change', (e) => {
            this.currentCategory = e.target.value;
            this.currentPage = 1;
            this.loadPlugins();
        });
        
        sortFilter?.addEventListener('change', (e) => {
            this.currentSort = e.target.value;
            this.currentPage = 1;
            this.loadPlugins();
        });

        // 登录相关
        document.getElementById('loginBtn')?.addEventListener('click', () => {
            this.showLoginModal();
        });
        
        document.getElementById('logoutBtn')?.addEventListener('click', () => {
            this.logout();
        });

        // 上传功能
        document.getElementById('uploadBtn')?.addEventListener('click', () => {
            if (this.authToken) {
                this.showUploadModal();
            }
        });
        
        document.getElementById('uploadBtnLoggedIn')?.addEventListener('click', () => {
            this.showUploadModal();
        });

        // 模态框关闭
        this.bindModalCloseEvents();
        
        // 登录表单
        this.bindLoginFormEvents();
        
        // 上传表单
        this.bindUploadFormEvents();
        
        // 评分表单
        this.bindRatingFormEvents();
    }

    bindModalCloseEvents() {
        // 统一的模态框关闭处理
        const modals = ['pluginModal', 'uploadModal', 'loginModal', 'ratingModal'];
        
        modals.forEach(modalId => {
            const modal = document.getElementById(modalId);
            const closeBtn = document.getElementById(`close${modalId.replace('Modal', '')}Modal`);
            
            // 背景点击关闭
            modal?.addEventListener('click', (e) => {
                if (e.target === modal) {
                    this.hideModal(modalId);
                }
            });
            
            // 关闭按钮
            closeBtn?.addEventListener('click', () => {
                this.hideModal(modalId);
            });
            
            // ESC键关闭
            document.addEventListener('keydown', (e) => {
                if (e.key === 'Escape' && !modal?.classList.contains('hidden')) {
                    this.hideModal(modalId);
                }
            });
        });
    }

    bindLoginFormEvents() {
        const sendCodeBtn = document.getElementById('sendCodeBtn');
        const verifyCodeBtn = document.getElementById('verifyCodeBtn');
        const backToEmailBtn = document.getElementById('backToEmailBtn');
        const loginForm = document.getElementById('loginForm');

        sendCodeBtn?.addEventListener('click', () => {
            this.sendVerificationCode();
        });

        backToEmailBtn?.addEventListener('click', () => {
            this.showSendCodeStep();
        });

        loginForm?.addEventListener('submit', (e) => {
            e.preventDefault();
            this.verifyCode();
        });
    }

    bindUploadFormEvents() {
        const uploadForm = document.getElementById('uploadForm');
        const pluginFile = document.getElementById('pluginFile');
        const dropArea = document.getElementById('dropArea');
        const cancelUpload = document.getElementById('cancelUpload');

        // 文件选择
        dropArea?.addEventListener('click', () => {
            pluginFile?.click();
        });

        pluginFile?.addEventListener('change', (e) => {
            this.handleFileSelect(e.target.files[0]);
        });

        // 拖拽上传
        dropArea?.addEventListener('dragover', (e) => {
            e.preventDefault();
            e.stopPropagation();
            dropArea.style.borderColor = 'var(--color-black)';
        });

        dropArea?.addEventListener('dragleave', (e) => {
            e.preventDefault();
            e.stopPropagation();
            dropArea.style.borderColor = 'var(--color-gray-300)';
        });

        dropArea?.addEventListener('drop', (e) => {
            e.preventDefault();
            e.stopPropagation();
            dropArea.style.borderColor = 'var(--color-gray-300)';
            
            const files = e.dataTransfer.files;
            if (files.length > 0) {
                this.handleFileSelect(files[0]);
            }
        });

        uploadForm?.addEventListener('submit', (e) => {
            e.preventDefault();
            this.uploadPlugin();
        });

        cancelUpload?.addEventListener('click', () => {
            this.hideModal('uploadModal');
        });
    }

    bindRatingFormEvents() {
        const ratingStars = document.querySelectorAll('.rating-star');
        const ratingForm = document.getElementById('ratingForm');
        const cancelRating = document.getElementById('cancelRating');

        ratingStars.forEach(star => {
            star.addEventListener('click', (e) => {
                e.preventDefault();
                const rating = parseInt(star.dataset.rating);
                this.setRating(rating);
            });

            star.addEventListener('mouseenter', () => {
                this.highlightStars(parseInt(star.dataset.rating));
            });
        });

        ratingForm?.addEventListener('submit', (e) => {
            e.preventDefault();
            this.submitRating();
        });

        cancelRating?.addEventListener('click', () => {
            this.hideModal('ratingModal');
        });
    }

    initAuth() {
        if (this.authToken && this.currentUser) {
            this.showLoggedInState();
        } else {
            this.showLoggedOutState();
        }
    }

    showLoggedInState() {
        document.getElementById('loggedOut')?.classList.add('hidden');
        document.getElementById('loggedIn')?.classList.remove('hidden');
        
        const userEmail = document.getElementById('userEmail');
        const userRole = document.getElementById('userRole');
        const adminPanelLink = document.getElementById('adminPanelLink');
        
        if (userEmail) userEmail.textContent = this.currentUser.email;
        
        if (this.currentUser.role === 'admin') {
            userRole?.classList.remove('hidden');
            adminPanelLink?.classList.remove('hidden');
        }
    }

    showLoggedOutState() {
        document.getElementById('loggedOut')?.classList.remove('hidden');
        document.getElementById('loggedIn')?.classList.add('hidden');
    }

    async loadStats() {
        try {
            const response = await fetch(`${this.baseURL}/health`);
            if (response.ok) {
                // 模拟统计数据，实际项目中应该有专门的统计接口
                this.updateStats({
                    totalPlugins: Math.floor(Math.random() * 1000) + 100,
                    activeDevs: Math.floor(Math.random() * 50) + 10,
                    totalDownloads: Math.floor(Math.random() * 10000) + 1000,
                    weeklyNew: Math.floor(Math.random() * 20) + 5
                });
            }
        } catch (error) {
            console.error('Failed to load stats:', error);
            // 显示默认数据
            this.updateStats({
                totalPlugins: '--',
                activeDevs: '--',
                totalDownloads: '--',
                weeklyNew: '--'
            });
        }
    }

    updateStats(stats) {
        const elements = {
            totalPlugins: document.getElementById('totalPlugins'),
            activeDevs: document.getElementById('activeDevs'),
            totalDownloads: document.getElementById('totalDownloads'),
            weeklyNew: document.getElementById('weeklyNew')
        };

        Object.entries(stats).forEach(([key, value]) => {
            const element = elements[key];
            if (element) {
                if (typeof value === 'number') {
                    this.animateNumbers(element, 0, value);
                } else {
                    element.textContent = value;
                }
            }
        });
    }

    async loadPlugins() {
        this.showLoadingState();
        
        try {
            const params = new URLSearchParams({
                page: this.currentPage,
                limit: this.pageSize,
                sort: this.currentSort
            });
            
            if (this.currentQuery) params.append('search', this.currentQuery);
            if (this.currentCategory) params.append('category', this.currentCategory);
            
            const response = await fetch(`${this.baseURL}/plugins?${params}`);
            
            if (!response.ok) {
                throw new Error('Failed to load plugins');
            }
            
            const data = await response.json();
            this.plugins = data.plugins || [];
            this.totalPages = Math.ceil((data.total || 0) / this.pageSize);
            
            this.renderPlugins();
            this.updatePluginCount(data.total || 0);
            this.renderPagination();
            
        } catch (error) {
            console.error('Failed to load plugins:', error);
            this.showErrorState();
        } finally {
            this.hideLoadingState();
        }
    }

    renderPlugins() {
        const pluginGrid = document.getElementById('pluginGrid');
        
        if (!pluginGrid) return;
        
        if (this.plugins.length === 0) {
            this.showEmptyState();
            return;
        }
        
        this.hideEmptyState();
        
        pluginGrid.innerHTML = this.plugins.map(plugin => this.createPluginCard(plugin)).join('');
        
        // 绑定插件卡片点击事件
        this.bindPluginCardEvents();
    }

    createPluginCard(plugin) {
        const stars = this.renderStars(plugin.rating || 0);
        const downloadCount = this.formatNumber(plugin.downloads || 0);
        
        return `
            <div class="plugin-card magnetic-effect" data-plugin-id="${plugin.id}">
                <div class="mb-4">
                    <div class="flex justify-between items-start mb-2">
                        <h3 class="text-subtitle font-mono">${this.escapeHtml(plugin.name)}</h3>
                        <span class="text-micro">${this.escapeHtml(plugin.version || 'v1.0.0')}</span>
                    </div>
                    <p class="text-caption font-mono">${this.escapeHtml(plugin.author)}</p>
                </div>
                
                <div class="mb-4">
                    <p class="text-body line-clamp-3">${this.escapeHtml(plugin.description || '')}</p>
                </div>
                
                <div class="mb-4">
                    <div class="flex flex-wrap gap-2">
                        ${plugin.category ? `<span class="text-micro px-2 py-1 bg-gray-100 border">${this.escapeHtml(plugin.category)}</span>` : ''}
                    </div>
                </div>
                
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                        <div class="flex items-center space-x-1">
                            ${stars}
                            <span class="text-micro">${plugin.rating || 0}</span>
                        </div>
                        <div class="flex items-center space-x-1">
                            <i class="fas fa-download text-micro"></i>
                            <span class="text-micro">${downloadCount}</span>
                        </div>
                    </div>
                    <button class="btn-secondary text-micro" onclick="marketplace.downloadPlugin(${plugin.id})">
                        下载
                    </button>
                </div>
            </div>
        `;
    }

    bindPluginCardEvents() {
        const pluginCards = document.querySelectorAll('.plugin-card');
        
        pluginCards.forEach(card => {
            card.addEventListener('click', (e) => {
                // 避免按钮点击事件冒泡
                if (e.target.tagName === 'BUTTON' || e.target.closest('button')) {
                    return;
                }
                
                const pluginId = card.dataset.pluginId;
                this.showPluginDetail(pluginId);
            });
        });
    }

    async showPluginDetail(pluginId) {
        try {
            const response = await fetch(`${this.baseURL}/plugins/${pluginId}`);
            
            if (!response.ok) {
                throw new Error('Failed to load plugin details');
            }
            
            const plugin = await response.json();
            this.renderPluginModal(plugin);
            this.showModal('pluginModal');
            
        } catch (error) {
            console.error('Failed to load plugin details:', error);
            this.showAlert('加载插件详情失败', 'error');
        }
    }

    renderPluginModal(plugin) {
        const modalContent = document.getElementById('modalContent');
        if (!modalContent) return;
        
        const stars = this.renderStars(plugin.rating || 0);
        const downloadCount = this.formatNumber(plugin.downloads || 0);
        
        modalContent.innerHTML = `
            <div class="p-6">
                <div class="flex items-center justify-between mb-6">
                    <div>
                        <h2 class="text-title font-mono mb-2">${this.escapeHtml(plugin.name)}</h2>
                        <p class="text-caption font-mono">${this.escapeHtml(plugin.author)} • ${this.escapeHtml(plugin.version || 'v1.0.0')}</p>
                    </div>
                    <button onclick="marketplace.hideModal('pluginModal')" class="btn-ghost">
                        <i class="fas fa-times text-xl"></i>
                    </button>
                </div>
                
                <div class="mb-6">
                    <p class="text-body">${this.escapeHtml(plugin.description || '')}</p>
                </div>
                
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
                    <div class="text-center">
                        <div class="text-title">${downloadCount}</div>
                        <div class="text-micro">下载次数</div>
                    </div>
                    <div class="text-center">
                        <div class="text-title flex items-center justify-center space-x-1">
                            ${stars}
                            <span>${plugin.rating || 0}</span>
                        </div>
                        <div class="text-micro">用户评分</div>
                    </div>
                    <div class="text-center">
                        <div class="text-title">${this.formatDate(plugin.updated_at)}</div>
                        <div class="text-micro">最后更新</div>
                    </div>
                </div>
                
                <div class="flex space-x-3">
                    <button onclick="marketplace.downloadPlugin(${plugin.id})" class="btn-primary flex-1">
                        <i class="fas fa-download mr-2"></i>下载插件
                    </button>
                    ${this.authToken ? `
                        <button onclick="marketplace.showRatingModal(${plugin.id})" class="btn-secondary">
                            <i class="fas fa-star mr-2"></i>评分
                        </button>
                    ` : ''}
                </div>
            </div>
        `;
    }

    async downloadPlugin(pluginId) {
        try {
            const response = await fetch(`${this.baseURL}/plugins/${pluginId}/download`, {
                method: 'GET'
            });
            
            if (!response.ok) {
                throw new Error('Download failed');
            }
            
            // 获取文件名
            const contentDisposition = response.headers.get('content-disposition');
            const filename = contentDisposition 
                ? contentDisposition.split('filename=')[1].replace(/"/g, '')
                : `plugin-${pluginId}.tar.gz`;
            
            const blob = await response.blob();
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            document.body.removeChild(a);
            
            this.showAlert('下载成功', 'success');
            
        } catch (error) {
            console.error('Download failed:', error);
            this.showAlert('下载失败', 'error');
        }
    }

    // Modal相关方法
    showModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.classList.remove('hidden');
            modal.classList.add('fade-in');
        }
    }

    hideModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.classList.add('hidden');
            modal.classList.remove('fade-in');
        }
    }

    // 状态显示方法
    showLoadingState() {
        document.getElementById('loadingState')?.classList.remove('hidden');
        document.getElementById('pluginGrid')?.classList.add('hidden');
        document.getElementById('emptyState')?.classList.add('hidden');
    }

    hideLoadingState() {
        document.getElementById('loadingState')?.classList.add('hidden');
        document.getElementById('pluginGrid')?.classList.remove('hidden');
    }

    showEmptyState() {
        document.getElementById('emptyState')?.classList.remove('hidden');
        document.getElementById('pluginGrid')?.classList.add('hidden');
    }

    hideEmptyState() {
        document.getElementById('emptyState')?.classList.add('hidden');
    }

    showErrorState() {
        this.showAlert('加载失败，请稍后重试', 'error');
        this.showEmptyState();
    }

    // 实用方法
    renderStars(rating) {
        const fullStars = Math.floor(rating);
        const hasHalfStar = rating % 1 >= 0.5;
        const emptyStars = 5 - fullStars - (hasHalfStar ? 1 : 0);
        
        let starsHtml = '';
        
        for (let i = 0; i < fullStars; i++) {
            starsHtml += '<i class="fas fa-star text-yellow-400"></i>';
        }
        
        if (hasHalfStar) {
            starsHtml += '<i class="fas fa-star-half-alt text-yellow-400"></i>';
        }
        
        for (let i = 0; i < emptyStars; i++) {
            starsHtml += '<i class="far fa-star text-gray-300"></i>';
        }
        
        return starsHtml;
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
        return date.toLocaleDateString('zh-CN');
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    updatePluginCount(count) {
        const pluginCount = document.getElementById('pluginCount');
        if (pluginCount) {
            pluginCount.textContent = count;
        }
    }

    showAlert(message, type = 'info') {
        // 简单的消息提示
        const alertDiv = document.createElement('div');
        alertDiv.className = `alert alert-${type} fixed top-4 right-4 z-50 max-w-sm`;
        alertDiv.textContent = message;
        
        document.body.appendChild(alertDiv);
        
        setTimeout(() => {
            alertDiv.remove();
        }, 3000);
    }

    // 分页相关（简化实现）
    renderPagination() {
        const paginationSection = document.getElementById('paginationSection');
        const pageNumbers = document.getElementById('pageNumbers');
        const prevPage = document.getElementById('prevPage');
        const nextPage = document.getElementById('nextPage');
        
        if (!paginationSection || this.totalPages <= 1) {
            paginationSection?.classList.add('hidden');
            return;
        }
        
        paginationSection.classList.remove('hidden');
        
        // 更新按钮状态
        if (prevPage) {
            prevPage.disabled = this.currentPage <= 1;
            prevPage.onclick = () => this.goToPage(this.currentPage - 1);
        }
        
        if (nextPage) {
            nextPage.disabled = this.currentPage >= this.totalPages;
            nextPage.onclick = () => this.goToPage(this.currentPage + 1);
        }
        
        // 渲染页码（简化版本）
        if (pageNumbers) {
            pageNumbers.innerHTML = `
                <span class="text-caption px-3 py-1">
                    ${this.currentPage} / ${this.totalPages}
                </span>
            `;
        }
    }

    goToPage(page) {
        if (page >= 1 && page <= this.totalPages) {
            this.currentPage = page;
            this.loadPlugins();
        }
    }

    // 文件上传相关方法需要在这里实现...
    setupFileUpload() {
        // 文件上传逻辑
    }

    // 登录相关方法需要在这里实现...
    showLoginModal() {
        this.showModal('loginModal');
    }

    showUploadModal() {
        this.showModal('uploadModal');
    }

    logout() {
        localStorage.removeItem('auth_token');
        localStorage.removeItem('current_user');
        this.authToken = null;
        this.currentUser = null;
        this.showLoggedOutState();
        this.showAlert('已退出登录', 'success');
    }

    // 其他必需的方法...
    async sendVerificationCode() {
        // 发送验证码逻辑
    }

    async verifyCode() {
        // 验证验证码逻辑
    }

    showSendCodeStep() {
        // 显示发送验证码步骤
    }

    handleFileSelect(file) {
        // 文件选择处理
    }

    async uploadPlugin() {
        // 上传插件逻辑
    }

    showRatingModal(pluginId) {
        // 显示评分模态框
    }

    setRating(rating) {
        // 设置评分
    }

    highlightStars(rating) {
        // 高亮星星
    }

    async submitRating() {
        // 提交评分
    }
}

// 初始化应用
let marketplace;
document.addEventListener('DOMContentLoaded', () => {
    marketplace = new PluginMarketplace();
});

// 全局导出，供HTML中的onclick使用
window.marketplace = marketplace;