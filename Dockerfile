# Multi-stage build - Build stage
FROM rust:1.82 as builder

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs and empty benchmark to pre-build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN mkdir benches && echo "fn main() {}" > benches/performance.rs
RUN cargo build --release
RUN rm src/main.rs benches/performance.rs

# Copy source code and benches
COPY src ./src
COPY benches ./benches

# Build application
RUN cargo build --release

# Runtime stage - Use smaller base image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false kline

# Copy compiled binary
COPY --from=builder /app/target/release/k-line /usr/local/bin/k-line

# Copy test client (optional)
COPY websocket_test.html /app/websocket_test.html

# Set user
USER kline

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/tokens || exit 1

# Start command
CMD ["k-line"] 