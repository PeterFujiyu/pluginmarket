# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GeekTools Plugin Marketplace is a full-stack web application for managing and distributing plugins. It consists of:
- **Frontend**: React TypeScript application with Tailwind CSS for the UI
- **Backend**: Rust-based API server using Axum framework
- **Database**: SQLite with SQLx migrations

## Development Commands

### Package Management
```bash
# Install all dependencies (frontend and backend)
pnpm install

# Install dependencies and build backend
pnpm run install:all
```

### Development Workflow
```bash
# Start all services (React frontend and Rust backend)
pnpm run dev

# Start individual services
pnpm run frontend:dev      # Start React development server
pnpm run backend:dev       # Start Rust backend only

# Build for production
pnpm run build             # Build both React frontend and backend
```

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
# Start React development server (with hot reload)
pnpm run frontend:dev

# Build React application for production
pnpm run frontend:build

# Serve built frontend files
pnpm run frontend:serve
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
# Connect to SQLite database
sqlite3 marketplace.db

# View tables
.tables

# View schema
.schema

# Reset database (delete the file and run migrations)
rm marketplace.db
sqlx migrate run
```

## Project Architecture

### Package Structure (pnpm workspace)
```
pluginmarket/
├── pnpm-workspace.yaml          # pnpm workspace configuration
├── package.json                 # Root package with dev scripts
├── frontend/                    # React TypeScript frontend
│   ├── package.json            # Frontend dependencies
│   ├── src/                    # React source code
│   ├── public/                 # Public assets
│   ├── build/                  # Built frontend (generated)
│   ├── tailwind.config.js      # Tailwind CSS configuration
│   └── tsconfig.json           # TypeScript configuration
├── server/                      # Backend package
│   ├── package.json            # Backend npm scripts
│   └── src/                    # Rust source code
└── proxy_server.py             # Legacy development proxy (optional)
```

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

### Frontend Structure (`frontend/src/`)
- `App.tsx` - Main React application component with routing
- `pages/` - React page components (MainPage, AdminPage)
- `components/` - Reusable React components (Header, PluginGrid, Modals, etc.)
- `contexts/` - React contexts (AuthContext for authentication state)
- `services/` - API service layer for backend communication
- `config/` - TypeScript types and configuration
- `index.css` - Tailwind CSS with custom styles

### Frontend Dependencies (managed by pnpm)
- `react` & `react-dom` - React framework
- `react-router-dom` - Client-side routing
- `typescript` - TypeScript support
- `tailwindcss` - Utility-first CSS framework
- `@fortawesome/fontawesome-free` - Icon library
- `@tailwindcss/typography` - Typography plugin

### Development Tools
- `react-scripts` - Create React App build tools
- `concurrently` - Run multiple npm scripts simultaneously

## Configuration

### Environment Variables (`server/.env`)
```bash
# Database
DATABASE_URL=sqlite://marketplace.db
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

### Admin
- `GET /api/v1/admin/dashboard` - Dashboard statistics
- `GET /api/v1/admin/users` - User management
- `GET /api/v1/admin/plugins` - Plugin management
- `POST /api/v1/admin/sql/execute` - Execute SQL queries

### Health & Monitoring
- `GET /api/v1/health` - Health check endpoint
- `GET /api/v1/metrics` - Application metrics

## Database Schema

Main tables:
- `users` - User accounts with email verification
- `plugins` - Plugin metadata and file information
- `ratings` - Plugin ratings and reviews
- `user_login_activities` - Login tracking for admin

**Note**: The database has been migrated from PostgreSQL to SQLite for easier deployment and reduced complexity.

## Testing

### Manual Testing
Access application:
- Frontend: http://localhost:3001 (React development server)
- Admin panel: http://localhost:3001/admin
- Backend API: http://localhost:3000/api/v1/health

### Development Workflow
1. Install dependencies: `pnpm install`
2. Set up environment variables in `server/.env` (copy from `.env.example`)
3. Run database migrations: `pnpm run db:migrate`
4. Start all services: `pnpm run dev` (starts React frontend on port 3001 and Rust backend on port 3000)
5. Access frontend at http://localhost:3001

### Alternative Development Workflow
1. Start backend: `pnpm run backend:dev` (runs on port 3000)
2. Start frontend: `pnpm run frontend:dev` (runs on port 3001)
3. The React app will make API requests to http://localhost:3000/api/v1

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

### Frontend (managed by pnpm)
- `react` & `react-dom` - React framework for UI development
- `react-router-dom` - Client-side routing for Single Page Application
- `typescript` - Static typing for better development experience
- `tailwindcss` - Utility-first CSS framework
- `@fortawesome/fontawesome-free` - Icon library
- `@tailwindcss/typography` - Typography plugin for better text styling

### Development Tools
- `react-scripts` - Create React App toolchain for building and bundling
- `concurrently` - Run multiple npm scripts simultaneously
- `pnpm` - Fast, disk space efficient package manager

## Production Deployment

For production deployment:
1. Set strong JWT_SECRET in environment
2. Ensure SQLite database file has proper permissions and backup
3. Set up SMTP for email verification
4. Use reverse proxy (Nginx) for static file serving
5. Enable HTTPS/SSL certificates
6. Set up proper backup procedures for SQLite database file and uploads