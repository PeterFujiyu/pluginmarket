# GeekTools 插件市场技术文档

## 文档导航

本文档库包含 GeekTools 插件市场项目的完整技术文档，适用于开发、部署和运维。

### 📁 文档结构

```
docs/
├── README.md                 # 本文档，文档导航
├── architecture/             # 架构设计文档
│   ├── overview.md          # 系统架构概述
│   ├── components.md        # 组件设计详解
│   └── security.md          # 安全架构设计
├── api/                     # API接口文档
│   ├── authentication.md   # 认证接口
│   ├── plugins.md          # 插件管理接口
│   ├── admin.md            # 管理后台接口
│   └── search.md           # 搜索接口
├── database/                # 数据库设计文档
│   ├── schema.md           # 数据库模式设计
│   ├── migrations.md       # 数据库迁移管理
│   └── performance.md      # 性能优化策略
├── frontend/                # 前端开发文档
│   ├── ui-components.md    # UI组件设计
│   ├── styling.md          # 样式系统
│   └── user-flows.md       # 用户交互流程
├── deployment/              # 部署运维文档
│   ├── production.md       # 生产环境部署
│   ├── docker.md           # Docker容器化部署
│   └── monitoring.md       # 监控和日志
└── development/             # 开发环境文档
    ├── setup.md            # 开发环境搭建
    ├── testing.md          # 测试策略
    └── contributing.md     # 开发贡献指南
```

### 🚀 快速开始

1. **开发者**: 从 [开发环境搭建](development/setup.md) 开始
2. **运维人员**: 查看 [生产环境部署](deployment/production.md)
3. **API集成**: 参考 [API接口文档](api/) 目录
4. **架构理解**: 阅读 [系统架构概述](architecture/overview.md)

### 📋 项目信息

- **项目名称**: GeekTools Plugin Marketplace
- **版本**: 1.0.0
- **技术栈**: Rust (Axum) + SQLite + HTML/JS/CSS
- **许可证**: MIT
- **文档更新日期**: 2024-08-01

### 🔗 相关链接

- [项目源码](https://github.com/geektools/pluginmarket)
- [问题反馈](https://github.com/geektools/pluginmarket/issues)
- [生产环境](https://plugins.geektools.com)

### 📝 文档维护

本文档库由 GeekTools 团队维护，如有问题请提交 Issue 或 Pull Request。