# 插件管理接口文档

## 概述

插件管理接口提供了插件的CRUD操作、文件上传下载、评分评论等功能。支持分页查询、搜索过滤、版本管理等高级特性。

## 基础URL

```
生产环境: https://api.plugins.geektools.com/api/v1
开发环境: http://localhost:3000/api/v1
```

## 接口详情

### 1. 获取插件列表

获取插件列表，支持分页、搜索和排序。

**接口地址**: `GET /plugins`

**请求头**:
```
Content-Type: application/json
Authorization: Bearer {token}  // 可选，用于个性化推荐
```

**查询参数**:
| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| page | integer | 否 | 1 | 页码，从1开始 |
| limit | integer | 否 | 20 | 每页数量，最大100 |
| search | string | 否 | - | 搜索关键词 |
| tag | string | 否 | - | 标签过滤 |
| sort | string | 否 | downloads | 排序字段 (downloads/rating/created_at/updated_at) |
| order | string | 否 | desc | 排序方向 (asc/desc) |

**请求示例**:
```bash
curl -X GET "http://localhost:3000/api/v1/plugins?page=1&limit=10&search=system&sort=rating&order=desc"
```

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "plugins": [
      {
        "id": "system_monitor_demo",
        "name": "System Monitor Demo",
        "description": "A demo plugin for system monitoring",
        "author": "GeekTools Team",
        "current_version": "1.2.0",
        "downloads": 1250,
        "rating": 4.5,
        "status": "active",
        "created_at": "2024-01-10T08:30:00Z",
        "updated_at": "2024-01-15T14:20:00Z",
        "min_geektools_version": "1.0.0",
        "homepage_url": "https://github.com/geektools/system-monitor",
        "repository_url": "https://github.com/geektools/system-monitor",
        "license": "MIT",
        "tags": ["system", "monitoring", "demo"]
      }
      // ... 更多插件
    ],
    "pagination": {
      "page": 1,
      "limit": 10,
      "total": 156,
      "pages": 16
    }
  }
}
```

**分页信息说明**:
- `page`: 当前页码
- `limit`: 每页数量
- `total`: 总记录数
- `pages`: 总页数

### 2. 获取插件详情

获取指定插件的详细信息。

**接口地址**: `GET /plugins/{plugin_id}`

**路径参数**:
- `plugin_id`: 插件ID (string)

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "system_monitor_demo",
    "name": "System Monitor Demo",
    "description": "A comprehensive system monitoring plugin that displays CPU, memory, disk usage and network statistics in real-time.",
    "author": "GeekTools Team",
    "current_version": "1.2.0",
    "downloads": 1250,
    "rating": 4.5,
    "status": "active",
    "created_at": "2024-01-10T08:30:00Z",
    "updated_at": "2024-01-15T14:20:00Z",
    "min_geektools_version": "1.0.0",
    "homepage_url": "https://github.com/geektools/system-monitor",
    "repository_url": "https://github.com/geektools/system-monitor",
    "license": "MIT",
    "tags": ["system", "monitoring", "demo"],
    "versions": [
      {
        "version": "1.2.0",
        "changelog": "Added network monitoring support",
        "file_size": 245760,
        "downloads": 850,
        "created_at": "2024-01-15T14:20:00Z",
        "is_stable": true
      },
      {
        "version": "1.1.0",
        "changelog": "Improved CPU monitoring accuracy",
        "file_size": 238592,
        "downloads": 400,
        "created_at": "2024-01-12T10:15:00Z",
        "is_stable": true
      }
    ],
    "recent_ratings": [
      {
        "id": 15,
        "user": {
          "username": "developer123",
          "display_name": "Dev User"
        },
        "rating": 5,
        "review": "Excellent plugin, very useful for system monitoring!",
        "created_at": "2024-01-14T16:30:00Z"
      }
    ],
    "rating_distribution": {
      "5": 45,
      "4": 12,
      "3": 3,
      "2": 1,
      "1": 0
    }
  }
}
```

❌ **插件不存在** (404 Not Found):
```json
{
  "success": false,
  "error": "Plugin not found"
}
```

### 3. 上传插件

上传新的插件包文件。需要认证。

**接口地址**: `POST /plugins/upload`

**请求头**:
```
Content-Type: multipart/form-data
Authorization: Bearer {token}  // 必需
```

**表单参数**:
- `plugin_file`: 插件文件 (file, 必需) - 必须为.tar.gz格式

**文件要求**:
- 文件格式: `.tar.gz`
- 最大大小: 100MB
- 必须包含有效的插件元数据文件

