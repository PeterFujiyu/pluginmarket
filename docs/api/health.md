# 健康检查和监控接口文档

## 概述

健康检查和监控接口提供系统运行状态监控、性能指标收集、服务可用性检查等功能。这些接口主要用于运维监控、负载均衡健康检查和系统诊断。

## 基础URL

```
生产环境: https://api.plugins.geektools.com/api/v1
开发环境: http://localhost:3000/api/v1
```

## 接口详情

### 1. 健康检查

基础的服务健康状态检查，用于负载均衡器和监控系统。

**接口地址**: `GET /health`

**请求头**: 无特殊要求

**响应示例**:

✅ **服务健康** (200 OK):
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2024-01-15T10:30:00Z",
    "version": "1.0.0",
    "services": {
      "database": "healthy",
      "storage": "healthy"
    },
    "uptime_seconds": 86400,
    "environment": "production"
  }
}
```

⚠️ **服务异常** (200 OK，但状态为unhealthy):
```json
{
  "success": true,
  "data": {
    "status": "unhealthy",
    "timestamp": "2024-01-15T10:30:00Z",
    "version": "1.0.0",
    "services": {
      "database": "timeout",
      "storage": "healthy"
    },
    "uptime_seconds": 86400,
    "environment": "production",
    "issues": [
      {
        "service": "database",
        "status": "timeout",
        "message": "Database query timeout after 5 seconds",
        "since": "2024-01-15T10:25:00Z"
      }
    ]
  }
}
```

**服务状态说明**:
- `healthy`: 服务正常运行
- `unhealthy`: 服务异常，但仍可响应
- `timeout`: 服务响应超时
- `error`: 服务错误
- `degraded`: 服务降级运行

**检查项目**:
1. **数据库连接**: 执行简单查询验证数据库可用性
2. **文件存储**: 检查文件系统访问权限和空间
3. **内存使用**: 检查内存使用情况
4. **磁盘空间**: 检查磁盘空间使用率

### 2. 就绪检查

更详细的服务就绪状态检查，确保服务能够处理请求。

**接口地址**: `GET /health/ready`

**请求头**: 无特殊要求

**响应示例**:

✅ **服务就绪** (200 OK):
```json
{
  "success": true,
  "data": {
    "status": "ready",
    "timestamp": "2024-01-15T10:30:00Z",
    "checks": {
      "database_connection": {
        "status": "pass",
        "response_time_ms": 15,
        "details": "PostgreSQL connection successful"
      },
      "database_migrations": {
        "status": "pass",
        "details": "All migrations applied",
        "version": "20240127000004"
      },
      "storage_access": {
        "status": "pass",
        "response_time_ms": 5,
        "details": "File system accessible"
      },
      "smtp_service": {
        "status": "pass",
        "details": "SMTP configuration valid"
      },
      "memory_usage": {
        "status": "pass",
        "usage_percentage": 45.2,
        "threshold": 80.0
      },
      "disk_space": {
        "status": "pass",
        "usage_percentage": 15.4,
        "threshold": 90.0
      }
    },
    "startup_time": "2024-01-15T08:00:00Z",
    "initialization_duration_ms": 2500
  }
}
```

❌ **服务未就绪** (503 Service Unavailable):
```json
{
  "success": false,
  "data": {
    "status": "not_ready",
    "timestamp": "2024-01-15T10:30:00Z",
    "checks": {
      "database_connection": {
        "status": "fail",
        "response_time_ms": 5000,
        "details": "Connection timeout",
        "error": "timeout after 5 seconds"
      },
      "database_migrations": {
        "status": "fail",
        "details": "Pending migrations found",
        "pending_migrations": ["20240127000005"]
      }
    },
    "failed_checks": ["database_connection", "database_migrations"]
  }
}
```

**检查状态**:
- `pass`: 检查通过
- `fail`: 检查失败
- `warn`: 检查有警告但不影响服务

### 3. 存活检查

简单的存活状态检查，仅验证服务进程是否运行。

**接口地址**: `GET /health/live`

**请求头**: 无特殊要求

**响应示例**:

✅ **服务存活** (200 OK):
```json
{
  "success": true,
  "data": {
    "status": "alive",
    "timestamp": "2024-01-15T10:30:00Z",
    "process_id": 12345,
    "uptime_seconds": 86400
  }
}
```

### 4. 性能指标

获取详细的系统性能指标和统计数据。

**接口地址**: `GET /metrics`

**请求头**: 无特殊要求

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "application_metrics": {
      "total_plugins": 156,
      "total_users": 1245,
      "total_downloads": 45678,
      "weekly_new": 12,
      "active_sessions": 89
    },
    "database_metrics": {
      "connection_pool": {
        "active_connections": 8,
        "idle_connections": 12,
        "max_connections": 20
      },
      "query_performance": {
        "average_query_time_ms": 25.6,
        "slow_queries_count": 3,
        "total_queries": 15678
      }
    },
    "system_metrics": {
      "memory": {
        "used_mb": 512,
        "available_mb": 1536,
        "usage_percentage": 25.0
      },
      "cpu": {
        "usage_percentage": 15.5,
        "load_average": {
          "1min": 0.5,
          "5min": 0.7,
          "15min": 0.6
        }
      },
      "disk": {
        "total_gb": 100,
        "used_gb": 15.4,
        "available_gb": 84.6,
        "usage_percentage": 15.4
      },
      "network": {
        "bytes_sent": 1048576000,
        "bytes_received": 2097152000,
        "packets_sent": 1000000,
        "packets_received": 1500000
      }
    },
    "http_metrics": {
      "total_requests": 50000,
      "requests_per_second": 25.5,
      "response_times": {
        "p50_ms": 45,
        "p95_ms": 150,
        "p99_ms": 300
      },
      "status_codes": {
        "2xx": 47500,
        "4xx": 2000,
        "5xx": 500
      },
      "endpoint_stats": [
        {
          "path": "/api/v1/plugins",
          "method": "GET",
          "count": 15000,
          "avg_response_time_ms": 35
        },
        {
          "path": "/api/v1/plugins/upload",
          "method": "POST", 
          "count": 500,
          "avg_response_time_ms": 250
        }
      ]
    },
    "error_metrics": {
      "error_rate": 0.05,
      "recent_errors": [
        {
          "timestamp": "2024-01-15T10:25:00Z",
          "level": "ERROR",
          "message": "Database connection timeout",
          "count": 3
        }
      ]
    },
    "uptime_seconds": 86400,
    "collection_time": "2024-01-15T10:30:00Z"
  }
}
```

