FROM rust:1.75-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files first (for better caching)
COPY Cargo.toml ./

# Copy source code
COPY src/ ./src/

# Build the application (Cargo.lock will be generated if missing)
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/fks_meta /app/fks_meta

# Copy entrypoint
COPY entrypoint.sh ./
RUN chmod +x entrypoint.sh

# Environment variables
ENV SERVICE_NAME=fks_meta \
    SERVICE_PORT=8005 \
    RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8005/health || exit 1

# Expose service port
EXPOSE 8005

# Create non-root user
RUN useradd -u 1000 -m appuser && chown -R appuser /app
USER appuser

# Use entrypoint script
ENTRYPOINT ["./entrypoint.sh"]