**请求示例**:
```bash
curl -X POST \
  -H "Authorization: Bearer your_token_here" \
  -F "plugin_file=@/path/to/plugin.tar.gz" \
  http://localhost:3000/api/v1/plugins/upload
```

**响应示例**:

✅ **成功响应** (201 Created):
```json
{
  "success": true,
  "message": "Plugin uploaded successfully",
  "data": {
    "id": "my_awesome_plugin",
    "name": "My Awesome Plugin",
    "version": "1.0.0",
    "upload_id": "550e8400-e29b-41d4-a716-446655440000",
    "file_size": 1048576,
    "file_hash": "sha256:a3b2c1d4e5f6...",
    "status": "processing",
    "created_at": "2024-01-15T10:30:00Z"
  }
}
```

❌ **文件格式错误** (400 Bad Request):
```json
{
  "success": false,
  "error": "File must be a .tar.gz archive"
}
```

❌ **文件过大** (400 Bad Request):
```json
{
  "success": false,
  "error": "File too large"
}
```

❌ **认证失败** (401 Unauthorized):
```json
{
  "success": false,
  "error": "Authentication required"
}
```

**插件包结构要求**:
```
plugin.tar.gz
├── plugin.json          # 插件元数据文件
├── README.md           # 插件说明文档
├── src/                # 源代码目录
│   ├── main.py        # 主程序文件
│   └── utils.py       # 工具函数
├── assets/            # 资源文件目录
│   └── icon.png       # 插件图标
└── requirements.txt   # 依赖说明文件
```

**plugin.json示例**:
```json
{
  "id": "my_awesome_plugin",
  "name": "My Awesome Plugin",
  "version": "1.0.0",
  "description": "A plugin that does awesome things",
  "author": "Plugin Developer",
  "license": "MIT",
  "min_geektools_version": "1.0.0",
  "homepage_url": "https://github.com/user/plugin",
  "repository_url": "https://github.com/user/plugin",
  "tags": ["utility", "productivity"],
  "main_script": "src/main.py",
  "dependencies": ["requests", "numpy"],
  "permissions": ["network", "filesystem"]
}
```

### 4. 下载插件

下载指定版本的插件文件。

**接口地址**: `GET /plugins/{plugin_id}/download`

**路径参数**:
- `plugin_id`: 插件ID (string)

**查询参数**:
| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| version | string | 否 | latest | 指定版本号，默认为最新版本 |

**请求示例**:
```bash
curl -X GET "http://localhost:3000/api/v1/plugins/system_monitor_demo/download?version=1.2.0" \
  -o plugin.tar.gz
```

**响应**:
- **成功**: 返回文件二进制流，Content-Type为`application/gzip`
- **失败**: 返回JSON错误信息

**响应头**:
```
Content-Type: application/gzip
Content-Disposition: attachment; filename="system_monitor_demo-1.2.0.tar.gz"
Content-Length: 245760
```

❌ **插件不存在** (404 Not Found):
```json
{
  "success": false,
  "error": "Plugin version not found"
}
```

**下载统计**:
- 每次成功下载会自动增加下载计数
- 统计信息会实时更新到插件详情中

### 5. 获取插件统计

获取插件的统计信息。

**接口地址**: `GET /plugins/{plugin_id}/stats`

**路径参数**:
- `plugin_id`: 插件ID (string)

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "plugin_id": "system_monitor_demo",
    "total_downloads": 1250,
    "monthly_downloads": 450,
    "weekly_downloads": 125,
    "daily_downloads": 18,
    "average_rating": 4.5,
    "total_ratings": 61,
    "rating_distribution": {
      "5": 45,
      "4": 12,
      "3": 3,
      "2": 1,
      "1": 0
    },
    "version_downloads": {
      "1.2.0": 850,
      "1.1.0": 400
    },
    "created_at": "2024-01-10T08:30:00Z",
    "last_updated": "2024-01-15T14:20:00Z"
  }
}
```

### 6. 获取插件评分列表

获取插件的评分和评论列表。

**接口地址**: `GET /plugins/{plugin_id}/ratings`

**路径参数**:
- `plugin_id`: 插件ID (string)

**查询参数**:
| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| page | integer | 否 | 1 | 页码 |
| limit | integer | 否 | 20 | 每页数量 |

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "ratings": [
      {
        "id": 15,
        "user": {
          "id": 123,
          "username": "developer123",
          "display_name": "Dev User"
        },
        "rating": 5,
        "review": "Excellent plugin, very useful for system monitoring! The interface is clean and the data is accurate.",
        "created_at": "2024-01-14T16:30:00Z",
        "updated_at": "2024-01-14T16:30:00Z"
      },
      {
        "id": 14,
        "user": {
          "id": 456,
          "username": "user456",
          "display_name": "Another User"
        },
        "rating": 4,
        "review": "Good plugin, works as expected. Could use some UI improvements.",
        "created_at": "2024-01-13T10:15:00Z",
        "updated_at": "2024-01-13T10:15:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 61,
      "pages": 4
    },
    "summary": {
      "average_rating": 4.5,
      "total_ratings": 61,
      "rating_distribution": {
        "5": 45,
        "4": 12,
        "3": 3,
        "2": 1,
        "1": 0
      }
    }
  }
}
```

