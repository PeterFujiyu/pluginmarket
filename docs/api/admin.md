# 管理后台接口文档

## 概述

管理后台接口提供系统管理功能，包括用户管理、插件管理、数据统计、系统监控等。所有管理接口都需要管理员权限认证。

## 基础URL

```
生产环境: https://api.plugins.geektools.com/api/v1/admin
开发环境: http://localhost:3000/api/v1/admin
```

## 权限等级

| 权限等级 | 说明 | 功能范围 |
|---------|------|----------|
| **Admin** | 普通管理员 | 用户管理、插件管理、数据查看 |
| **SuperAdmin** | 超级管理员 | 所有功能 + SQL执行权限 |

## 认证要求

所有管理接口都需要在请求头中包含有效的管理员Token：

```
Authorization: Bearer {admin_token}
```

系统会验证Token的有效性并检查用户是否具有管理员权限。

## 接口详情

### 1. 获取仪表板统计

获取系统整体统计数据，用于管理仪表板展示。

**接口地址**: `GET /admin/dashboard`

**请求头**:
```
Authorization: Bearer {admin_token}  // 必需
Content-Type: application/json
```

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "overview": {
      "total_users": 1245,
      "total_plugins": 156,
      "total_downloads": 45678,
      "total_ratings": 892
    },
    "recent_activity": {
      "new_users_today": 12,
      "new_plugins_today": 3,
      "downloads_today": 234,
      "active_users_today": 89
    },
    "growth_metrics": {
      "users_growth_rate": 12.5,      // 用户增长率 (%)
      "plugins_growth_rate": 8.3,     // 插件增长率 (%)
      "downloads_growth_rate": 15.7   // 下载增长率 (%)
    },
    "system_health": {
      "database_status": "healthy",
      "storage_usage": {
        "used_gb": 15.4,
        "total_gb": 100.0,
        "usage_percentage": 15.4
      },
      "response_time_ms": 145
    },
    "top_plugins": [
      {
        "id": "system_monitor_demo",
        "name": "System Monitor Demo",
        "downloads": 1250,
        "rating": 4.5
      }
    ],
    "recent_users": [
      {
        "id": 123,
        "username": "newuser123",
        "email": "new***@example.com",
        "created_at": "2024-01-15T10:30:00Z"
      }
    ]
  }
}
```

❌ **权限不足** (403 Forbidden):
```json
{
  "success": false,
  "error": "需要管理员权限"
}
```

### 2. 用户管理

#### 2.1 获取用户列表

获取系统中所有用户的列表，支持分页和搜索。

**接口地址**: `GET /admin/users`

**查询参数**:
| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| page | integer | 否 | 1 | 页码 |
| limit | integer | 否 | 20 | 每页数量，最大100 |
| search | string | 否 | - | 搜索用户名或邮箱 |
| status | string | 否 | - | 用户状态过滤 (active/inactive/banned) |

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "users": [
      {
        "id": 123,
        "username": "john_doe",
        "email": "joh***@example.com",    // 邮箱脱敏显示
        "display_name": "John Doe",
        "is_active": true,
        "is_verified": true,
        "is_admin": false,
        "created_at": "2024-01-10T08:30:00Z",
        "updated_at": "2024-01-15T14:20:00Z",
        "last_login_at": "2024-01-15T09:15:00Z",
        "login_count": 25,
        "plugin_count": 3,
        "total_downloads": 456
      }
      // ... 更多用户
    ],
    "total_count": 1245
  }
}
```

#### 2.2 更新用户邮箱

管理员可以更新用户的邮箱地址。

**接口地址**: `PUT /admin/users/email`

**请求参数**:
```json
{
  "user_id": 123,                      // 目标用户ID，必需
  "new_email": "newemail@example.com", // 新邮箱地址，必需
  "reason": "string"                   // 修改原因，可选
}
```

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "message": "用户邮箱更新成功",
  "data": {}
}
```

❌ **邮箱已存在** (400 Bad Request):
```json
{
  "success": false,
  "error": "Email already exists"
}
```

#### 2.3 封禁用户

封禁违规用户账户。

**接口地址**: `POST /admin/users/ban`

**请求参数**:
```json
{
  "user_id": 123,                    // 目标用户ID，必需
  "reason": "spam behavior",         // 封禁原因，必需
  "duration_days": 30               // 封禁天数，可选，null为永久封禁
}
```

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "message": "用户封禁成功",
  "data": {
    "user_id": 123,
    "banned_until": "2024-02-15T10:30:00Z",
    "reason": "spam behavior"
  }
}
```

