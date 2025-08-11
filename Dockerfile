# Multi-stage Dockerfile for Rust development and production

# Development stage
FROM rust:latest AS development

# Install additional tools for development
RUN apt-get update && apt-get install -y \
    git \
    # vim \
    # curl \
    && rm -rf /var/lib/apt/lists/*

# Install Rust components
RUN rustup component add rustfmt clippy

# Set working directory
WORKDIR /app

# Create cargo cache directory with proper permissions
RUN mkdir -p /usr/local/cargo/registry \
    && chmod -R 777 /usr/local/cargo

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock* ./

# Create dummy src to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless Cargo.toml changes)
RUN cargo build --release
RUN cargo build

# Remove dummy src
RUN rm -rf src

# Default command for development
CMD ["bash"]

# Builder stage
FROM rust:latest AS builder

WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock* ./

# Create dummy src to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY src/ ./src/
# COPY tests/ ./tests/

# Build the application
RUN cargo build --release

# Production stage
FROM debian:bookworm-slim AS production

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Copy binary from builder stage
COPY --from=builder /app/target/release/doit /usr/local/bin/doit

# Change ownership
RUN chown appuser:appuser /usr/local/bin/doit

# Switch to app user
USER appuser

# Set entrypoint
ENTRYPOINT ["pmon"]