### 7. 创建/更新插件评分

为插件添加评分和评论。需要认证。

**接口地址**: `POST /plugins/{plugin_id}/ratings`

**路径参数**:
- `plugin_id`: 插件ID (string)

**请求头**:
```
Content-Type: application/json
Authorization: Bearer {token}  // 必需
```

**请求参数**:
```json
{
  "rating": 5,                    // 评分，1-5的整数，必需
  "review": "string"              // 评论内容，可选，最大1000字符
}
```

**响应示例**:

✅ **成功响应** (201 Created):
```json
{
  "success": true,
  "message": "Rating created successfully",
  "data": {
    "id": 16,
    "plugin_id": "system_monitor_demo",
    "user_id": 123,
    "rating": 5,
    "review": "Great plugin, highly recommended!",
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

✅ **更新已有评分** (200 OK):
```json
{
  "success": true,
  "message": "Rating updated successfully",
  "data": {
    "id": 16,
    "plugin_id": "system_monitor_demo",
    "user_id": 123,
    "rating": 4,
    "review": "Updated my review after using it more.",
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T11:45:00Z"
  }
}
```

❌ **验证错误** (422 Unprocessable Entity):
```json
{
  "success": false,
  "error": "Validation failed",
  "details": {
    "rating": ["Rating must be between 1 and 5"],
    "review": ["Review cannot exceed 1000 characters"]
  }
}
```

**评分规则**:
- 每个用户对每个插件只能有一个评分
- 重复提交会更新已有评分
- 评分范围: 1-5分
- 评论内容可选，最大1000字符

### 8. 临时上传接口 (开发环境)

用于开发环境测试的临时上传接口，无需认证。

**接口地址**: `POST /plugins/upload-temp`

**请求头**:
```
Content-Type: multipart/form-data
```

**表单参数**:
- `plugin_file`: 插件文件 (file, 必需)

**响应示例**:

✅ **成功响应** (201 Created):
```json
{
  "success": true,
  "message": "Plugin uploaded successfully",
  "data": {
    "id": "test_plugin",
    "name": "Test Plugin",
    "version": "1.0.0",
    "upload_id": "temp-upload-12345",
    "status": "active"
  }
}
```

**注意**: 此接口仅用于开发环境测试，生产环境中不应启用。

## 状态码说明

| HTTP状态码 | 说明 |
|-----------|------|
| 200 OK | 请求成功 |
| 201 Created | 资源创建成功 |
| 400 Bad Request | 请求参数错误 |
| 401 Unauthorized | 认证失败 |
| 403 Forbidden | 权限不足 |
| 404 Not Found | 资源不存在 |
| 413 Payload Too Large | 文件过大 |
| 422 Unprocessable Entity | 参数验证失败 |
| 429 Too Many Requests | 请求频率超限 |
| 500 Internal Server Error | 服务器内部错误 |

## 插件状态说明

| 状态 | 说明 |
|------|------|
| active | 正常可用状态 |
| deprecated | 已弃用，不推荐使用 |
| banned | 已禁用，无法下载 |
| processing | 上传后处理中 |

## 限流规则

| 接口类型 | 限制规则 |
|---------|----------|
| 查询接口 | 每分钟100次请求 |
| 上传接口 | 每小时10次上传 |
| 下载接口 | 每分钟50次下载 |
| 评分接口 | 每小时20次评分 |

## 示例代码

### JavaScript客户端示例

```javascript
class PluginClient {
  constructor(baseUrl, authToken) {
    this.baseUrl = baseUrl;
    this.authToken = authToken;
  }

  // 获取插件列表
  async getPlugins(params = {}) {
    const queryString = new URLSearchParams(params).toString();
    const response = await fetch(`${this.baseUrl}/plugins?${queryString}`, {
      headers: {
        'Authorization': `Bearer ${this.authToken}`,
      },
    });
    return await response.json();
  }

