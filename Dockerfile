# Multi-stage Docker build for Proxy Re-encryption Enclave Application

# -----------------------------------------------------------------------------
# Builder stage: Compile the Rust application
# -----------------------------------------------------------------------------
FROM rust:1.70-slim AS builder

# Install required packages for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src/

# Copy actual source code
COPY src ./src

# Build the application
RUN touch src/main.rs && cargo build --release

# -----------------------------------------------------------------------------
# Runtime stage: Create minimal production image
# -----------------------------------------------------------------------------
FROM debian:bullseye-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Create app directory
WORKDIR /app

# Copy the compiled binary from builder stage
COPY --from=builder /app/target/release/proxy_reencyption_enclave_app /usr/local/bin/

# Change ownership to non-root user
RUN chown appuser:appuser /usr/local/bin/proxy_reencyption_enclave_app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/ || exit 1

# Set default command
CMD ["proxy_reencyption_enclave_app", "client", "--port", "8000", "--cid", "3"]

# -----------------------------------------------------------------------------
# Development stage: For development with hot reload
# -----------------------------------------------------------------------------
FROM rust:1.70-slim AS development

# Install development tools
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-watch for hot reload
RUN cargo install cargo-watch

# Create app directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Expose port
EXPOSE 8000

# Default command for development
CMD ["cargo", "watch", "-x", "run -- client --port 8000 --cid 3"]

# -----------------------------------------------------------------------------
# Test stage: For running tests in container
# -----------------------------------------------------------------------------
FROM rust:1.70-slim AS test

# Install test dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Copy test files
COPY tests ./tests

# Run tests
RUN cargo test --verbose

# -----------------------------------------------------------------------------
# AWS Nitro Enclave stage: Specialized for enclave deployment
# -----------------------------------------------------------------------------
FROM rust:1.70-slim AS enclave-builder

# Install AWS Nitro CLI and dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    musl-tools \
    wget \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Install AWS Nitro CLI (adjust version as needed)
RUN wget https://aws-nitro-enclaves-cli.s3.amazonaws.com/aws-nitro-enclaves-cli.x86_64.zip \
    && unzip aws-nitro-enclaves-cli.x86_64.zip \
    && mv aws-nitro-enclaves-cli /usr/local/bin/ \
    && chmod +x /usr/local/bin/aws-nitro-enclaves-cli

# Set target for musl (required for Nitro Enclaves)
ENV RUST_TARGET=x86_64-unknown-linux-musl
RUN rustup target add $RUST_TARGET

# Create app directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for musl target
RUN cargo build --release --target $RUST_TARGET

# Copy enclave-specific files
COPY docker-enclave ./docker-enclave

# Build enclave image
RUN aws-nitro-enclaves-cli build-enclave \
    --docker-dir ./docker-enclave \
    --docker-uri proxy-reencryption-enclave \
    --output-file enclave.eif

# -----------------------------------------------------------------------------
# Usage examples:
# -----------------------------------------------------------------------------
# Production build:
# docker build --target runtime -t proxy-reencryption:latest .
#
# Development with hot reload:
# docker build --target development -t proxy-reencryption:dev .
#
# Run tests:
# docker build --target test -t proxy-reencryption:test .
#
# AWS Nitro Enclave build:
# docker build --target enclave-builder -t proxy-reencryption:enclave .
#
# Run production container:
# docker run -p 8000:8000 --name proxy-reencryption proxy-reencryption:latest
#
# Run with custom configuration:
# docker run -p 8000:8000 \
#   -e RUST_LOG=debug \
#   --name proxy-reencryption \
#   proxy-reencryption:latest \
#   proxy_reencyption_enclave_app client --port 8000 --cid 16