### 5. Prometheus格式指标

提供Prometheus兼容的指标格式，用于监控系统集成。

**接口地址**: `GET /metrics/prometheus`

**请求头**: 无特殊要求

**响应示例**:

✅ **成功响应** (200 OK):
```
# HELP geektools_plugins_total Total number of plugins
# TYPE geektools_plugins_total counter
geektools_plugins_total 156

# HELP geektools_users_total Total number of users  
# TYPE geektools_users_total counter
geektools_users_total 1245

# HELP geektools_downloads_total Total number of downloads
# TYPE geektools_downloads_total counter
geektools_downloads_total 45678

# HELP geektools_http_requests_total Total HTTP requests
# TYPE geektools_http_requests_total counter
geektools_http_requests_total{method="GET",status="200"} 47500
geektools_http_requests_total{method="POST",status="200"} 2000
geektools_http_requests_total{method="GET",status="404"} 1500

# HELP geektools_http_request_duration_seconds HTTP request duration
# TYPE geektools_http_request_duration_seconds histogram
geektools_http_request_duration_seconds_bucket{le="0.1"} 30000
geektools_http_request_duration_seconds_bucket{le="0.5"} 45000
geektools_http_request_duration_seconds_bucket{le="1.0"} 49500
geektools_http_request_duration_seconds_bucket{le="+Inf"} 50000

# HELP geektools_database_connections_active Active database connections
# TYPE geektools_database_connections_active gauge
geektools_database_connections_active 8

# HELP geektools_memory_usage_bytes Memory usage in bytes
# TYPE geektools_memory_usage_bytes gauge
geektools_memory_usage_bytes 536870912

# HELP geektools_disk_usage_bytes Disk usage in bytes
# TYPE geektools_disk_usage_bytes gauge
geektools_disk_usage_bytes{device="/data"} 16530855936
```

### 6. 深度健康检查

执行更全面的系统健康检查，包括依赖服务验证。

**接口地址**: `GET /health/deep`

