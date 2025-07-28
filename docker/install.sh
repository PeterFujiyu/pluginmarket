#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DATA_DIR="./data"
CONFIG_FILE="$DATA_DIR/config.env"
ENV_EXAMPLE="./server/.env.example"

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Banner
echo -e "${BLUE}"
cat << 'EOF'
╔══════════════════════════════════════════════════════════════════╗
║              GeekTools Plugin Marketplace Installer             ║
║                        Docker Deployment                        ║
╚══════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Check if Docker is installed
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        print_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    print_success "Docker and Docker Compose are available"
}

# Create data directory
create_data_dir() {
    print_status "Creating data directory..."
    mkdir -p "$DATA_DIR"
    print_success "Data directory created: $DATA_DIR"
}

# Generate random password
generate_password() {
    if command -v openssl &> /dev/null; then
        openssl rand -base64 32 | tr -d "=+/" | cut -c1-32
    else
        date +%s | sha256sum | base64 | head -c 32
    fi
}

# Interactive configuration
interactive_config() {
    print_status "Setting up configuration..."
    
    # Check if config already exists
    if [ -f "$CONFIG_FILE" ]; then
        echo ""
        read -p "Configuration file already exists. Do you want to recreate it? [y/N]: " recreate
        if [[ ! "$recreate" =~ ^[Yy]$ ]]; then
            print_status "Using existing configuration file"
            return
        fi
    fi
    
    # Load defaults from .env.example
    if [ ! -f "$ENV_EXAMPLE" ]; then
        print_error ".env.example file not found. Please ensure you're running this from the project root."
        exit 1
    fi
    
    echo ""
    print_status "Please provide configuration values (press Enter to use defaults):"
    echo ""
    
    # Database configuration
    echo -e "${YELLOW}Database Configuration:${NC}"
    read -p "PostgreSQL password [auto-generated]: " db_password
    if [ -z "$db_password" ]; then
        db_password=$(generate_password)
        print_status "Generated database password: $db_password"
    fi
    
    # Server configuration
    echo ""
    echo -e "${YELLOW}Server Configuration:${NC}"
    read -p "Server host [0.0.0.0]: " server_host
    server_host=${server_host:-0.0.0.0}
    
    read -p "Server port [3000]: " server_port
    server_port=${server_port:-3000}
    
    # JWT configuration
    echo ""
    echo -e "${YELLOW}Security Configuration:${NC}"
    read -p "JWT secret key [auto-generated]: " jwt_secret
    if [ -z "$jwt_secret" ]; then
        jwt_secret=$(generate_password)
        print_status "Generated JWT secret: ${jwt_secret:0:8}..."
    fi
    
    read -p "JWT access token expires in seconds [3600]: " jwt_access_expires
    jwt_access_expires=${jwt_access_expires:-3600}
    
    read -p "JWT refresh token expires in seconds [604800]: " jwt_refresh_expires
    jwt_refresh_expires=${jwt_refresh_expires:-604800}
    
    # Storage configuration
    echo ""
    echo -e "${YELLOW}Storage Configuration:${NC}"
    read -p "Max file size in bytes [104857600]: " max_file_size
    max_file_size=${max_file_size:-104857600}
    
    # SMTP configuration
    echo ""
    echo -e "${YELLOW}Email Configuration (optional):${NC}"
    read -p "Enable SMTP? [y/N]: " enable_smtp
    
    smtp_enabled="false"
    smtp_host=""
    smtp_port=""
    smtp_username=""
    smtp_password=""
    smtp_from_address=""
    smtp_from_name=""
    
    if [[ "$enable_smtp" =~ ^[Yy]$ ]]; then
        smtp_enabled="true"
        read -p "SMTP host: " smtp_host
        read -p "SMTP port [587]: " smtp_port
        smtp_port=${smtp_port:-587}
        read -p "SMTP username: " smtp_username
        read -s -p "SMTP password: " smtp_password
        echo ""
        read -p "From email address: " smtp_from_address
        read -p "From name [GeekTools Plugin Marketplace]: " smtp_from_name
        smtp_from_name=${smtp_from_name:-"GeekTools Plugin Marketplace"}
    fi
    
    # Write configuration file
    print_status "Writing configuration file..."
    
    cat > "$CONFIG_FILE" << EOF
# GeekTools Plugin Marketplace Configuration
# Generated on $(date)

# Server Configuration
SERVER_HOST=$server_host
SERVER_PORT=$server_port

# Database Configuration
DATABASE_URL=postgres://postgres:$db_password@postgres:5432/marketplace
DATABASE_MAX_CONNECTIONS=10

# JWT Configuration
JWT_SECRET=$jwt_secret
JWT_ACCESS_TOKEN_EXPIRES_IN=$jwt_access_expires
JWT_REFRESH_TOKEN_EXPIRES_IN=$jwt_refresh_expires

# Storage Configuration
STORAGE_UPLOAD_PATH=./uploads
STORAGE_MAX_FILE_SIZE=$max_file_size
STORAGE_USE_CDN=false
STORAGE_CDN_BASE_URL=https://cdn.geektools.dev

# SMTP Configuration
SMTP_ENABLED=$smtp_enabled
SMTP_HOST=$smtp_host
SMTP_PORT=$smtp_port
SMTP_USERNAME=$smtp_username
SMTP_PASSWORD=$smtp_password
SMTP_FROM_ADDRESS=$smtp_from_address
SMTP_FROM_NAME=$smtp_from_name

# API Configuration
API_BASE_URL=http://localhost:$server_port/api/v1

# Logging
RUST_LOG=info
RUST_BACKTRACE=1

# Docker environment variables
POSTGRES_PASSWORD=$db_password
APP_PORT=$server_port
PROXY_PORT=8080
EOF
    
    print_success "Configuration file created: $CONFIG_FILE"
}

