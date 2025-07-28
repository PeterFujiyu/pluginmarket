# GeekTools Plugin Marketplace - Docker Deployment

This guide provides instructions for deploying the GeekTools Plugin Marketplace using Docker and Docker Compose.

## Quick Start

### Prerequisites

- Docker Engine 20.10+
- Docker Compose 2.0+ (or docker-compose 1.29+)

### One-Command Installation

Run the interactive installer:

```bash
chmod +x docker/install.sh
./docker/install.sh
```

The installer will:
- Create the `data` directory structure
- Generate secure configuration with interactive prompts
- Build and start all services
- Provide access URLs and management commands

## Manual Setup

### 1. Create Data Directory

```bash
mkdir -p data
```

### 2. Configure Environment

Create `data/config.env` based on your requirements:

```bash
cp server/.env.example data/config.env
# Edit data/config.env with your settings
```

**Key configurations:**
- `DATABASE_URL`: PostgreSQL connection (auto-configured for Docker)
- `JWT_SECRET`: Change to a secure random string
- `SMTP_*`: Configure email settings (optional)

### 3. Set Docker Environment

Create `.env` file in project root:

```env
POSTGRES_PASSWORD=your_secure_password
APP_PORT=3000
PROXY_PORT=8080
POSTGRES_PORT=5432
```

### 4. Start Services

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f
```

## Services

The deployment includes three services:

### PostgreSQL Database (`postgres`)
- **Image**: `postgres:15-alpine`
- **Port**: 5432 (configurable)
- **Data**: Persisted in `postgres_data` volume
- **Health Check**: Built-in PostgreSQL readiness check

### Application Server (`app`)
- **Build**: From project Dockerfile
- **Port**: 3000 (configurable)
- **Config**: Mounted from `./data/config.env`
- **Uploads**: Persisted in `app_uploads` volume
- **Dependencies**: Waits for PostgreSQL health check

### Proxy Server (`proxy`)
- **Image**: `python:3.11-alpine`
- **Port**: 8080 (configurable)
- **Purpose**: Serves frontend and proxies API requests
- **CORS**: Automatically handles cross-origin requests

## Configuration

### Environment Variables

The application supports configuration through environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `POSTGRES_PASSWORD` | Database password | `defaultpassword` |
| `APP_PORT` | Application server port | `3000` |
| `PROXY_PORT` | Proxy server port | `8080` |
| `POSTGRES_PORT` | Database port | `5432` |

### Volume Mounts

| Host Path | Container Path | Purpose |
|-----------|----------------|---------|
| `./data` | `/data` | Configuration files |
| `postgres_data` | `/var/lib/postgresql/data` | Database data |
| `app_uploads` | `/app/uploads` | Plugin uploads |

### Configuration File

The `data/config.env` file contains all application settings:

```env
# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Database Configuration  
DATABASE_URL=postgres://postgres:password@postgres:5432/marketplace

# JWT Configuration
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_ACCESS_TOKEN_EXPIRES_IN=3600
JWT_REFRESH_TOKEN_EXPIRES_IN=604800

# Storage Configuration
STORAGE_UPLOAD_PATH=./uploads
STORAGE_MAX_FILE_SIZE=104857600

# SMTP Configuration (Optional)
SMTP_ENABLED=false
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_ADDRESS=noreply@geektools.dev
SMTP_FROM_NAME=GeekTools Plugin Marketplace

# API Configuration
API_BASE_URL=http://localhost:3000/api/v1

# Logging
RUST_LOG=info
RUST_BACKTRACE=1
```

## Access URLs

After deployment, access the application at:

- **Frontend**: http://localhost:8080
- **API**: http://localhost:3000/api/v1
- **Admin Panel**: http://localhost:8080/admin.html
- **Health Check**: http://localhost:3000/api/v1/health

## Management Commands

### View Logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f app
docker-compose logs -f postgres
docker-compose logs -f proxy
```