**请求头**: 
```
Authorization: Bearer {admin_token}  // 需要管理员权限
```

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2024-01-15T10:30:00Z",
    "detailed_checks": {
      "database": {
        "postgresql": {
          "status": "healthy",
          "version": "14.9",
          "connections": {
            "active": 8,
            "max": 20,
            "utilization": 40.0
          },
          "performance": {
            "avg_query_time_ms": 25.6,
            "slow_queries": 3,
            "cache_hit_ratio": 95.5
          },
          "replication": {
            "status": "active",
            "lag_bytes": 0,
            "replicas": ["replica-1", "replica-2"]
          }
        }
      },
      "storage": {
        "filesystem": {
          "status": "healthy",
          "mount_point": "/data/uploads",
          "total_space_gb": 100,
          "used_space_gb": 15.4,
          "available_space_gb": 84.6,
          "inode_usage": 5.2
        },
        "permissions": {
          "status": "healthy",
          "readable": true,
          "writable": true,
          "executable": true
        }
      },
      "external_services": {
        "smtp": {
          "status": "healthy",
          "host": "smtp.gmail.com",
          "port": 587,
          "tls": true,
          "auth": true,
          "response_time_ms": 150
        },
        "dns": {
          "status": "healthy",
          "resolvers": ["8.8.8.8", "8.8.4.4"],
          "response_time_ms": 25
        }
      },
      "security": {
        "ssl_certificates": {
          "status": "healthy",
          "expires_at": "2024-12-31T23:59:59Z",
          "days_until_expiry": 351
        },
        "jwt_keys": {
          "status": "healthy",
          "key_rotation_due": false,
          "next_rotation": "2024-06-01T00:00:00Z"
        }
      },
      "business_logic": {
        "plugin_processing": {
          "status": "healthy",
          "queue_size": 5,
          "processing_rate": 2.5,
          "failed_jobs": 0
        },
        "email_delivery": {
          "status": "healthy",
          "queue_size": 12,
          "delivery_rate": 95.5,
          "bounce_rate": 1.2
        }
      }
    },
    "recommendations": [
      {
        "level": "info",
        "component": "database",
        "message": "Consider increasing connection pool size during peak hours",
        "action": "Monitor connection utilization and adjust pool size"
      }
    ],
    "overall_health_score": 98.5
  }
}
```

## 监控集成

### Kubernetes健康检查

适用于Kubernetes部署的健康检查配置：

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: geektools-marketplace
    image: geektools/marketplace:latest
    ports:
    - containerPort: 3000
    livenessProbe:
      httpGet:
        path: /api/v1/health/live
        port: 3000
      initialDelaySeconds: 30
      periodSeconds: 10
      timeoutSeconds: 5
      failureThreshold: 3
    readinessProbe:
      httpGet:
        path: /api/v1/health/ready
        port: 3000
      initialDelaySeconds: 10
      periodSeconds: 5
      timeoutSeconds: 3
      failureThreshold: 3
    startupProbe:
      httpGet:
        path: /api/v1/health
        port: 3000
      initialDelaySeconds: 10
      periodSeconds: 10
      timeoutSeconds: 5
      failureThreshold: 30
```

### Docker Compose健康检查

```yaml
version: '3.8'
services:
  marketplace:
    image: geektools/marketplace:latest
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/v1/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    depends_on:
      database:
        condition: service_healthy
```

### Nginx负载均衡器配置

```nginx
upstream marketplace_backend {
    server 127.0.0.1:3000 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:3001 max_fails=3 fail_timeout=30s;
}

server {
    location /health {
        proxy_pass http://marketplace_backend/api/v1/health;
        proxy_set_header Host $host;
        proxy_connect_timeout 5s;
        proxy_read_timeout 10s;
    }
    
    location / {
        proxy_pass http://marketplace_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

## 监控告警

### 告警规则示例

基于指标的告警规则：

```yaml
# Prometheus告警规则
groups:
- name: geektools_marketplace
  rules:
  - alert: HighErrorRate
    expr: geektools_http_requests_total{status=~"5.."} / geektools_http_requests_total > 0.05
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate detected"
      description: "Error rate is {{ $value }}% over 5 minutes"

  - alert: DatabaseConnectionHigh
    expr: geektools_database_connections_active / geektools_database_connections_max > 0.8
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "Database connection pool nearly full"
      description: "Connection utilization is {{ $value }}%"

  - alert: DiskSpaceHigh
    expr: geektools_disk_usage_bytes / geektools_disk_total_bytes > 0.9
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Disk space critical"
      description: "Disk usage is {{ $value }}%"

  - alert: ServiceDown
    expr: up{job="geektools_marketplace"} == 0
    for: 1m
    labels:
      severity: critical  
    annotations:
      summary: "GeekTools Marketplace service is down"
      description: "Service has been down for more than 1 minute"
