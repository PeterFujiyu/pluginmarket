# 认证接口文档

## 概述

GeekTools 插件市场使用基于邮箱验证码的认证系统，结合JWT Token进行会话管理。认证流程不需要传统的用户名密码，而是通过邮箱验证码实现安全登录。

## 基础URL

```
生产环境: https://api.plugins.geektools.com/api/v1
开发环境: http://localhost:3000/api/v1
```

## 认证流程

```mermaid
sequenceDiagram
    participant Client as 客户端
    participant API as API服务器
    participant Email as 邮箱服务

    Client->>API: 1. 发送验证码请求
    API->>Email: 2. 发送验证码邮件
    Email-->>Client: 3. 用户收到验证码
    Client->>API: 4. 提交验证码验证
    API-->>Client: 5. 返回JWT Token
    Client->>API: 6. 使用Token访问受保护资源
```

## 接口详情

### 1. 用户注册

注册新用户账户。

**接口地址**: `POST /auth/register`

**请求头**:
```
Content-Type: application/json
```

**请求参数**:
```json
{
  "username": "string",      // 用户名，3-100字符，必需
  "email": "string",         // 邮箱地址，必需
  "password": "string",      // 密码，最少8字符，必需
  "display_name": "string"   // 显示名称，可选，最大255字符
}
```

**参数验证规则**:
- `username`: 长度3-100字符，只能包含字母、数字、下划线
- `email`: 有效的邮箱地址格式
- `password`: 最少8字符，建议包含大小写字母、数字和特殊字符
- `display_name`: 可选，最大255字符

**响应示例**:

✅ **成功响应** (201 Created):
```json
{
  "success": true,
  "message": "User registered successfully",
  "data": {
    "id": 123,
    "username": "john_doe",
    "email": "john@example.com",
    "display_name": "John Doe",
    "is_active": true,
    "is_verified": false,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

❌ **错误响应** (400 Bad Request):
```json
{
  "success": false,
  "error": "Username or email already exists"
}
```

❌ **验证错误** (422 Unprocessable Entity):
```json
{
  "success": false,
  "error": "Validation failed",
  "details": {
    "username": ["Username must be between 3 and 100 characters"],
    "email": ["Invalid email format"],
    "password": ["Password must be at least 8 characters"]
  }
}
```

### 2. 发送验证码

向指定邮箱发送登录验证码。

**接口地址**: `POST /auth/send-code`

**请求头**:
```
Content-Type: application/json
```

**请求参数**:
```json
{
  "email": "string"  // 邮箱地址，必需
}
```

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "message": "验证码已发送到您的邮箱，请查收",
    "code": null  // 生产环境下为null，开发环境下可能显示验证码
  }
}
```

**开发环境响应** (SMTP未配置时):
```json
{
  "success": true,
  "data": {
    "message": "验证码已生成，请查看下方显示的验证码",
    "code": "123456"  // 开发环境下直接显示验证码
  }
}
```

❌ **错误响应** (400 Bad Request):
```json
{
  "success": false,
  "error": "Invalid email format"
}
```

**限流规则**:
- 同一邮箱每分钟最多发送1次验证码
- 同一IP地址每分钟最多发送5次验证码
- 验证码有效期为5分钟

### 3. 验证码登录

使用邮箱验证码进行登录认证。

**接口地址**: `POST /auth/verify-code`

**请求头**:
```
Content-Type: application/json
```

**请求参数**:
```json
{
  "email": "string",  // 邮箱地址，必需
  "code": "string"    // 6位验证码，必需
}
```

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "message": "登录成功",
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 3600,  // Access Token过期时间(秒)
    "user": {
      "id": 123,
      "username": "john_doe",
      "email": "john@example.com",
      "display_name": "John Doe",
      "is_active": true,
      "is_verified": true
    }
  }
}
```

❌ **错误响应** (400 Bad Request):
```json
{
  "success": false,
  "error": "Invalid or expired verification code"
}
```

❌ **用户不存在** (404 Not Found):
```json
{
  "success": false,
  "error": "User not found with this email"
}
```

**验证码规则**:
- 验证码为6位数字
- 每个验证码只能使用一次
- 验证码5分钟内有效
- 验证码错误3次后将被锁定15分钟

### 4. 传统用户名密码登录

支持传统的用户名密码登录方式。

**接口地址**: `POST /auth/login`

**请求头**:
```
Content-Type: application/json
```

**请求参数**:
```json
{
  "username": "string",  // 用户名或邮箱，必需
  "password": "string"   // 密码，必需
}
```

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 3600,
    "user": {
      "id": 123,
      "username": "john_doe",
      "email": "john@example.com",
      "display_name": "John Doe",
      "is_active": true,
      "is_verified": true
    }
  }
}
```

❌ **认证失败** (401 Unauthorized):
```json
{
  "success": false,
  "error": "Invalid credentials"
}
```

### 5. 刷新Token

使用Refresh Token获取新的Access Token。

**接口地址**: `POST /auth/refresh`

**请求头**:
```
Content-Type: application/json
```

