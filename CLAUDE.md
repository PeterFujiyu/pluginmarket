# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GeekTools Plugin Marketplace is a full-stack web application for managing and distributing plugins. It consists of:
- **Frontend**: HTML/JS/CSS with Tailwind CSS for the UI
- **Backend**: Rust-based API server using Axum framework
- **Database**: PostgreSQL with SQLx migrations
- **Proxy**: Python-based development proxy server for CORS handling

## Development Commands

### Backend (Rust)
```bash
# Navigate to server directory
cd server/

# Build the application
cargo build

# Run in development mode
cargo run

# Build for production
cargo build --release

# Run production binary
./target/release/server

# Install database migration tool
cargo install sqlx-cli --no-default-features --features postgres

# Run database migrations
sqlx migrate run

# Create new migration
sqlx migrate add <migration_name>

# Reset database (development)
sqlx migrate revert
```

### Frontend Development
```bash
# Start proxy server (handles CORS and serves static files)
python3 proxy_server.py

# Alternative: Simple HTTP server
python3 -m http.server 8080
```

### Docker Deployment
```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Database Operations
```bash
# Connect to PostgreSQL
psql -U postgres

# Create database
CREATE DATABASE marketplace;

# Reset database schema
DROP SCHEMA public CASCADE; 
CREATE SCHEMA public;
```

## Project Architecture

### Backend Structure (`server/src/`)
- `main.rs` - Application entry point, router setup, CORS configuration
- `handlers/` - HTTP request handlers (auth, plugins, admin, search, health)
- `services/` - Business logic layer (auth, plugin management, storage, SMTP)
- `models/` - Data models and database entities
- `middleware/` - Auth middleware and rate limiting
- `utils/` - Configuration, validation utilities
- `migrations/` - Database schema migrations

### Key Components
- **Authentication**: JWT-based with email verification codes
- **File Upload**: Plugin tar.gz files with validation
- **Admin Panel**: User management, plugin management, SQL execution
- **Search**: Advanced search with filters and suggestions
- **Rate Limiting**: Built-in request rate limiting
- **SMTP Integration**: Email sending via Lettre crate

### Frontend Files
- `index.html` - Main plugin marketplace interface
- `admin.html` - Admin management panel
- `config-manager.html` - Configuration management interface
- `backup-manager.html` - Database backup management interface
- `status-monitor.html` - System status monitoring interface
- `config-history.html` - Configuration history and rollback interface
- `app.js` - Main application logic
- `admin.js` - Admin panel functionality
- `config-manager.js` - Configuration management logic
- `backup-manager.js` - Backup management logic
- `status-monitor.js` - System monitoring logic
- `config-history.js` - Configuration history logic
- `config.js` - Frontend configuration and API settings
- `proxy_server.py` - Development proxy for CORS handling

## Configuration

### Environment Variables (`server/.env`)
```bash
# Database
DATABASE_URL=postgres://username:password@localhost:5432/marketplace
DATABASE_MAX_CONNECTIONS=10

# JWT
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_ACCESS_TOKEN_EXPIRES_IN=3600
JWT_REFRESH_TOKEN_EXPIRES_IN=604800

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Storage
STORAGE_UPLOAD_PATH=./uploads
STORAGE_MAX_FILE_SIZE=104857600

# SMTP (optional)
SMTP_ENABLED=false
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password