```

## 示例代码

### Python监控脚本

```python
import requests
import time
import logging
from typing import Dict, List

class HealthMonitor:
    def __init__(self, base_url: str, admin_token: str = None):
        self.base_url = base_url
        self.admin_token = admin_token
        self.session = requests.Session()
        
        if admin_token:
            self.session.headers.update({
                'Authorization': f'Bearer {admin_token}'
            })
        
        logging.basicConfig(level=logging.INFO)
        self.logger = logging.getLogger(__name__)

    def check_health(self) -> Dict:
        """基础健康检查"""
        try:
            response = self.session.get(f'{self.base_url}/health', timeout=10)
            return response.json()
        except Exception as e:
            self.logger.error(f"Health check failed: {e}")
            return {"success": False, "error": str(e)}

    def check_readiness(self) -> Dict:
        """就绪状态检查"""
        try:
            response = self.session.get(f'{self.base_url}/health/ready', timeout=10)
            return response.json()
        except Exception as e:
            self.logger.error(f"Readiness check failed: {e}")
            return {"success": False, "error": str(e)}

    def get_metrics(self) -> Dict:
        """获取性能指标"""
        try:
            response = self.session.get(f'{self.base_url}/metrics', timeout=10)
            return response.json()
        except Exception as e:
            self.logger.error(f"Metrics collection failed: {e}")
            return {"success": False, "error": str(e)}

    def deep_health_check(self) -> Dict:
        """深度健康检查"""
        if not self.admin_token:
            raise ValueError("Admin token required for deep health check")
        
        try:
            response = self.session.get(f'{self.base_url}/health/deep', timeout=30)
            return response.json()
        except Exception as e:
            self.logger.error(f"Deep health check failed: {e}")
            return {"success": False, "error": str(e)}

    def monitor_continuously(self, interval: int = 60):
        """持续监控"""
        self.logger.info(f"Starting continuous monitoring (interval: {interval}s)")
        
        while True:
            try:
                # 基础健康检查
                health = self.check_health()
                if health.get('success'):
                    status = health['data']['status']
                    self.logger.info(f"Health status: {status}")
                    
                    if status != 'healthy':
                        self.logger.warning(f"Service unhealthy: {health['data']}")
                        self.send_alert("Service unhealthy", health['data'])
                else:
                    self.logger.error(f"Health check failed: {health}")
                    self.send_alert("Health check failed", health)

                # 性能指标检查
                metrics = self.get_metrics()
                if metrics.get('success'):
                    self.check_performance_thresholds(metrics['data'])

                time.sleep(interval)
                
            except KeyboardInterrupt:
                self.logger.info("Monitoring stopped by user")
                break
            except Exception as e:
                self.logger.error(f"Monitoring error: {e}")
                time.sleep(interval)

    def check_performance_thresholds(self, metrics: Dict):
        """检查性能阈值"""
        # 检查内存使用率
        memory_usage = metrics['system_metrics']['memory']['usage_percentage']
        if memory_usage > 80:
            self.logger.warning(f"High memory usage: {memory_usage}%")
            self.send_alert("High memory usage", {"usage": memory_usage})

        # 检查磁盘使用率
        disk_usage = metrics['system_metrics']['disk']['usage_percentage']
        if disk_usage > 90:
            self.logger.error(f"Critical disk usage: {disk_usage}%")
            self.send_alert("Critical disk usage", {"usage": disk_usage})

        # 检查数据库连接
        db_metrics = metrics['database_metrics']
        connection_utilization = (db_metrics['connection_pool']['active_connections'] / 
                                db_metrics['connection_pool']['max_connections']) * 100
        if connection_utilization > 80:
            self.logger.warning(f"High database connection usage: {connection_utilization}%")

        # 检查错误率
        error_rate = metrics['error_metrics']['error_rate']
        if error_rate > 0.05:  # 5%
            self.logger.error(f"High error rate: {error_rate}")
            self.send_alert("High error rate", {"rate": error_rate})

    def send_alert(self, title: str, details: Dict):
        """发送告警（示例实现）"""
        self.logger.error(f"ALERT: {title} - {details}")
        # 这里可以集成实际的告警系统，如发送邮件、Slack通知等

    def generate_health_report(self) -> str:
        """生成健康报告"""
        health = self.check_health()
        metrics = self.get_metrics()
        
        if self.admin_token:
            deep_health = self.deep_health_check()
        else:
            deep_health = None

        report = ["=== GeekTools Marketplace Health Report ==="]
        report.append(f"Timestamp: {time.strftime('%Y-%m-%d %H:%M:%S')}")
        report.append("")

        # 基础健康状态
        if health.get('success'):
            data = health['data']
            report.append(f"Overall Status: {data['status']}")
            report.append(f"Version: {data['version']}")
            report.append(f"Uptime: {data['uptime_seconds']} seconds")
            report.append("")
            
            report.append("Service Status:")
            for service, status in data['services'].items():
                report.append(f"  {service}: {status}")
        else:
            report.append(f"Health Check Failed: {health}")

        # 性能指标
        if metrics.get('success'):
            data = metrics['data']
            report.append("")
            report.append("Performance Metrics:")
            report.append(f"  Total Plugins: {data['application_metrics']['total_plugins']}")
            report.append(f"  Total Users: {data['application_metrics']['total_users']}")
            report.append(f"  Total Downloads: {data['application_metrics']['total_downloads']}")
            report.append(f"  Memory Usage: {data['system_metrics']['memory']['usage_percentage']}%")
            report.append(f"  Disk Usage: {data['system_metrics']['disk']['usage_percentage']}%")
            report.append(f"  CPU Usage: {data['system_metrics']['cpu']['usage_percentage']}%")

        # 深度检查结果
        if deep_health and deep_health.get('success'):
            data = deep_health['data']
            report.append("")
            report.append(f"Deep Health Check: {data['status']}")
            report.append(f"Health Score: {data['overall_health_score']}")
            
            if data.get('recommendations'):
                report.append("")
                report.append("Recommendations:")
                for rec in data['recommendations']:
                    report.append(f"  [{rec['level']}] {rec['component']}: {rec['message']}")

        return "\n".join(report)

