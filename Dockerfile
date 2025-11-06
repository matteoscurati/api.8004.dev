# Multi-stage build for smaller image size
FROM rustlang/rust:nightly-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:trixie-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/api_8004_dev /usr/local/bin/api_8004_dev

# Copy migrations
COPY migrations ./migrations

# Create non-root user
RUN useradd -m -u 1000 indexer && \
    chown -R indexer:indexer /app

USER indexer

EXPOSE 8080

CMD ["api_8004_dev"]
