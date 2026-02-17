# Build stage
FROM rust:1.85-bookworm as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Copy source code
COPY src ./src

# Build the release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install CA certificates for HTTPS upstream connections
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/relay /usr/local/bin/relay

# Copy default config (can be overridden with volume mount)
COPY config.toml /app/config.toml

# Expose the default port from config
EXPOSE 4000

# Run the binary
CMD ["relay"]