# 使用示例
if __name__ == "__main__":
    monitor = HealthMonitor('http://localhost:3000/api/v1', 'admin-token')
    
    # 生成健康报告
    report = monitor.generate_health_report()
    print(report)
    
    # 启动持续监控（可选）
    # monitor.monitor_continuously(interval=30)
```

### JavaScript健康检查客户端

```javascript
class HealthChecker {
  constructor(baseUrl, adminToken = null) {
    this.baseUrl = baseUrl;
    this.adminToken = adminToken;
  }

  async checkHealth() {
    try {
      const response = await fetch(`${this.baseUrl}/health`);
      return await response.json();
    } catch (error) {
      return { success: false, error: error.message };
    }
  }

  async checkReadiness() {
    try {
      const response = await fetch(`${this.baseUrl}/health/ready`);
      return await response.json();
    } catch (error) {
      return { success: false, error: error.message };
    }
  }

  async getMetrics() {
    try {
      const response = await fetch(`${this.baseUrl}/metrics`);
      return await response.json();
    } catch (error) {
      return { success: false, error: error.message };
    }
  }

  // 创建健康状态仪表板
  createDashboard(containerId) {
    const container = document.getElementById(containerId);
    if (!container) return;

    container.innerHTML = `
      <div class="health-dashboard">
        <h2>System Health Dashboard</h2>
        <div id="health-status" class="status-card">
          <h3>Overall Status</h3>
          <div id="status-indicator" class="status-indicator loading">Checking...</div>
        </div>
        <div id="metrics-grid" class="metrics-grid">
          <div id="system-metrics" class="metric-card">
            <h4>System Metrics</h4>
            <div id="system-data"></div>
          </div>
          <div id="app-metrics" class="metric-card">
            <h4>Application Metrics</h4>
            <div id="app-data"></div>
          </div>
        </div>
      </div>
    `;

    this.updateDashboard();
    
    // 定期更新
    setInterval(() => this.updateDashboard(), 30000);
  }