**请求参数**:
```json
{
  "refresh_token": "string"  // Refresh Token，必需
}
```

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 3600
  }
}
```

❌ **Token无效** (401 Unauthorized):
```json
{
  "success": false,
  "error": "Invalid refresh token"
}
```

## JWT Token 规范

### Access Token

- **算法**: HS256 (HMAC-SHA256)
- **有效期**: 1小时 (3600秒)
- **用途**: API访问认证

**Token载荷示例**:
```json
{
  "sub": "123",                    // 用户ID
  "user_id": 123,                  // 用户ID
  "username": "john_doe",          // 用户名
  "email": "john@example.com",     // 邮箱
  "iat": 1642248000,              // 签发时间
  "exp": 1642251600,              // 过期时间
  "iss": "geektools-marketplace"   // 签发者
}
```

### Refresh Token

- **算法**: HS256 (HMAC-SHA256)
- **有效期**: 7天 (604800秒)
- **用途**: 刷新Access Token

## 认证中间件

### 使用Bearer Token

所有需要认证的接口都需要在请求头中包含有效的Access Token：

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### 认证状态

- **已认证**: 提供有效的Access Token
- **未认证**: 未提供Token或Token无效
- **Token过期**: 需要使用Refresh Token刷新

## 错误码说明

| HTTP状态码 | 错误类型 | 说明 |
|-----------|----------|------|
| 400 | Bad Request | 请求参数错误或格式不正确 |
| 401 | Unauthorized | 认证失败或Token无效 |
| 403 | Forbidden | 权限不足 |
| 404 | Not Found | 用户不存在 |
| 422 | Unprocessable Entity | 参数验证失败 |
| 429 | Too Many Requests | 请求频率超限 |
| 500 | Internal Server Error | 服务器内部错误 |

## 安全建议

### 客户端实现

1. **Token存储**: 使用secure storage存储Token，避免XSS攻击
2. **自动刷新**: 在Access Token过期前自动刷新
3. **错误处理**: 妥善处理认证失败和Token过期
4. **安全传输**: 生产环境必须使用HTTPS

### 服务端安全

1. **Token签名**: 使用强密钥签名JWT Token
2. **有效期控制**: 合理设置Token有效期
3. **IP绑定**: 可选择绑定Token与IP地址
4. **异常监控**: 监控异常登录行为

## 示例代码

### JavaScript/前端示例

```javascript
class AuthClient {
  constructor(baseUrl) {
    this.baseUrl = baseUrl;
    this.accessToken = localStorage.getItem('access_token');
    this.refreshToken = localStorage.getItem('refresh_token');
  }

  // 发送验证码
  async sendVerificationCode(email) {
    const response = await fetch(`${this.baseUrl}/auth/send-code`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ email }),
    });
    
    return await response.json();
  }

  // 验证码登录
  async verifyCodeLogin(email, code) {
    const response = await fetch(`${this.baseUrl}/auth/verify-code`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ email, code }),
    });
    
    const result = await response.json();
    
    if (result.success) {
      this.accessToken = result.data.access_token;
      this.refreshToken = result.data.refresh_token;
      
      localStorage.setItem('access_token', this.accessToken);
      localStorage.setItem('refresh_token', this.refreshToken);
    }
    
    return result;
  }

  // 自动刷新Token
  async refreshAccessToken() {
    if (!this.refreshToken) {
      throw new Error('No refresh token available');
    }

    const response = await fetch(`${this.baseUrl}/auth/refresh`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ refresh_token: this.refreshToken }),
    });
    
    const result = await response.json();
    
    if (result.success) {
      this.accessToken = result.data.access_token;
      this.refreshToken = result.data.refresh_token;
      
      localStorage.setItem('access_token', this.accessToken);
      localStorage.setItem('refresh_token', this.refreshToken);
    }
    
    return result;
  }

  // 带认证的API请求
  async authenticatedRequest(url, options = {}) {
    const headers = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    if (this.accessToken) {
      headers['Authorization'] = `Bearer ${this.accessToken}`;
    }

    let response = await fetch(url, {
      ...options,
      headers,
    });

    // Token过期时自动刷新
    if (response.status === 401) {
      await this.refreshAccessToken();
      headers['Authorization'] = `Bearer ${this.accessToken}`;
      response = await fetch(url, {
        ...options,
        headers,
      });
    }

    return response;
  }
}
```

### Rust/后端示例

```rust
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

// JWT中间件
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 从Authorization头获取Token
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    if let Some(token) = auth_header {
        // 验证JWT Token
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(token_data) => {
                // 将用户信息添加到请求扩展中
                request.extensions_mut().insert(token_data.claims);
                Ok(next.run(request).await)
            }
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
```

## 常见问题

### Q: 验证码收不到怎么办？
A: 检查垃圾邮件文件夹，或联系管理员检查SMTP配置。开发环境下验证码会直接在响应中返回。

### Q: Token过期了如何处理？
A: 使用Refresh Token调用 `/auth/refresh` 接口获取新的Access Token。

### Q: 可以同时登录多个设备吗？
A: 是的，每次登录都会生成新的Token对，支持多设备同时登录。

### Q: 如何注销登录？
A: 客户端删除本地存储的Token即可。服务端可以选择实现Token黑名单机制。

### Q: 密码找回功能？
A: 系统主要使用邮箱验证码登录，如果忘记密码可以直接使用验证码登录，然后修改密码。