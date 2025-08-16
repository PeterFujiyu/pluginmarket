# 头像上传功能文档

## 概述
为插件市场添加了完整的用户头像上传功能，支持图片上传、裁剪、存储和管理。

## 功能特性

### 前端功能
- **头像上传界面**：现代化的玻璃态设计
- **图片裁剪**：基于Cropper.js的专业图片编辑
- **拖拽上传**：支持拖拽和点击上传
- **实时预览**：裁剪时同步更新预览
- **文件验证**：类型和大小验证
- **错误处理**：友好的错误提示

### 后端功能
- **文件存储**：安全的文件存储系统
- **数据库管理**：头像元数据管理
- **RESTful API**：标准的REST API接口
- **权限控制**：用户身份验证
- **文件服务**：优化的文件服务

## API 接口

### 1. 上传头像
```http
POST /api/v1/user/avatar
Content-Type: multipart/form-data
Authorization: Bearer <token>

Body: avatar=<image_file>
```

**响应**：
```json
{
  "success": true,
  "message": "头像上传成功",
  "avatar_url": "/api/v1/avatars/avatar_123_uuid.jpg"
}
```

### 2. 获取头像文件
```http
GET /api/v1/avatars/{filename}
```

**响应**：图片文件（带适当的Content-Type和缓存头）

### 3. 获取用户头像信息
```http
GET /api/v1/user/avatar
Authorization: Bearer <token>
```

**响应**：
```json
{
  "success": true,
  "avatar_url": "/api/v1/avatars/avatar_123_uuid.jpg",
  "file_size": 123456,
  "upload_time": "2025-08-16T12:00:00Z"
}
```

### 4. 删除头像
```http
DELETE /api/v1/user/avatar
Authorization: Bearer <token>
```

**响应**：
```json
{
  "success": true,
  "message": "头像删除成功"
}
```

## 数据库结构

### 用户表更新
```sql
ALTER TABLE users ADD COLUMN avatar_url VARCHAR(500);
```

### 头像表
```sql
CREATE TABLE user_avatars (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    upload_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,
    UNIQUE(user_id, is_active) -- 每个用户只能有一个活跃头像
);
```

## 文件存储

### 目录结构
```
uploads/
└── avatars/
    ├── avatar_1_uuid1.jpg
    ├── avatar_2_uuid2.png
    └── ...
```

### 文件命名规则
- 格式：`avatar_{user_id}_{uuid}.{extension}`
- 支持格式：JPG, PNG, GIF, WebP
- 最大大小：5MB

## 安全特性

### 文件验证
- MIME类型验证
- 文件大小限制
- 文件名安全检查

### 权限控制
- JWT令牌验证
- 用户身份确认
- 资源访问控制

### 数据库安全
- 外键约束
- 事务处理
- SQL注入防护

## 部署说明

### 运行数据库迁移
```bash
cd server
sqlx migrate run
```

### 创建上传目录
```bash
mkdir -p uploads/avatars
mkdir -p uploads/temp
```

### 启动服务器
```bash
cd server
cargo run
```

## 测试指南

### 手动测试
1. 启动服务器和前端代理
2. 登录用户账户
3. 点击用户头像图标
4. 上传图片文件
5. 调整裁剪区域
6. 保存头像

### API测试
```bash
# 上传头像
curl -X POST \
  -H "Authorization: Bearer <token>" \
  -F "avatar=@test_image.jpg" \
  http://localhost:3000/api/v1/user/avatar

# 获取头像
curl http://localhost:3000/api/v1/avatars/avatar_1_uuid.jpg

# 删除头像
curl -X DELETE \
  -H "Authorization: Bearer <token>" \
  http://localhost:3000/api/v1/user/avatar
```

## 故障排除

### 常见问题

1. **上传失败**
   - 检查文件大小（最大5MB）
   - 检查文件格式（JPG/PNG/GIF/WebP）
   - 检查用户认证令牌

2. **图片无法显示**
   - 检查文件是否存在于uploads/avatars目录
   - 检查数据库中的记录
   - 检查文件权限

3. **数据库错误**
   - 确认数据库迁移已运行
   - 检查数据库连接
   - 检查外键约束

### 日志查看
```bash
# 查看服务器日志
RUST_LOG=debug cargo run

# 查看数据库查询日志
RUST_LOG=sqlx=debug cargo run
```

## 性能优化

### 缓存策略
- 头像文件缓存1天
- 使用适当的HTTP缓存头
- 考虑CDN集成

### 文件优化
- 图片压缩和优化
- 多尺寸头像生成
- 懒加载实现

## 扩展功能

### 未来改进
- [ ] 多尺寸头像生成
- [ ] 图片压缩优化
- [ ] CDN集成
- [ ] 头像历史记录
- [ ] 批量头像管理
- [ ] 头像审核系统

### 配置选项
可在config.yaml中添加头像相关配置：
```yaml
avatar:
  max_file_size: 5242880  # 5MB
  allowed_types: ["image/jpeg", "image/png", "image/gif", "image/webp"]
  storage_path: "uploads/avatars"
  enable_compression: true
  max_dimension: 1024
```