#### 2.4 解封用户

解除用户账户封禁。

**接口地址**: `POST /admin/users/unban`

**请求参数**:
```json
{
  "user_id": 123,              // 目标用户ID，必需
  "reason": "appeal approved"  // 解封原因，可选
}
```

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "message": "用户解封成功",
  "data": {}
}
```

### 3. 插件管理

#### 3.1 获取插件管理列表

获取所有插件的管理视图，包含详细的管理信息。

**接口地址**: `GET /admin/plugins`

**查询参数**:
| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| page | integer | 否 | 1 | 页码 |
| limit | integer | 否 | 20 | 每页数量 |
| status | string | 否 | - | 插件状态过滤 |
| search | string | 否 | - | 搜索插件名称或作者 |

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
        "author": "GeekTools Team",
        "author_email": "tea***@geektools.com",
        "current_version": "1.2.0",
        "status": "active",
        "downloads": 1250,
        "rating": 4.5,
        "total_ratings": 61,
        "file_size": 245760,
        "created_at": "2024-01-10T08:30:00Z",
        "updated_at": "2024-01-15T14:20:00Z",
        "last_download_at": "2024-01-15T16:45:00Z",
        "violation_reports": 0,
        "is_featured": false
      }
      // ... 更多插件
    ],
    "total_count": 156
  }
}
```

#### 3.2 删除插件

管理员删除违规或有问题的插件。

**接口地址**: `DELETE /admin/plugins`

**请求参数**:
```json
{
  "plugin_id": "malicious_plugin",  // 插件ID，必需
  "reason": "security violation",   // 删除原因，必需
  "notify_author": true            // 是否通知作者，可选，默认true
}
```

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "message": "插件删除成功",
  "data": {
    "plugin_id": "malicious_plugin",
    "deleted_at": "2024-01-15T10:30:00Z",
    "reason": "security violation"
  }
}
```

### 4. 登录活动监控

#### 4.1 获取用户登录活动

查看用户的登录活动记录。

**接口地址**: `GET /admin/login-activities`

**查询参数**:
| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| user_id | integer | 否 | - | 特定用户ID，不指定则显示所有用户 |
| page | integer | 否 | 1 | 页码 |
| limit | integer | 否 | 20 | 每页数量 |

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "activities": [
      {
        "id": 1001,
        "user_id": 123,
        "username": "john_doe",
        "email": "joh***@example.com",
        "ip_address": "192.168.1.100",
        "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "login_at": "2024-01-15T10:30:00Z",
        "success": true,
        "failure_reason": null,
        "location": {
          "country": "China",
          "city": "Beijing",
          "region": "Beijing"
        }
      },
      {
        "id": 1000,
        "user_id": 456,
        "username": "failed_user",
        "email": "fai***@example.com",
        "ip_address": "192.168.1.200",
        "user_agent": "curl/7.68.0",
        "login_at": "2024-01-15T10:25:00Z",
        "success": false,
        "failure_reason": "invalid_credentials",
        "location": {
          "country": "China",
          "city": "Shanghai",
          "region": "Shanghai"
        }
      }
    ],
    "total_count": 5678
  }
}
```

#### 4.2 获取最近登录活动

获取最近的登录活动，用于仪表板显示。

**接口地址**: `GET /admin/recent-logins`

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": [
    {
      "id": 1001,
      "user_id": 123,
      "username": "john_doe",
      "ip_address": "192.168.1.100",
      "login_at": "2024-01-15T10:30:00Z",
      "success": true,
      "location": {
        "country": "China",
        "city": "Beijing"
      }
    }
    // ... 最近20条记录
  ]
}
```

### 5. SQL执行器 (超级管理员)

直接执行SQL查询的高级功能，仅限超级管理员使用。

**接口地址**: `POST /admin/sql/execute`

⚠️ **警告**: 此功能具有极高的危险性，仅限超级管理员在紧急情况下使用，并且会记录详细的操作日志。

**请求参数**:
```json
{
  "query": "SELECT COUNT(*) as total FROM users WHERE created_at >= '2024-01-01'",
  "description": "统计2024年新用户数量"  // 查询说明，必需
}
```

**安全限制**:
- 只允许SELECT查询
- 禁止DELETE、DROP、TRUNCATE等危险操作
- 查询结果最多返回1000行
- 查询超时时间为30秒
- 所有操作都会记录审计日志

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "query": "SELECT COUNT(*) as total FROM users WHERE created_at >= '2024-01-01'",
    "execution_time_ms": 25,
    "row_count": 1,
    "columns": ["total"],
    "rows": [
      {"total": 245}
    ],
    "warnings": []
  }
}
```