  // 获取插件详情
  async getPlugin(pluginId) {
    const response = await fetch(`${this.baseUrl}/plugins/${pluginId}`, {
      headers: {
        'Authorization': `Bearer ${this.authToken}`,
      },
    });
    return await response.json();
  }

  // 上传插件
  async uploadPlugin(file) {
    const formData = new FormData();
    formData.append('plugin_file', file);

    const response = await fetch(`${this.baseUrl}/plugins/upload`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.authToken}`,
      },
      body: formData,
    });
    return await response.json();
  }

  // 下载插件
  async downloadPlugin(pluginId, version = null) {
    const url = new URL(`${this.baseUrl}/plugins/${pluginId}/download`);
    if (version) {
      url.searchParams.append('version', version);
    }

    const response = await fetch(url);
    
    if (response.ok) {
      return await response.blob();
    } else {
      const error = await response.json();
      throw new Error(error.error);
    }
  }

  // 创建评分
  async createRating(pluginId, rating, review = null) {
    const response = await fetch(`${this.baseUrl}/plugins/${pluginId}/ratings`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.authToken}`,
      },
      body: JSON.stringify({ rating, review }),
    });
    return await response.json();
  }
}

// 使用示例
const client = new PluginClient('http://localhost:3000/api/v1', 'your-token');

// 获取热门插件
const popularPlugins = await client.getPlugins({
  sort: 'downloads',
  order: 'desc',
  limit: 10
});

// 上传插件
const fileInput = document.getElementById('plugin-file');
const file = fileInput.files[0];
const uploadResult = await client.uploadPlugin(file);

// 下载插件
const pluginBlob = await client.downloadPlugin('system_monitor_demo', '1.2.0');
const downloadUrl = URL.createObjectURL(pluginBlob);
const a = document.createElement('a');
a.href = downloadUrl;
a.download = 'plugin.tar.gz';
a.click();
```

### Python客户端示例

```python
import requests
import json

class PluginClient:
    def __init__(self, base_url, auth_token=None):
        self.base_url = base_url
        self.auth_token = auth_token
        self.session = requests.Session()
        if auth_token:
            self.session.headers.update({
                'Authorization': f'Bearer {auth_token}'
            })

    def get_plugins(self, **params):
        """获取插件列表"""
        response = self.session.get(f'{self.base_url}/plugins', params=params)
        return response.json()

    def get_plugin(self, plugin_id):
        """获取插件详情"""
        response = self.session.get(f'{self.base_url}/plugins/{plugin_id}')
        return response.json()

    def upload_plugin(self, file_path):
        """上传插件"""
        with open(file_path, 'rb') as f:
            files = {'plugin_file': f}
            response = self.session.post(
                f'{self.base_url}/plugins/upload',
                files=files
            )
        return response.json()

    def download_plugin(self, plugin_id, version=None, save_path=None):
        """下载插件"""
        params = {}
        if version:
            params['version'] = version
            
        response = self.session.get(
            f'{self.base_url}/plugins/{plugin_id}/download',
            params=params
        )
        
        if response.ok:
            filename = save_path or f'{plugin_id}.tar.gz'
            with open(filename, 'wb') as f:
                f.write(response.content)
            return filename
        else:
            return response.json()

    def create_rating(self, plugin_id, rating, review=None):
        """创建评分"""
        data = {'rating': rating}
        if review:
            data['review'] = review
            
        response = self.session.post(
            f'{self.base_url}/plugins/{plugin_id}/ratings',
            json=data
        )
        return response.json()

# 使用示例
client = PluginClient('http://localhost:3000/api/v1', 'your-token')

# 搜索插件
plugins = client.get_plugins(search='monitor', sort='rating', order='desc')
print(f"Found {plugins['data']['pagination']['total']} plugins")

# 下载插件
client.download_plugin('system_monitor_demo', version='1.2.0', save_path='./monitor.tar.gz')
```

## 常见问题

### Q: 上传的插件包有什么要求？
A: 插件包必须是.tar.gz格式，包含有效的plugin.json元数据文件，文件大小不超过100MB。

### Q: 如何实现断点续传？
A: 当前版本不支持断点续传，建议将大文件分割为多个小文件或使用压缩优化文件大小。

### Q: 插件版本管理是如何工作的？
A: 系统根据plugin.json中的version字段自动管理版本，同一插件ID可以有多个版本。

### Q: 评分是否可以修改？
A: 是的，用户可以随时修改自己的评分和评论，系统会自动更新插件的平均评分。

### Q: 如何实现插件的自动更新检查？
A: 客户端可以定期调用插件详情接口，比较本地版本与最新版本号来检查更新。