# Domain Exporter

Domain Exporter is a Rust-based tool for monitoring domain expiration. It queries WHOIS information and outputs domain expiration time in Prometheus metrics format.

## Features

- Supports multiple WHOIS servers and date formats
- Built-in caching to avoid frequent queries
- Automatic retry mechanism for reliability
- Prometheus metrics output
- Configurable timeout and cache duration
- Embedded WHOIS server configuration

## Installation

Ensure you have the Rust toolchain installed, then run:

```bash
cargo install --git https://github.com/yourusername/domain_exporter
```

Or build from source:

```bash
git clone https://github.com/yourusername/domain_exporter
cd domain_exporter
cargo build --release
```

## Configuration Options

```bash
domain_exporter [OPTIONS]

OPTIONS:
    --cache-ttl <SECONDS>      Cache duration in seconds [default: 86400]
    --whois-timeout <SECONDS>  WHOIS query timeout in seconds [default: 10]
    --listen-addr <ADDR>       Server listen address [default: 0.0.0.0:9222]
    -h, --help                 Show help information
    -V, --version              Show version information
```

## API Endpoints

### Domain Query
```bash
GET /probe?target=example.com
```

Example response:
```
# HELP domain_expiry_days Days until domain expiry
# TYPE domain_expiry_days gauge
domain_expiry_days{domain="example.com"} 365
# HELP domain_probe_success Displays whether or not the domain probe was successful
# TYPE domain_probe_success gauge
domain_probe_success{domain="example.com"} 1
```

## Error Handling

The service handles the following error scenarios:
- WHOIS query failure
- Date parsing errors
- Query timeout
- Server busy

Errors are automatically retried (up to 3 times) and appropriate error metrics are returned.

## Caching Mechanism

- Successful query results are cached
- Cache duration is configurable (default 24 hours)
- Only valid expiration dates are cached
- Cache is in-memory and cleared on service restart

## Prometheus Integration

Add the following configuration to your Prometheus config file:

```yaml
scrape_configs:
  - job_name: 'domain_expiry'
    metrics_path: /probe
    params:
      module: [http_2xx]
    static_configs:
      - targets:
        - example.com
        - example.org
    relabel_configs:
      - source_labels: [__address__]
        target_label: __param_target
      - source_labels: [__param_target]
        target_label: instance
      - target_label: __address__
        replacement: localhost:9222  # Address of the domain exporter
```

## Development

### Project Structure

```
src/
├── main.rs      # Main program entry
├── cache.rs     # Cache implementation
├── config.rs    # Configuration handling
├── error.rs     # Error definitions
└── whois.rs     # WHOIS query implementation
```

### Build and Test

```bash
# Run tests
cargo test

# Build development version
cargo build

# Build release version
cargo build --release
```

## License

MIT License

## Contributing

Contributions are welcome! Please submit issues and pull requests.

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a pull request

## Docker Usage

### Using Docker Compose

1. Build and start the service:
```bash
docker-compose up -d
```

2. Check the logs:
```bash
docker-compose logs -f
```

3. Stop the service:
```bash
docker-compose down
```

### Using Docker Directly

1. Build the image:
```bash
docker build -t domain-exporter .
```

2. Run the container:
```bash
docker run -d -p 9222:9222 domain-exporter
```

### Docker Environment Variables

You can configure the exporter using environment variables:

```bash
docker run -d \
  -p 9222:9222 \
  -e CACHE_TTL=86400 \
  -e WHOIS_TIMEOUT=10 \
  domain-exporter
```