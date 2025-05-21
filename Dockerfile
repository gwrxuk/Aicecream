# Build stage
FROM rust:1.75-slim-bullseye as builder

WORKDIR /usr/src/galato

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/galato/target/release/galato .

# Copy configuration
COPY --from=builder /usr/src/galato/config.yaml /etc/galato/config.yaml

# Set environment variables
ENV CONFIG_PATH=/etc/galato/config.yaml
ENV RUST_LOG=info

# Expose the application port
EXPOSE 8080

# Run the application
CMD ["galato"] 