# Logging
RUST_LOG=info
RUST_BACKTRACE=1
```

## API Endpoints

### Authentication
- `POST /api/v1/auth/register` - User registration
- `POST /api/v1/auth/login` - User login
- `POST /api/v1/auth/send-code` - Send email verification code
- `POST /api/v1/auth/verify-code` - Verify code and login
- `POST /api/v1/auth/refresh` - Refresh JWT token

### Plugins
- `GET /api/v1/plugins` - List all plugins with pagination
- `POST /api/v1/plugins` - Upload new plugin (authenticated)
- `POST /api/v1/plugins/upload` - Upload plugin (temporary, no auth)
- `GET /api/v1/plugins/:id` - Get plugin details
- `GET /api/v1/plugins/:id/download` - Download plugin file
- `POST /api/v1/plugins/:id/ratings` - Create plugin rating

### Admin Management
- `GET /api/v1/admin/dashboard` - Dashboard statistics
- `GET /api/v1/admin/users` - User management with pagination
- `POST /api/v1/admin/users/update-email` - Update user email address
- `POST /api/v1/admin/users/ban` - Ban user account
- `POST /api/v1/admin/users/unban` - Unban user account
- `GET /api/v1/admin/plugins` - Plugin management with pagination
- `POST /api/v1/admin/plugins/delete` - Delete plugin
- `GET /api/v1/admin/login-activities` - User login activity history
- `GET /api/v1/admin/recent-logins` - Recent login activities
- `POST /api/v1/admin/sql/execute` - Execute SQL queries

### Configuration Management (New)
- `GET /api/v1/admin/config` - Get current configuration
- `POST /api/v1/admin/config/update` - Update configuration
- `POST /api/v1/admin/config/test` - Test configuration (SMTP, DB, etc.)
- `POST /api/v1/admin/config/rollback` - Rollback to previous configuration
- `GET /api/v1/admin/config/history` - Get configuration history
- `POST /api/v1/admin/config/snapshot` - Create configuration snapshot
- `POST /api/v1/admin/config/compare` - Compare configuration versions

### Backup Management (New)
- `GET /api/v1/admin/backup/list` - List all backups
- `POST /api/v1/admin/backup/create` - Create new backup
- `POST /api/v1/admin/backup/restore` - Restore from backup
- `DELETE /api/v1/admin/backup/:id` - Delete backup
- `GET /api/v1/admin/backup/:id/download` - Download backup file
- `GET /api/v1/admin/backup/status` - Get backup operation status
- `POST /api/v1/admin/backup/schedule` - Configure scheduled backups
- `GET /api/v1/admin/backup/schedules` - List backup schedules

### System Monitoring (New)
- `GET /api/v1/admin/monitor/system` - System metrics (CPU, memory, disk)
- `GET /api/v1/admin/monitor/services` - Service health status
- `GET /api/v1/admin/monitor/database` - Database status and metrics
- `GET /api/v1/admin/monitor/smtp` - SMTP service status
- `GET /api/v1/admin/monitor/logs` - System logs with filtering
- `POST /api/v1/admin/monitor/test/email` - Send test email
- `POST /api/v1/admin/monitor/test/database` - Test database connection

### Health & Monitoring
- `GET /api/v1/health` - Health check endpoint
- `GET /api/v1/metrics` - Application metrics

## Database Schema

Main tables:
- `users` - User accounts with email verification
- `plugins` - Plugin metadata and file information
- `ratings` - Plugin ratings and reviews
- `user_login_activities` - Login tracking for admin

## Testing

### Manual Testing
Access test pages:
- Frontend: http://localhost:8080
- Admin panel: http://localhost:8080/admin.html
- Configuration manager: http://localhost:8080/config-manager.html
- Backup manager: http://localhost:8080/backup-manager.html
- Status monitor: http://localhost:8080/status-monitor.html
- Config history: http://localhost:8080/config-history.html
- Health check: http://localhost:3000/api/v1/health

### Development Workflow
1. Start PostgreSQL database
2. Set up environment variables in `server/.env`
3. Run database migrations: `sqlx migrate run`
4. Start backend: `cargo run` (from server/ directory)
5. Start proxy: `python3 proxy_server.py` (from project root)
6. Access frontend at http://localhost:8080

## Key Dependencies

### Rust Crates
- `axum` - Web framework
- `sqlx` - Database toolkit
- `tokio` - Async runtime
- `jsonwebtoken` - JWT authentication
- `lettre` - SMTP email sending
- `bcrypt` - Password hashing
- `serde` - Serialization
- `validator` - Input validation

### Frontend
- Tailwind CSS - Utility-first CSS framework
- Vanilla JavaScript - No additional frameworks
- Font Awesome - Icon library

## API Request/Response Examples

### Configuration Management

#### Get Current Configuration
```bash
GET /api/v1/admin/config
Authorization: Bearer <token>

Response:
{
  "success": true,
  "data": {
    "smtp": {
      "enabled": false,
      "host": "smtp.gmail.com",
      "port": 587,
      "username": "test@example.com",
      "password": "***",
      "from_address": "noreply@geektools.dev",
      "from_name": "GeekTools",
      "use_tls": true
    },
    "database": {
      "max_connections": 10,
      "connect_timeout": 30
    },
    "server": {
      "host": "0.0.0.0",
      "port": 3000,
      "jwt_access_token_expires_in": 3600,
      "jwt_refresh_token_expires_in": 604800
    },
    "storage": {
      "upload_path": "./uploads",
      "max_file_size": 104857600,
      "use_cdn": false,
      "cdn_base_url": "https://cdn.geektools.dev"
    }
  }
}
```

#### Update Configuration
```bash
POST /api/v1/admin/config/update
Authorization: Bearer <token>
Content-Type: application/json