❌ **权限不足** (403 Forbidden):
```json
{
  "success": false,
  "error": "Only super administrators can execute SQL queries"
}
```

❌ **危险查询** (400 Bad Request):
```json
{
  "success": false,
  "error": "DELETE operations are not allowed"
}
```

## 审计日志

所有管理操作都会记录到审计日志中，包括：

- 操作时间和操作者
- 操作类型和详细内容
- 操作结果和影响范围
- 客户端IP地址和User-Agent
- 操作前后的数据变化

**审计日志示例**:
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "admin_id": 1,
  "admin_email": "admin@geektools.com",
  "action": "ban_user",
  "target_type": "user",
  "target_id": "123",
  "details": {
    "reason": "spam behavior",
    "duration_days": 30
  },
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0...",
  "result": "success"
}
```

## 错误码说明

| HTTP状态码 | 错误类型 | 说明 |
|-----------|----------|------|
| 200 OK | 成功 | 操作成功完成 |
| 400 Bad Request | 请求错误 | 参数错误或业务逻辑错误 |
| 401 Unauthorized | 认证失败 | Token无效或已过期 |
| 403 Forbidden | 权限不足 | 非管理员或权限级别不够 |
| 404 Not Found | 资源不存在 | 目标用户或插件不存在 |
| 422 Unprocessable Entity | 验证失败 | 请求参数验证失败 |
| 500 Internal Server Error | 服务器错误 | 系统内部错误 |

## 限流规则

| 操作类型 | 限制规则 |
|---------|----------|
| 查询操作 | 每分钟200次 |
| 修改操作 | 每分钟50次 |
| 删除操作 | 每分钟10次 |
| SQL执行 | 每分钟5次 |

## 示例代码

### JavaScript管理客户端示例

```javascript
class AdminClient {
  constructor(baseUrl, adminToken) {
    this.baseUrl = baseUrl;
    this.adminToken = adminToken;
  }

  async request(endpoint, options = {}) {
    const response = await fetch(`${this.baseUrl}/admin${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.adminToken}`,
        ...options.headers,
      },
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Request failed');
    }

    return await response.json();
  }

  // 获取仪表板数据
  async getDashboard() {
    return await this.request('/dashboard');
  }

  // 获取用户列表
  async getUsers(params = {}) {
    const queryString = new URLSearchParams(params).toString();
    return await this.request(`/users?${queryString}`);
  }

  // 封禁用户
  async banUser(userId, reason, durationDays = null) {
    return await this.request('/users/ban', {
      method: 'POST',
      body: JSON.stringify({
        user_id: userId,
        reason,
        duration_days: durationDays,
      }),
    });
  }

  // 解封用户
  async unbanUser(userId, reason = null) {
    return await this.request('/users/unban', {
      method: 'POST',
      body: JSON.stringify({
        user_id: userId,
        reason,
      }),
    });
  }

  // 删除插件
  async deletePlugin(pluginId, reason, notifyAuthor = true) {
    return await this.request('/plugins', {
      method: 'DELETE',
      body: JSON.stringify({
        plugin_id: pluginId,
        reason,
        notify_author: notifyAuthor,
      }),
    });
  }

  // 获取登录活动
  async getLoginActivities(params = {}) {
    const queryString = new URLSearchParams(params).toString();
    return await this.request(`/login-activities?${queryString}`);
  }

  // 执行SQL查询 (仅超级管理员)
  async executeSQL(query, description) {
    return await this.request('/sql/execute', {
      method: 'POST',
      body: JSON.stringify({
        query,
        description,
      }),
    });
  }
}

// 使用示例
const admin = new AdminClient('http://localhost:3000/api/v1', 'admin-token');

// 获取仪表板数据
const dashboard = await admin.getDashboard();
console.log('系统概览:', dashboard.data.overview);

// 搜索用户
const users = await admin.getUsers({ search: 'john', limit: 10 });
console.log('找到用户:', users.data.users.length);

