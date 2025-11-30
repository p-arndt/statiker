# API Reference

This document provides detailed information about Statiker's command-line interface and configuration API.

## Command Line Interface

### Usage

```bash
statiker [OPTIONS]
```

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--help` | `-h` | Display help information and exit | - |
| `--config <PATH>` | `-c` | Path to configuration file | `statiker.yaml` |

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CONFIG` | Path to configuration file | `statiker.yaml` |
| `RUST_LOG` | Log level override | Value from config file or `info` |

### Examples

```bash
# Use default config file
statiker

# Specify custom config file
statiker --config /path/to/config.yaml
statiker -c custom.yaml

# Use environment variable
CONFIG=my-config.yaml statiker

# Override log level
RUST_LOG=debug statiker
```

## HTTP API

Statiker serves HTTP requests and supports the following methods and features.

### Supported HTTP Methods

- **GET**: Retrieve files and resources
- **HEAD**: Retrieve headers only (includes `Content-Length`)

Other methods return `405 Method Not Allowed`.

### Response Headers

Statiker automatically sets appropriate headers:

- **Content-Type**: Based on file MIME type
- **Content-Length**: File size (for both GET and HEAD requests)
- **Cache-Control**: For assets when caching is enabled
- **Custom Security Headers**: As configured in `security.headers`
- **CORS Headers**: When CORS is enabled
- **Compression Headers**: When compression is enabled

### Status Codes

| Code | Description |
|------|-------------|
| `200 OK` | Successful request |
| `403 Forbidden` | Path traversal attempt detected |
| `404 Not Found` | File or resource not found |
| `405 Method Not Allowed` | Unsupported HTTP method |
| `429 Too Many Requests` | Rate limit exceeded |
| `500 Internal Server Error` | Server error |

### Path Patterns

Route paths support the following patterns:

- **Exact match**: `/` matches exactly `/`
- **Wildcard**: `/api/*` matches `/api/` and all sub-paths like `/api/users`, `/api/users/123`, etc.

### Proxy Headers

When proxying requests, Statiker:

1. Forwards the original request to the backend
2. Adds configured headers from `add_headers`
3. Replaces `{client_ip}` placeholder with the actual client IP
4. Preserves original request headers (except those explicitly overridden)

### Client IP Detection

The client IP is determined in this order:

1. `X-Forwarded-For` header (first IP in the comma-separated list)
2. Socket address from the connection
3. Fallback to `0.0.0.0` if neither is available (for rate limiting)

## Configuration API

### Server Configuration

```yaml
server:
  host: string          # Default: "0.0.0.0"
  port: number          # Default: 8080
  root: string          # Default: "."
  index: string         # Default: "index.html"
  auto_index: boolean  # Default: false
```

### TLS Configuration

```yaml
tls:
  enabled: boolean      # Default: false
  cert_path: string     # Required if enabled
  key_path: string      # Required if enabled
```

### Routing Configuration

```yaml
routing:
  - path: string        # Route path pattern
    serve: string       # "static" to serve static files
    proxy: object       # Proxy configuration (mutually exclusive with serve)
```

### Proxy Configuration

```yaml
proxy:
  url: string           # Backend URL
  timeout: duration     # Default: "5s"
  add_headers: object   # Key-value pairs, supports {client_ip}
```

### SPA Configuration

```yaml
spa:
  enabled: boolean      # Default: false
  fallback: string      # Default: "index.html"
```

### Compression Configuration

```yaml
compression:
  enable: boolean      # Default: false
  gzip: boolean        # Default: true
  br: boolean          # Default: true
```

### CORS Configuration

```yaml
security:
  cors:
    enabled: boolean           # Default: false
    allowed_origins: array      # Default: [] (allows all)
    allowed_methods: array      # Default: [] (common methods)
```

### Rate Limiting Configuration

```yaml
security:
  rate_limit:
    enabled: boolean          # Default: false
    requests_per_min: number   # Default: 60
```

### Security Headers Configuration

```yaml
security:
  headers:
    Header-Name: "header-value"
```

### Asset Cache Configuration

```yaml
assets:
  cache:
    enabled: boolean   # Default: false
    max_age: duration  # Default: "3600s"
    etag: boolean      # Default: true (not implemented)
```

### Observability Configuration

```yaml
obs:
  level: string  # "debug", "info", "warn", "error" (default: "info")
```

## Data Types

### Duration Format

Duration values support human-readable formats:

- `5s` - 5 seconds
- `1m` - 1 minute
- `30s` - 30 seconds
- `1h` - 1 hour
- `7d` - 7 days
- `3600s` - 3600 seconds

### Path Format

- Absolute paths: `/etc/ssl/certs/cert.pem`
- Relative paths: `./tls/cert.pem` (relative to working directory)

## Error Handling

### Configuration Errors

- **Invalid YAML**: Returns parsing error at startup
- **Missing TLS files**: Returns error if TLS enabled but files don't exist
- **Invalid routes**: Logs warning if route has both `serve` and `proxy`

### Runtime Errors

- **File not found**: Returns `404 Not Found`
- **Path traversal**: Returns `403 Forbidden`
- **Rate limit exceeded**: Returns `429 Too Many Requests`
- **Server errors**: Returns `500 Internal Server Error` with error details in logs

## Startup Sequence

1. Initialize logging (respects `RUST_LOG` or config)
2. Parse command-line arguments
3. Load configuration file (or use defaults)
4. Print configuration summary
5. Validate TLS configuration (if enabled)
6. Build router and middleware
7. Start server (HTTP or HTTPS)

## Logging

Log messages follow this format:

```
[LEVEL] message
```

Available levels:
- `debug`: Detailed debugging information
- `info`: General informational messages
- `warn`: Warning messages
- `error`: Error messages

Example output:
```
INFO Loaded configuration from statiker.yaml
INFO Mounting static route: /
INFO listening http://0.0.0.0:8080
```

