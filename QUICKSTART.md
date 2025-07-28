# GeekTools Plugin Marketplace - Quick Start Guide

This guide will help you quickly set up and run the GeekTools Plugin Marketplace server locally.

## Prerequisites

Before starting, ensure you have the following installed:

- **Rust** (latest stable version)
- **PostgreSQL** (version 12 or higher)
- **Python 3** (for the proxy server)
- **Git**

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Install PostgreSQL
```bash
# macOS (using Homebrew)
brew install postgresql
brew services start postgresql

# Ubuntu/Debian
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

### Install sqlx-cli
```bash
cargo install sqlx-cli --no-default-features --features postgres
```

## Step 1: Clone and Navigate to Project

```bash
git clone <repository-url>
cd pluginmarket/server
```

## Step 2: Database Setup

### Create Database
```bash
# Connect to PostgreSQL as superuser
psql -U postgres  # or your PostgreSQL username

# Create the marketplace database
CREATE DATABASE marketplace;
\q
```

### Reset Database (if needed)
If you need to reset the database:
```bash
psql -U postgres -d marketplace
DROP SCHEMA public CASCADE; 
CREATE SCHEMA public;
\q
```

## Step 3: Environment Configuration

Copy the example environment file and configure it:
```bash
cp .env.example .env
```

Edit the `.env` file with your settings:
```bash
vim .env  # or use your preferred editor
```

**Key configurations to update:**
- `DATABASE_URL`: Update with your PostgreSQL credentials
- `JWT_SECRET`: Change to a secure random string for production
- `SMTP_*`: Configure email settings if needed (optional for development)

**Example `.env` configuration:**
```env
# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Database Configuration
DATABASE_URL=postgres://username:password@localhost:5432/marketplace
DATABASE_MAX_CONNECTIONS=10

# JWT Configuration (CHANGE THIS!)
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_ACCESS_TOKEN_EXPIRES_IN=3600
JWT_REFRESH_TOKEN_EXPIRES_IN=604800

# Storage Configuration
STORAGE_UPLOAD_PATH=./uploads
STORAGE_MAX_FILE_SIZE=104857600

# API Configuration  
API_BASE_URL=http://localhost:3000/api/v1

# Logging
RUST_LOG=info
RUST_BACKTRACE=1
```

## Step 4: Run Database Migrations

Apply the database schema:
```bash
sqlx migrate run
```

This will create all necessary tables and initialize the database structure.

## Step 5: Build the Application

Compile the Rust application:
```bash
cargo build
```

## Step 6: Start the Proxy Server (for Frontend)

The proxy server handles CORS issues when developing the frontend. Start it in the background:
```bash
cd ..  # Go back to project root
python proxy_server.py &
```

The proxy server will:
- Serve static files on http://localhost:8080
- Proxy `/api/*` requests to the backend server
- Handle CORS headers automatically

## Step 7: Start the Backend Server

```bash
cd server  # Go back to server directory
./target/debug/server
```

The server will start on http://localhost:3000 and you should see:
```
Starting GeekTools Marketplace Server on port 3000
Server listening on http://0.0.0.0:3000
```

## Step 8: Access the Application

- **Frontend**: http://localhost:8080 (served by Python proxy)
- **Backend API**: http://localhost:3000/api/v1
- **Admin Panel**: http://localhost:8080/admin.html

## Development Workflow

### Making Changes
1. Edit source code in `src/`
2. Rebuild: `cargo build`
3. Restart server: `./target/debug/server`

### Database Changes
1. Create new migration: `sqlx migrate add <migration_name>`
2. Edit the generated SQL file in `migrations/`
3. Apply migration: `sqlx migrate run`

### Running in Production Mode
```bash
# Build optimized binary
cargo build --release

# Run the optimized server
./target/release/server
```

## Troubleshooting

### Common Issues

**Database Connection Error:**
- Verify PostgreSQL is running: `brew services list | grep postgres` (macOS)
- Check DATABASE_URL in `.env` file
- Ensure database exists and user has permissions

**Port Already in Use:**
- Change SERVER_PORT in `.env` file
- Kill existing process: `lsof -ti:3000 | xargs kill -9`

**Migration Errors:**
- Reset database (see Step 2)
- Check migration files for syntax errors
- Ensure sqlx-cli is installed and up to date

**Compilation Errors:**
- Update Rust: `rustup update`
- Clean build cache: `cargo clean && cargo build`

### Logs and Debugging

Enable detailed logging:
```bash
export RUST_LOG=debug
./target/debug/server
```

## Next Steps

1. **Create Admin User**: Use the admin panel to create your first admin account
2. **Upload Plugins**: Test plugin upload functionality
3. **Configure SMTP**: Set up email for user verification (optional)
4. **Review Security**: Update JWT_SECRET and database credentials for production

## API Documentation

The server provides a RESTful API. Key endpoints:

- `GET /api/v1/plugins` - List all plugins
- `POST /api/v1/plugins` - Upload new plugin
- `GET /api/v1/plugins/{id}` - Get plugin details
- `POST /api/v1/auth/login` - User authentication
- `GET /api/v1/health` - Health check

For detailed API documentation, refer to the handler files in `src/handlers/`.

## Support

If you encounter issues:
1. Check the logs for error messages
2. Verify all prerequisites are installed
3. Ensure database is properly configured
4. Review the troubleshooting section above