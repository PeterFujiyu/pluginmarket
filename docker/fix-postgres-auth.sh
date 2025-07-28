#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

print_status "Fixing PostgreSQL authentication issue..."

# Check if .env file exists
if [ ! -f ".env" ]; then
    print_error ".env file not found. Please run ./docker/install.sh first."
    exit 1
fi

# Check if data/config.env exists
if [ ! -f "data/config.env" ]; then
    print_error "data/config.env file not found. Please run ./docker/install.sh first."
    exit 1
fi

# Extract password from config file
DB_PASSWORD=$(grep "^POSTGRES_PASSWORD=" "data/config.env" | cut -d'=' -f2)

if [ -z "$DB_PASSWORD" ]; then
    print_error "Could not find POSTGRES_PASSWORD in data/config.env"
    exit 1
fi

print_status "Found database password in config file"

# Update .env file to ensure password matches
print_status "Updating .env file with correct password..."

# Create a new .env file with the correct password
cat > .env << EOF
# Docker Compose Environment Variables
POSTGRES_PASSWORD=$DB_PASSWORD
APP_PORT=$(grep "^APP_PORT=" .env | cut -d'=' -f2 || echo "3000")
PROXY_PORT=$(grep "^PROXY_PORT=" .env | cut -d'=' -f2 || echo "8080")
POSTGRES_PORT=5432
EOF

print_success ".env file updated"

# Stop containers
print_status "Stopping containers..."
if docker compose version &> /dev/null; then
    COMPOSE_CMD="docker compose"
else
    COMPOSE_CMD="docker-compose"
fi

$COMPOSE_CMD down

# Remove postgres volume to reset database
print_warning "Removing PostgreSQL data volume to reset database..."
docker volume rm pluginmarket_postgres_data 2>/dev/null || true

# Start containers again
print_status "Starting containers with correct configuration..."
$COMPOSE_CMD up -d

print_success "Fix applied! Containers are starting up..."
print_status "Monitor logs with: $COMPOSE_CMD logs -f"

# Wait a moment and check status
sleep 5
print_status "Container status:"
$COMPOSE_CMD ps