# Create .env file for docker-compose
create_docker_env() {
    print_status "Creating Docker environment file..."
    
    # Extract values from config.env
    if [ -f "$CONFIG_FILE" ]; then
        POSTGRES_PASSWORD=$(grep "^POSTGRES_PASSWORD=" "$CONFIG_FILE" | cut -d'=' -f2)
        APP_PORT=$(grep "^APP_PORT=" "$CONFIG_FILE" | cut -d'=' -f2)
        PROXY_PORT=$(grep "^PROXY_PORT=" "$CONFIG_FILE" | cut -d'=' -f2)
    else
        print_error "Configuration file not found!"
        exit 1
    fi
    
    cat > .env << EOF
# Docker Compose Environment Variables
POSTGRES_PASSWORD=$POSTGRES_PASSWORD
APP_PORT=$APP_PORT
PROXY_PORT=$PROXY_PORT
POSTGRES_PORT=5432
EOF
    
    print_success "Docker environment file created"
}

# Build and start services
start_services() {
    print_status "Building and starting services..."
    
    # Use docker compose if available, otherwise docker-compose
    if docker compose version &> /dev/null; then
        COMPOSE_CMD="docker compose"
    else
        COMPOSE_CMD="docker-compose"
    fi
    
    print_status "Building Docker images..."
    $COMPOSE_CMD build
    
    print_status "Starting services..."
    $COMPOSE_CMD up -d
    
    print_success "Services started successfully!"
}

# Show status and access information
show_status() {
    echo ""
    print_success "GeekTools Plugin Marketplace is now running!"
    echo ""
    echo -e "${BLUE}Access Information:${NC}"
    
    APP_PORT=$(grep "^APP_PORT=" "$CONFIG_FILE" | cut -d'=' -f2)
    PROXY_PORT=$(grep "^PROXY_PORT=" "$CONFIG_FILE" | cut -d'=' -f2)
    
    echo "  Frontend:    http://localhost:$PROXY_PORT"
    echo "  Backend API: http://localhost:$APP_PORT/api/v1"
    echo "  Admin Panel: http://localhost:$PROXY_PORT/admin.html"
    echo ""
    echo -e "${BLUE}Management Commands:${NC}"
    echo "  View logs:   docker-compose logs -f"
    echo "  Stop:        docker-compose down"
    echo "  Restart:     docker-compose restart"
    echo "  Update:      docker-compose pull && docker-compose up -d"
    echo ""
    echo -e "${BLUE}Configuration:${NC}"
    echo "  Config file: $CONFIG_FILE"
    echo "  Data dir:    $DATA_DIR"
    echo ""
    print_status "Installation completed successfully!"
}

# Main installation process
main() {
    echo "Starting installation process..."
    echo ""
    
    check_docker
    create_data_dir
    interactive_config
    create_docker_env
    start_services
    show_status
}

# Run main function
main

echo ""
print_success "Installation complete! Enjoy using GeekTools Plugin Marketplace!"