### Restart Services
```bash
# All services
docker-compose restart

# Specific service
docker-compose restart app
```

### Stop Services
```bash
docker-compose down
```

### Update and Restart
```bash
docker-compose pull
docker-compose up -d
```

### Database Operations
```bash
# Access PostgreSQL shell
docker-compose exec postgres psql -U postgres -d marketplace

# Run migrations manually
docker-compose exec app ./sqlx migrate run

# Backup database
docker-compose exec postgres pg_dump -U postgres marketplace > backup.sql

# Restore database
docker-compose exec -T postgres psql -U postgres marketplace < backup.sql
```

### Application Operations
```bash
# Access application shell
docker-compose exec app bash

# View application logs
docker-compose exec app tail -f /var/log/app.log

# Check application status
curl http://localhost:3000/api/v1/health
```

## Troubleshooting

### Common Issues

**Services won't start:**
```bash
# Check service status
docker-compose ps

# View detailed logs
docker-compose logs

# Rebuild containers
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

**Database connection issues:**
```bash
# Check PostgreSQL logs
docker-compose logs postgres

# Verify database is ready
docker-compose exec postgres pg_isready -U postgres

# Reset database (WARNING: destroys data)
docker-compose down -v
docker-compose up -d
```

**Configuration not loading:**
```bash
# Verify config file exists
ls -la data/config.env

# Check file permissions
chmod 644 data/config.env

# Restart application
docker-compose restart app
```

**Port conflicts:**
```bash
# Check what's using the port
lsof -i :3000
lsof -i :8080

# Change ports in .env file
echo "APP_PORT=3001" >> .env
echo "PROXY_PORT=8081" >> .env
docker-compose up -d
```

### Performance Tuning

**Database Performance:**
```env
# Add to data/config.env
DATABASE_MAX_CONNECTIONS=20
```

**Application Performance:**
```env
# Add to data/config.env
RUST_LOG=warn  # Reduce log verbosity
STORAGE_MAX_FILE_SIZE=52428800  # Reduce max file size
```

**Resource Limits:**
Add to `docker-compose.yml`:
```yaml
services:
  app:
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '0.5'
```

## Security Considerations

### Production Deployment

1. **Change Default Passwords:**
   ```bash
   # Generate secure password
   openssl rand -base64 32
   ```

2. **Use HTTPS:**
   - Deploy behind reverse proxy (nginx/traefik)
   - Configure SSL certificates
   - Update API_BASE_URL to use HTTPS

3. **Network Security:**
   ```yaml
   # In docker-compose.yml, remove port exposure for internal services
   services:
     postgres:
       # ports:  # Comment out to make internal only
       #   - "5432:5432"
   ```

4. **File Permissions:**
   ```bash
   chmod 600 data/config.env  # Restrict config file access
   ```

### Backup Strategy

```bash
#!/bin/bash
# backup.sh - Regular backup script

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="./backups"

mkdir -p $BACKUP_DIR

# Backup database
docker-compose exec -T postgres pg_dump -U postgres marketplace > $BACKUP_DIR/db_$DATE.sql

# Backup uploads
docker-compose exec app tar czf - uploads > $BACKUP_DIR/uploads_$DATE.tar.gz

# Backup configuration
cp -r data $BACKUP_DIR/config_$DATE

echo "Backup completed: $DATE"
```

## Monitoring

### Health Checks

The application includes built-in health checks:

```bash
# Application health
curl http://localhost:3000/api/v1/health

# Database health  
docker-compose exec postgres pg_isready -U postgres
```

### Log Monitoring

```bash
# Monitor all logs
docker-compose logs -f --tail=100

# Monitor with timestamps
docker-compose logs -f -t

# Filter logs by service
docker-compose logs -f app | grep ERROR
```

## Support

For issues and questions:
1. Check the troubleshooting section above
2. Review application logs
3. Check the main project documentation
4. Create an issue in the project repository

## License

This Docker deployment configuration is part of the GeekTools Plugin Marketplace project and follows the same license terms.