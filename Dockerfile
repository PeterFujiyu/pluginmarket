# Multi-stage build for GeekTools Plugin Marketplace  
FROM rustlang/rust:nightly-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# We'll handle migrations through the application itself
# No need to install sqlx-cli separately

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY server/Cargo.toml server/Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY server/src ./src
COPY server/migrations ./migrations

# Build the application
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    libssl3 \
    ca-certificates \
    python3 \
    python3-urllib3 \
    postgresql-client \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app appuser

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/server ./server

# Copy application files
COPY --chown=appuser:appuser server/migrations ./migrations
COPY --chown=appuser:appuser server/config ./config
COPY --chown=appuser:appuser proxy_server.py ./
COPY --chown=appuser:appuser *.html *.js ./

# Copy default environment file
COPY server/.env.example ./.env.example

# Create directories
RUN mkdir -p uploads data && \
    chown -R appuser:appuser uploads data

# Create entrypoint script
RUN echo '#!/bin/bash' > /app/entrypoint.sh && \
    echo 'set -e' >> /app/entrypoint.sh && \
    echo '' >> /app/entrypoint.sh && \
    echo '# Wait for database to be ready' >> /app/entrypoint.sh && \
    echo 'echo "Waiting for database..."' >> /app/entrypoint.sh && \
    echo 'until PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$POSTGRES_HOST" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "\\q" 2>/dev/null; do' >> /app/entrypoint.sh && \
    echo '  echo "Database is unavailable - sleeping"' >> /app/entrypoint.sh && \
    echo '  sleep 1' >> /app/entrypoint.sh && \
    echo 'done' >> /app/entrypoint.sh && \
    echo '' >> /app/entrypoint.sh && \
    echo 'echo "Database is up - executing command"' >> /app/entrypoint.sh && \
    echo '' >> /app/entrypoint.sh && \
    echo '# Use config file if mounted, otherwise use default' >> /app/entrypoint.sh && \
    echo 'if [ -f "/data/config.env" ]; then' >> /app/entrypoint.sh && \
    echo '    echo "Using mounted config file: /data/config.env"' >> /app/entrypoint.sh && \
    echo '    export $(cat /data/config.env | grep -v "^#" | xargs)' >> /app/entrypoint.sh && \
    echo 'else' >> /app/entrypoint.sh && \
    echo '    echo "Using default config file: .env.example"' >> /app/entrypoint.sh && \
    echo '    export $(cat .env.example | grep -v "^#" | xargs)' >> /app/entrypoint.sh && \
    echo 'fi' >> /app/entrypoint.sh && \
    echo '' >> /app/entrypoint.sh && \
    echo '# Migrations will be handled by the application automatically' >> /app/entrypoint.sh && \
    echo '' >> /app/entrypoint.sh && \
    echo '# Start the application' >> /app/entrypoint.sh && \
    echo 'echo "Starting GeekTools Plugin Marketplace Server..."' >> /app/entrypoint.sh && \
    echo 'exec ./server' >> /app/entrypoint.sh

RUN chmod +x /app/entrypoint.sh && chown appuser:appuser /app/entrypoint.sh

# Switch to app user
USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/v1/health || exit 1

# Set entrypoint
ENTRYPOINT ["/app/entrypoint.sh"]