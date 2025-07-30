#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
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

# Function to generate random password
generate_password() {
    openssl rand -base64 32 | tr -d "=+/" | cut -c1-25
}

# Function to generate JWT secret
generate_jwt_secret() {
    openssl rand -base64 64 | tr -d "=+/" | cut -c1-64
}

# Function to validate email format
validate_email() {
    if [[ $1 =~ ^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$ ]]; then
        return 0
    else
        return 1
    fi
}

# Function to validate port number
validate_port() {
    if [[ $1 =~ ^[0-9]+$ ]] && [ $1 -ge 1024 ] && [ $1 -le 65535 ]; then
        return 0
    else
        return 1
    fi
}

echo ""
echo "=============================================="
echo "  Plugin Marketplace Installation Script"
echo "=============================================="
echo ""

print_info "This script will help you configure and deploy the Plugin Marketplace application."
echo ""

# Check if docker and docker-compose are installed
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker first."
    exit 1
fi

if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    print_error "Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

print_success "Docker and Docker Compose are installed."
echo ""

# Configuration variables
DB_PASSWORD=""
JWT_SECRET=""
API_PORT=""
PROXY_PORT=""
DB_PORT=""
SMTP_ENABLED=""
SMTP_HOST=""
SMTP_PORT=""
SMTP_USERNAME=""
SMTP_PASSWORD=""
SMTP_FROM_ADDRESS=""
SMTP_FROM_NAME=""

# Database configuration
print_info "=== Database Configuration ==="
echo ""

read -p "Enter database name (default: marketplace): " DB_NAME
DB_NAME=${DB_NAME:-marketplace}

read -p "Enter database username (default: postgres): " DB_USER
DB_USER=${DB_USER:-postgres}

