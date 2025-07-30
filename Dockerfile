# Multi-stage build for the Rust backend
FROM rust:1.75 as rust-builder

WORKDIR /app/server
COPY server/Cargo.toml server/Cargo.lock ./
COPY server/src ./src
COPY server/migrations ./migrations
COPY server/config ./config

# Build the Rust application
RUN cargo build --release

# Final stage
FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    postgresql-client \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install PostgreSQL
RUN apt-get update && apt-get install -y \
    postgresql \
    postgresql-contrib \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Rust binary
COPY --from=rust-builder /app/server/target/release/server ./server/

# Copy server config and migrations
COPY server/config ./server/config/
COPY server/migrations ./server/migrations/

# Copy Python proxy server
COPY proxy_server.py ./

# Copy frontend files
COPY *.html ./
COPY *.js ./
COPY app_uploads/ ./app_uploads/

# Create uploads directory
RUN mkdir -p uploads

# Copy startup script
COPY <<EOF /app/start.sh
#!/bin/bash
set -e

# Initialize PostgreSQL if needed
if [ ! -d "/var/lib/postgresql/data" ]; then
    mkdir -p /var/lib/postgresql/data
    chown postgres:postgres /var/lib/postgresql/data
    su - postgres -c '/usr/lib/postgresql/*/bin/initdb -D /var/lib/postgresql/data'
fi

# Start PostgreSQL
su - postgres -c '/usr/lib/postgresql/*/bin/pg_ctl -D /var/lib/postgresql/data -l /var/lib/postgresql/data/logfile start'

# Wait for PostgreSQL to start
sleep 5

# Create database and user
su - postgres -c 'createdb marketplace' || true
su - postgres -c "psql -c \"CREATE USER postgres WITH PASSWORD 'password';\"" || true
su - postgres -c "psql -c \"GRANT ALL PRIVILEGES ON DATABASE marketplace TO postgres;\"" || true

# Run migrations (you may need to adjust this based on your migration tool)
export DATABASE_URL="postgres://postgres:password@localhost:5432/marketplace"
export JWT_SECRET="your-secret-key-change-this-in-production"

# Start the Rust server in background
cd /app && ./server/server &

# Wait a moment for the server to start
sleep 3

# Start the Python proxy server
cd /app && python3 proxy_server.py &

# Keep the container running
wait
EOF

RUN chmod +x /app/start.sh

# Expose ports
EXPOSE 3000 8080 5432

# Set environment variables
ENV DATABASE_URL="postgres://postgres:password@localhost:5432/marketplace"
ENV JWT_SECRET="your-secret-key-change-this-in-production"

# Start all services
CMD ["/app/start.sh"]