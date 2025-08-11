# Multi-stage build for GeekTools Plugin Marketplace
FROM debian:bookworm-slim AS build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    curl \
    build-essential \
    ca-certificates \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path

# Install Node.js and pnpm for frontend dependencies
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g pnpm

# Set working directory
WORKDIR /app

# Copy workspace files
COPY pnpm-workspace.yaml package.json pnpm-lock.yaml ./
COPY frontend/package.json ./frontend/

# Install frontend dependencies
RUN pnpm install

# Copy server source
COPY server ./server

# Build Rust application
WORKDIR /app/server
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    ca-certificates \
    python3 \
    libssl3 \
    postgresql-client \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r app && useradd -r -g app app

# Set working directory
WORKDIR /app

# Copy built application
COPY --from=build /app/server/target/release/server /app/
COPY --from=build /app/server/migrations /app/migrations/
COPY --from=build /app/frontend /app/frontend/
COPY --from=build /app/node_modules /app/node_modules/
COPY proxy_server.py /app/

# Create uploads directory
RUN mkdir -p uploads && chown -R app:app /app

# Switch to app user
USER app

# Expose port 3000 for backend API
EXPOSE 3000

# Health check for the backend service
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:3000/api/v1/health || exit 1

# Default command - just run the Rust server
CMD ["./server"]