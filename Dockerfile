# Build stage
FROM rust:1.75-slim as builder

WORKDIR /usr/src/app
COPY . .

# Build the application with release profile
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install necessary SSL certificates for WHOIS queries
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /usr/src/app/target/release/domain_exporter /app/domain_exporter

# Expose the default port
EXPOSE 9222

# Run the binary
ENTRYPOINT ["/app/domain_exporter"] 