{
  "config_type": "smtp",
  "config_data": {
    "enabled": true,
    "host": "smtp.office365.com",
    "port": 587,
    "username": "admin@company.com",
    "password": "new-password",
    "use_tls": true
  }
}

Response:
{
  "success": true,
  "message": "Configuration updated successfully",
  "data": {
    "version": "v2.1.4",
    "applied_at": "2024-01-20T15:30:00Z"
  }
}
```

### Backup Management

#### Create Backup
```bash
POST /api/v1/admin/backup/create
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "manual_backup_2024_01_20",
  "description": "Pre-upgrade backup",
  "type": "full",
  "compress": true
}

Response:
{
  "success": true,
  "data": {
    "backup_id": "backup_123",
    "status": "started",
    "estimated_duration": 180
  }
}
```

#### List Backups
```bash
GET /api/v1/admin/backup/list?page=1&limit=20
Authorization: Bearer <token>

Response:
{
  "success": true,
  "data": {
    "backups": [
      {
        "id": "backup_123",
        "name": "manual_backup_2024_01_20",
        "description": "Pre-upgrade backup",
        "type": "full",
        "status": "completed",
        "size": 157286400,
        "created_at": "2024-01-20T14:30:00Z",
        "created_by": "admin@company.com",
        "file_path": "/backups/backup_123.sql.gz"
      }
    ],
    "total_count": 15,
    "page": 1,
    "limit": 20
  }
}
```

### System Monitoring

#### Get System Metrics
```bash
GET /api/v1/admin/monitor/system
Authorization: Bearer <token>

Response:
{
  "success": true,
  "data": {
    "cpu": {
      "usage_percent": 25.3,
      "cores": 4,
      "load_average": [0.65, 0.72, 0.80]
    },
    "memory": {
      "total_gb": 16,
      "used_gb": 11.2,
      "usage_percent": 70.0,
      "available_gb": 4.8
    },
    "disk": {
      "total_gb": 500,
      "used_gb": 229,
      "usage_percent": 45.8,
      "free_gb": 271
    },
    "network": {
      "bytes_sent": 1048576000,
      "bytes_recv": 2097152000,
      "packets_sent": 1000000,
      "packets_recv": 1500000
    },
    "uptime_seconds": 2847392,
    "timestamp": "2024-01-20T15:30:00Z"
  }
}
```

#### Get Service Status
```bash
GET /api/v1/admin/monitor/services
Authorization: Bearer <token>

Response:
{
  "success": true,
  "data": {
    "services": [
      {
        "name": "web_server",
        "status": "healthy",
        "response_time_ms": 12,
        "uptime_percent": 99.9,
        "last_check": "2024-01-20T15:29:45Z",
        "details": {
          "port": 3000,
          "active_connections": 47
        }
      },
      {
        "name": "database",
        "status": "healthy",
        "response_time_ms": 8,
        "uptime_percent": 99.8,
        "last_check": "2024-01-20T15:29:45Z",
        "details": {
          "active_connections": 15,
          "max_connections": 100,
          "queries_per_second": 45.7
        }
      },
      {
        "name": "smtp",
        "status": "warning",
        "response_time_ms": null,
        "uptime_percent": 0,
        "last_check": "2024-01-20T15:29:45Z",
        "details": {
          "configured": false,
          "last_email_sent": null
        }
      }
    ]
  }
}
```

## Frontend Configuration

### config.js Settings
The frontend uses `config.js` for dynamic configuration:

```javascript
window.GeekToolsConfig = {
  // API base URL - automatically detected or manual override
  apiBaseUrl: '/api/v1',
  
  // Admin panel settings
  admin: {
    autoRefreshInterval: 30000,  // 30 seconds
    config: {
      autoBackupOnChange: true,
      maxVersionHistory: 50
    },
    backup: {
      defaultRetentionDays: 30,
      maxBackupSize: 1024,
      defaultScheduleTime: '02:00'
    },
    monitor: {
      refreshInterval: 30000,
      chartDataPoints: 50,
      logLimit: 50
    }
  }
};
```

## Production Deployment

For production deployment:
1. Set strong JWT_SECRET in environment
2. Configure proper database credentials
3. Set up SMTP for email verification
4. Use reverse proxy (Nginx) for static file serving
5. Enable HTTPS/SSL certificates
6. Set up proper backup procedures for database and uploads
7. Configure monitoring alerts for system health
8. Set up log rotation and retention policies