  async updateDashboard() {
    // 更新健康状态
    const health = await this.checkHealth();
    const statusIndicator = document.getElementById('status-indicator');
    
    if (health.success) {
      const status = health.data.status;
      statusIndicator.textContent = status.toUpperCase();
      statusIndicator.className = `status-indicator ${status}`;
    } else {
      statusIndicator.textContent = 'ERROR';
      statusIndicator.className = 'status-indicator error';
    }

    // 更新指标
    const metrics = await this.getMetrics();
    if (metrics.success) {
      this.updateSystemMetrics(metrics.data.system_metrics);
      this.updateAppMetrics(metrics.data.application_metrics);
    }
  }

  updateSystemMetrics(systemMetrics) {
    const container = document.getElementById('system-data');
    container.innerHTML = `
      <div class="metric">
        <span class="label">Memory Usage:</span>
        <span class="value">${systemMetrics.memory.usage_percentage}%</span>
      </div>
      <div class="metric">
        <span class="label">CPU Usage:</span>
        <span class="value">${systemMetrics.cpu.usage_percentage}%</span>
      </div>
      <div class="metric">
        <span class="label">Disk Usage:</span>
        <span class="value">${systemMetrics.disk.usage_percentage}%</span>
      </div>
    `;
  }

  updateAppMetrics(appMetrics) {
    const container = document.getElementById('app-data');
    container.innerHTML = `
      <div class="metric">
        <span class="label">Total Plugins:</span>
        <span class="value">${appMetrics.total_plugins}</span>
      </div>
      <div class="metric">
        <span class="label">Total Users:</span>
        <span class="value">${appMetrics.total_users}</span>
      </div>
      <div class="metric">
        <span class="label">Total Downloads:</span>
        <span class="value">${appMetrics.total_downloads}</span>
      </div>
      <div class="metric">
        <span class="label">Active Sessions:</span>
        <span class="value">${appMetrics.active_sessions}</span>
      </div>
    `;
  }
}

// CSS样式
const styles = `
  .health-dashboard {
    font-family: Arial, sans-serif;
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
  }

  .status-card {
    background: white;
    border-radius: 8px;
    padding: 20px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    margin-bottom: 20px;
  }

  .status-indicator {
    display: inline-block;
    padding: 8px 16px;
    border-radius: 4px;
    font-weight: bold;
    text-transform: uppercase;
  }

  .status-indicator.healthy {
    background: #d4edda;
    color: #155724;
  }

  .status-indicator.unhealthy {
    background: #f8d7da;
    color: #721c24;
  }

  .status-indicator.loading {
    background: #fff3cd;
    color: #856404;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
  }

  .metric-card {
    background: white;
    border-radius: 8px;
    padding: 20px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  }

  .metric {
    display: flex;
    justify-content: space-between;
    padding: 8px 0;
    border-bottom: 1px solid #eee;
  }

  .metric:last-child {
    border-bottom: none;
  }

  .label {
    font-weight: 500;
  }

  .value {
    font-weight: bold;
    color: #007bff;
  }
`;

// 添加样式到页面
const styleSheet = document.createElement('style');
styleSheet.textContent = styles;
document.head.appendChild(styleSheet);

// 使用示例
const healthChecker = new HealthChecker('http://localhost:3000/api/v1');
healthChecker.createDashboard('health-dashboard-container');
```

## 常见问题

### Q: 健康检查接口是否需要认证？
A: 基础的健康检查接口（/health, /health/ready, /health/live）不需要认证，但深度健康检查需要管理员权限。

### Q: 如何在负载均衡器中配置健康检查？
A: 使用 `/health` 接口作为健康检查端点，设置合适的超时时间和重试次数。

### Q: 指标接口的数据更新频率是多少？
A: 大部分指标实时更新，部分统计数据（如趋势分析）可能有1-5分钟的延迟。

### Q: 如何集成到现有的监控系统？
A: 可以使用 `/metrics/prometheus` 接口与Prometheus集成，或直接调用 `/metrics` 接口解析JSON数据。

### Q: 系统异常时健康检查接口是否仍然可用？
A: 健康检查接口设计为高可用，即使在系统部分异常时也能响应，但会返回相应的错误状态信息。