// 封禁垃圾邮件用户
await admin.banUser(123, '发送垃圾邮件', 30);
console.log('用户已被封禁30天');
```

### Python管理脚本示例

```python
import requests
import json
from datetime import datetime

class AdminClient:
    def __init__(self, base_url, admin_token):
        self.base_url = base_url
        self.admin_token = admin_token
        self.session = requests.Session()
        self.session.headers.update({
            'Authorization': f'Bearer {admin_token}',
            'Content-Type': 'application/json'
        })

    def get_dashboard(self):
        """获取仪表板数据"""
        response = self.session.get(f'{self.base_url}/admin/dashboard')
        return response.json()

    def get_users(self, **params):
        """获取用户列表"""
        response = self.session.get(f'{self.base_url}/admin/users', params=params)
        return response.json()

    def ban_user(self, user_id, reason, duration_days=None):
        """封禁用户"""
        data = {
            'user_id': user_id,
            'reason': reason
        }
        if duration_days:
            data['duration_days'] = duration_days
            
        response = self.session.post(f'{self.base_url}/admin/users/ban', json=data)
        return response.json()

    def get_plugins(self, **params):
        """获取插件列表"""
        response = self.session.get(f'{self.base_url}/admin/plugins', params=params)
        return response.json()

    def execute_sql(self, query, description):
        """执行SQL查询"""
        data = {
            'query': query,
            'description': description
        }
        response = self.session.post(f'{self.base_url}/admin/sql/execute', json=data)
        return response.json()

# 使用示例
admin = AdminClient('http://localhost:3000/api/v1', 'admin-token')

# 生成每日报告
def generate_daily_report():
    dashboard = admin.get_dashboard()
    
    print("=== GeekTools 插件市场每日报告 ===")
    print(f"报告时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print()
    
    overview = dashboard['data']['overview']
    print("系统概览:")
    print(f"  总用户数: {overview['total_users']}")
    print(f"  总插件数: {overview['total_plugins']}")
    print(f"  总下载数: {overview['total_downloads']}")
    print()
    
    activity = dashboard['data']['recent_activity']
    print("今日活动:")
    print(f"  新增用户: {activity['new_users_today']}")
    print(f"  新增插件: {activity['new_plugins_today']}")
    print(f"  今日下载: {activity['downloads_today']}")
    print(f"  活跃用户: {activity['active_users_today']}")

# 批量处理违规用户
def handle_spam_users():
    # 获取最近注册但未验证邮箱的用户
    users = admin.get_users(status='unverified', limit=100)
    
    for user in users['data']['users']:
        # 检查是否为垃圾邮箱域名
        if user['email'].endswith(('@tempmail.com', '@10minutemail.com')):
            print(f"封禁垃圾邮箱用户: {user['username']}")
            admin.ban_user(user['id'], '使用临时邮箱注册', 365)

# 执行报告生成
generate_daily_report()
```

## 安全建议

### 管理员账户安全

1. **强密码策略**: 管理员账户必须使用强密码
2. **双因素认证**: 启用2FA提高账户安全性
3. **IP白名单**: 限制管理界面访问IP范围
4. **定期轮换**: 定期更换管理员Token

### 操作审计

1. **详细日志**: 记录所有管理操作的详细信息
2. **实时监控**: 监控异常管理操作
3. **定期审查**: 定期审查管理操作日志
4. **报警机制**: 设置敏感操作报警

### 权限控制

1. **最小权限**: 按需分配管理权限
2. **角色划分**: 区分普通管理员和超级管理员
3. **操作限制**: 限制危险操作的执行频率
4. **审批流程**: 重要操作需要多人审批

## 常见问题

### Q: 如何区分普通管理员和超级管理员？
A: 系统根据用户的角色字段判断，超级管理员可以执行SQL查询等高危操作。

### Q: 删除的插件是否可以恢复？
A: 删除操作是永久性的，但会在审计日志中保留删除记录，建议在删除前做好备份。

### Q: SQL执行器有哪些安全限制？
A: 只允许SELECT查询，禁止修改数据的操作，结果限制1000行，查询超时30秒。

### Q: 管理操作是否有通知机制？
A: 重要操作（如封禁用户、删除插件）可以配置邮件通知相关用户。

### Q: 如何处理管理员权限被滥用的情况？
A: 通过审计日志可以追踪所有操作，发现异常后可以立即撤销权限并进行调查。