while true; do
    read -p "Generate random database password? [Y/n]: " generate_db_pass
    generate_db_pass=${generate_db_pass:-Y}
    
    if [[ $generate_db_pass =~ ^[Yy]$ ]]; then
        DB_PASSWORD=$(generate_password)
        print_success "Generated database password: $DB_PASSWORD"
        break
    elif [[ $generate_db_pass =~ ^[Nn]$ ]]; then
        while true; do
            read -s -p "Enter database password: " DB_PASSWORD
            echo ""
            if [ ${#DB_PASSWORD} -ge 8 ]; then
                break
            else
                print_error "Password must be at least 8 characters long."
            fi
        done
        break
    else
        print_error "Please enter Y or N."
    fi
done

echo ""

# Port configuration
print_info "=== Port Configuration ==="
echo ""

while true; do
    read -p "Enter API server port (default: 3000): " API_PORT
    API_PORT=${API_PORT:-3000}
    
    if validate_port $API_PORT; then
        break
    else
        print_error "Invalid port number. Please enter a port between 1024-65535."
    fi
done

while true; do
    read -p "Enter proxy server port (default: 8080): " PROXY_PORT
    PROXY_PORT=${PROXY_PORT:-8080}
    
    if validate_port $PROXY_PORT && [ $PROXY_PORT != $API_PORT ]; then
        break
    else
        if [ $PROXY_PORT == $API_PORT ]; then
            print_error "Proxy port cannot be the same as API port."
        else
            print_error "Invalid port number. Please enter a port between 1024-65535."
        fi
    fi
done

while true; do
    read -p "Enter database port (default: 5432): " DB_PORT
    DB_PORT=${DB_PORT:-5432}
    
    if validate_port $DB_PORT && [ $DB_PORT != $API_PORT ] && [ $DB_PORT != $PROXY_PORT ]; then
        break
    else
        if [ $DB_PORT == $API_PORT ] || [ $DB_PORT == $PROXY_PORT ]; then
            print_error "Database port cannot be the same as API or proxy port."
        else
            print_error "Invalid port number. Please enter a port between 1024-65535."
        fi
    fi
done

echo ""

# JWT Secret
print_info "=== Security Configuration ==="
echo ""

while true; do
    read -p "Generate random JWT secret? [Y/n]: " generate_jwt
    generate_jwt=${generate_jwt:-Y}
    
    if [[ $generate_jwt =~ ^[Yy]$ ]]; then
        JWT_SECRET=$(generate_jwt_secret)
        print_success "Generated JWT secret."
        break
    elif [[ $generate_jwt =~ ^[Nn]$ ]]; then
        while true; do
            read -s -p "Enter JWT secret (minimum 32 characters): " JWT_SECRET
            echo ""
            if [ ${#JWT_SECRET} -ge 32 ]; then
                break
            else
                print_error "JWT secret must be at least 32 characters long."
            fi
        done
        break
    else
        print_error "Please enter Y or N."
    fi
done

echo ""

# SMTP Configuration
print_info "=== Email (SMTP) Configuration ==="
echo ""

while true; do
    read -p "Enable email notifications? [y/N]: " SMTP_ENABLED
    SMTP_ENABLED=${SMTP_ENABLED:-N}
    
    if [[ $SMTP_ENABLED =~ ^[Yy]$ ]]; then
        SMTP_ENABLED="true"
        
        read -p "Enter SMTP server host (e.g., smtp.gmail.com): " SMTP_HOST
        while [ -z "$SMTP_HOST" ]; do
            print_error "SMTP host cannot be empty."
            read -p "Enter SMTP server host: " SMTP_HOST
        done
        
        while true; do
            read -p "Enter SMTP server port (default: 587): " SMTP_PORT
            SMTP_PORT=${SMTP_PORT:-587}
            
            if [[ $SMTP_PORT =~ ^[0-9]+$ ]] && [ $SMTP_PORT -ge 1 ] && [ $SMTP_PORT -le 65535 ]; then
                break
            else
                print_error "Invalid port number."
            fi
        done
        
        while true; do
            read -p "Enter SMTP username (email address): " SMTP_USERNAME
            if validate_email "$SMTP_USERNAME"; then
                break
            else
                print_error "Invalid email format."
            fi
        done
        
        read -s -p "Enter SMTP password: " SMTP_PASSWORD
        echo ""
        while [ -z "$SMTP_PASSWORD" ]; do
            print_error "SMTP password cannot be empty."
            read -s -p "Enter SMTP password: " SMTP_PASSWORD
            echo ""
        done
        
        read -p "Enter 'From' email address (default: $SMTP_USERNAME): " SMTP_FROM_ADDRESS
        SMTP_FROM_ADDRESS=${SMTP_FROM_ADDRESS:-$SMTP_USERNAME}
        
        read -p "Enter 'From' name (default: Plugin Marketplace): " SMTP_FROM_NAME
        SMTP_FROM_NAME=${SMTP_FROM_NAME:-"Plugin Marketplace"}
        
        break
    elif [[ $SMTP_ENABLED =~ ^[Nn]$ ]]; then
        SMTP_ENABLED="false"
        break
    else
        print_error "Please enter Y or N."
    fi
done

echo ""

# Summary
print_info "=== Configuration Summary ==="
echo ""
echo "Database Name: $DB_NAME"
echo "Database User: $DB_USER"
echo "Database Port: $DB_PORT"
echo "API Port: $API_PORT"
echo "Proxy Port: $PROXY_PORT"
echo "SMTP Enabled: $SMTP_ENABLED"
if [ "$SMTP_ENABLED" = "true" ]; then
    echo "SMTP Host: $SMTP_HOST"
    echo "SMTP Port: $SMTP_PORT"
    echo "SMTP Username: $SMTP_USERNAME"
    echo "From Address: $SMTP_FROM_ADDRESS"
    echo "From Name: $SMTP_FROM_NAME"
fi
echo ""

read -p "Continue with these settings? [Y/n]: " confirm
confirm=${confirm:-Y}

if [[ ! $confirm =~ ^[Yy]$ ]]; then
    print_info "Installation cancelled."
    exit 0
fi

# Generate docker-compose.yml
print_info "Generating docker-compose.yml..."

cat > docker-compose.yml << EOF
version: '3.8'

services:
  postgres:
    image: postgres:15
    container_name: pluginmarket-db
    env_file:
      - .env
    environment:
      POSTGRES_DB: \${POSTGRES_DB}
      POSTGRES_USER: \${POSTGRES_USER}
      POSTGRES_PASSWORD: \${POSTGRES_PASSWORD}
    ports:
      - "\${DB_PORT}:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./server/migrations:/docker-entrypoint-initdb.d
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U \${POSTGRES_USER} -d \${POSTGRES_DB}"]
      interval: 10s
      timeout: 5s
      retries: 5

  backend:
    build:
      context: .
      dockerfile: Dockerfile.backend
    container_name: pluginmarket-backend
    env_file:
      - .env
    environment:
      DATABASE_URL: \${DATABASE_URL}
      JWT_SECRET: \${JWT_SECRET}
      SMTP_ENABLED: \${SMTP_ENABLED}
      SMTP_HOST: \${SMTP_HOST}
      SMTP_PORT: \${SMTP_PORT}
      SMTP_USERNAME: \${SMTP_USERNAME}
      SMTP_PASSWORD: \${SMTP_PASSWORD}
      SMTP_FROM_ADDRESS: \${SMTP_FROM_ADDRESS}
      SMTP_FROM_NAME: \${SMTP_FROM_NAME}
      SERVER_HOST: "0.0.0.0"
      SERVER_PORT: "3000"
      STORAGE_UPLOAD_PATH: "/app/uploads"
      RUST_LOG: "info"
    ports:
      - "\${API_PORT}:3000"
    volumes:
      - ./uploads:/app/uploads:rw
      - ./server/config:/app/config:ro
    depends_on:
      postgres:
        condition: service_healthy
    restart: unless-stopped

  proxy:
    image: python:3.11-slim
    container_name: pluginmarket-proxy
    working_dir: /app
    command: ["python3", "proxy_server.py"]
    env_file:
      - .env
    environment:
      BACKEND_URL: \${BACKEND_URL}
    ports:
      - "\${PROXY_PORT}:8080"
    volumes:
      - ./proxy_server.py:/app/proxy_server.py:ro
      - ./index.html:/app/index.html:ro
      - ./admin.html:/app/admin.html:ro
      - ./admin.js:/app/admin.js:ro
      - ./test-admin.html:/app/test-admin.html:ro
      - ./test-frontend.html:/app/test-frontend.html:ro
      - ./test.html:/app/test.html:ro
      - ./upload-test.html:/app/upload-test.html:ro
      - ./proxy-test.html:/app/proxy-test.html:ro
      - ./uploads:/app/uploads:ro
    depends_on:
      - backend
    restart: unless-stopped

volumes:
  postgres_data:
EOF

# Create backend Dockerfile
print_info "Creating backend Dockerfile..."

cat > Dockerfile.backend << EOF
FROM rust:latest as builder

WORKDIR /app
COPY server/Cargo.toml server/Cargo.lock ./

# Build dependencies first for better caching and reduced memory usage
RUN cargo fetch

COPY server/src ./src
COPY server/migrations ./migrations

# Build with reduced parallelism to avoid memory issues
ENV CARGO_BUILD_JOBS=1
# Temporarily reduce optimization level to save memory during compilation
RUN sed -i 's/lto = true/lto = false/' Cargo.toml && \\
    sed -i 's/opt-level = 3/opt-level = 2/' Cargo.toml && \\
    cargo build --release

FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \\
    ca-certificates \\
    && rm -rf /var/lib/apt/lists/* \\
    && groupadd -r appuser \\
    && useradd -r -g appuser appuser

WORKDIR /app

COPY --from=builder /app/target/release/server ./
COPY server/config ./config/

RUN mkdir -p uploads \\
    && chown -R appuser:appuser /app \\
    && chmod -R 755 /app

USER appuser

EXPOSE 3000

CMD ["./server"]
EOF

# Proxy service will use pre-built Python image directly
print_info "Proxy service configured to use python:3.11-slim image..."

# Update proxy_server.py to use environment variable
print_info "Updating proxy server configuration..."

cat > proxy_server.py << 'EOF'
#!/usr/bin/env python3
"""
Simple HTTP server with API proxy to work around CORS issues
"""
import http.server
import socketserver
import urllib.request
import urllib.parse
import json
import os

PORT = 8080
BACKEND_URL = os.getenv('BACKEND_URL', 'http://localhost:3000')

class ProxyHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path.startswith('/api/'):
            self.proxy_request()
        else:
            super().do_GET()
    
    def do_POST(self):
        if self.path.startswith('/api/'):
            self.proxy_request()
        else:
            self.send_error(405, "Method not allowed")
    
    def do_PUT(self):
        if self.path.startswith('/api/'):
            self.proxy_request()
        else:
            self.send_error(405, "Method not allowed")
    
    def do_DELETE(self):
        if self.path.startswith('/api/'):
            self.proxy_request()
        else:
            self.send_error(405, "Method not allowed")
    
    def do_OPTIONS(self):
        if self.path.startswith('/api/'):
            self.send_response(200)
            self.send_header('Access-Control-Allow-Origin', '*')
            self.send_header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
            self.send_header('Access-Control-Allow-Headers', 'Content-Type, Authorization')
            self.end_headers()
        else:
            super().do_OPTIONS()
    
    def proxy_request(self):
        try:
            # Build the backend URL
            backend_url = BACKEND_URL + self.path
            
            # Prepare the request
            headers = {}
            for header, value in self.headers.items():
                if header.lower() not in ['host', 'connection']:
                    headers[header] = value
            
            # Get request body if it's a POST request
            content_length = int(self.headers.get('Content-Length', 0))
            post_data = None
            if content_length > 0:
                post_data = self.rfile.read(content_length)
            
            # Make the request to backend
            req = urllib.request.Request(backend_url, data=post_data, headers=headers, method=self.command)
            
            with urllib.request.urlopen(req) as response:
                # Send response
                self.send_response(response.status)
                
                # Copy headers
                for header, value in response.headers.items():
                    if header.lower() not in ['connection', 'transfer-encoding']:
                        self.send_header(header, value)
                
                # Add CORS headers
                self.send_header('Access-Control-Allow-Origin', '*')
                self.send_header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
                self.send_header('Access-Control-Allow-Headers', 'Content-Type, Authorization')
                
                self.end_headers()
                
                # Copy response body
                self.wfile.write(response.read())
                
        except Exception as e:
            print(f"Proxy error: {e}")
            self.send_error(500, f"Proxy error: {str(e)}")

if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    
    with socketserver.TCPServer(("", PORT), ProxyHandler) as httpd:
        print(f"Serving at http://localhost:{PORT}")
        print(f"Proxying /api/* requests to {BACKEND_URL}")
        print("Press Ctrl+C to stop")
        httpd.serve_forever()
EOF

# Create .env file for reference
print_info "Creating .env file for reference..."

cat > .env << EOF
# Database Configuration
POSTGRES_DB=${DB_NAME}
POSTGRES_USER=${DB_USER}
POSTGRES_PASSWORD=${DB_PASSWORD}
DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@postgres:5432/${DB_NAME}

# Security
JWT_SECRET=${JWT_SECRET}

# Ports
API_PORT=${API_PORT}
PROXY_PORT=${PROXY_PORT}
DB_PORT=${DB_PORT}

# SMTP Configuration
SMTP_ENABLED=${SMTP_ENABLED}
SMTP_HOST=${SMTP_HOST}
SMTP_PORT=${SMTP_PORT}
SMTP_USERNAME=${SMTP_USERNAME}
SMTP_PASSWORD=${SMTP_PASSWORD}
SMTP_FROM_ADDRESS=${SMTP_FROM_ADDRESS}
SMTP_FROM_NAME=${SMTP_FROM_NAME}

# Backend URL for proxy
BACKEND_URL=http://backend:3000
EOF

# Create uploads directory with proper permissions
print_info "Creating uploads directory..."
mkdir -p uploads
chmod 755 uploads

print_success "Configuration files generated successfully!"
echo ""
print_info "=== Next Steps ==="
echo ""
echo "1. Build backend and start all services:"
echo "   docker-compose up --build -d"
echo ""
echo "2. Access the application:"
echo "   - Frontend: http://localhost:${PROXY_PORT}"
echo "   - API: http://localhost:${API_PORT}"
echo "   - Database: localhost:${DB_PORT}"
echo ""
echo "3. To stop the services:"
echo "   docker-compose down"
echo ""
echo "4. To view logs:"
echo "   docker-compose logs -f"
echo ""

read -p "Start the services now? [Y/n]: " start_now
start_now=${start_now:-Y}

if [[ $start_now =~ ^[Yy]$ ]]; then
    print_info "Starting services..."
    docker-compose up --build -d
    
    print_success "Services started successfully!"
    echo ""
    print_info "Waiting for services to be ready..."
    sleep 10
    
    echo ""
    print_success "Plugin Marketplace is now running!"
    echo ""
    echo "Frontend: http://localhost:${PROXY_PORT}"
    echo "API: http://localhost:${API_PORT}"
    echo ""
    print_info "Check service status with: docker-compose ps"
    print_info "View logs with: docker-compose logs -f"
else
    print_info "Services not started. Run 'docker-compose up --build -d' when ready."
fi

echo ""
print_success "Installation completed!"