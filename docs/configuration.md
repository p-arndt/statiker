# Configuration Reference

Statiker uses YAML configuration files. By default, it looks for `statiker.yaml` in the current directory.

## Configuration File Structure

```yaml
server:
  host: 0.0.0.0
  port: 8080
  root: static
  index: index.html
  auto_index: true

tls:
  enabled: false
  cert_path: ./tls/fullchain.pem
  key_path: ./tls/privkey.pem

routing:
  - path: /
    serve: static
  - path: /api/*
    proxy:
      url: https://backend.example.com
      timeout: 5s
      add_headers:
        X-Forwarded-For: "{client_ip}"

spa:
  enabled: true
  fallback: /index.html

assets:
  cache:
    enabled: true
    max_age: 7d
    etag: false

compression:
  enable: true
  gzip: true
  br: true

security:
  cors:
    enabled: true
    allowed_origins:
      - https://app.example.com
    allowed_methods:
      - GET
      - POST
      - PUT
      - DELETE
      - OPTIONS
  rate_limit:
    enabled: true
    requests_per_min: 100
  headers:
    Strict-Transport-Security: "max-age=63072000; includeSubDomains; preload"
    X-Frame-Options: "DENY"
    X-Content-Type-Options: "nosniff"

obs:
  level: info
```

## Configuration Sections

### Server

Basic server configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `host` | string | `0.0.0.0` | Host address to bind to |
| `port` | number | `8080` | Port number to listen on |
| `root` | string | `.` | Root directory to serve files from |
| `index` | string | `index.html` | Default index file name |
| `auto_index` | boolean | `false` | Enable automatic directory listings |

**Example:**

```yaml
server:
  host: 127.0.0.1
  port: 3000
  root: ./public
  index: index.html
  auto_index: true
```

### TLS

TLS/HTTPS configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable TLS/HTTPS |
| `cert_path` | string | - | Path to certificate file (PEM format) |
| `key_path` | string | - | Path to private key file (PEM format) |

**Example:**

```yaml
tls:
  enabled: true
  cert_path: /etc/ssl/certs/fullchain.pem
  key_path: /etc/ssl/private/privkey.pem
```

**Note:** Both `cert_path` and `key_path` must be provided when TLS is enabled. Statiker will validate that the files exist at startup.

### Routing

Route configuration for static file serving and proxying.

**Default Behavior:** If no routes are configured, Statiker automatically serves static files from `server.root` at `/`.

| Field | Type | Description |
|-------|------|-------------|
| `path` | string | Route path pattern (e.g., `/`, `/api/*`) |
| `serve` | string | Set to `"static"` to serve static files |
| `proxy` | object | Proxy configuration (see below) |

**Important:** Routes are mutually exclusive. A route can either `serve: static` OR have a `proxy` configuration, not both. If both are specified, the proxy will be ignored and a warning will be logged.

**Proxy Configuration:**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `url` | string | - | Backend URL to proxy to |
| `timeout` | duration | `5s` | Request timeout (supports formats like `5s`, `1m`, `30s`) |
| `add_headers` | object | - | Headers to add to proxied requests (supports `{client_ip}` placeholder) |

**Examples:**

```yaml
# Serve static files at root
routing:
  - path: /
    serve: static

# Proxy API requests
routing:
  - path: /api/*
    proxy:
      url: http://localhost:3000
      timeout: 10s
      add_headers:
        X-Forwarded-For: "{client_ip}"
        X-Custom-Header: "value"

# Multiple routes
routing:
  - path: /
    serve: static
  - path: /api/*
    proxy:
      url: https://api.example.com
```

### SPA (Single Page Application)

SPA fallback routing configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable SPA fallback |
| `fallback` | string | `index.html` | Fallback file for SPA routing (relative to root) |

**Example:**

```yaml
spa:
  enabled: true
  fallback: /index.html
```

**Security Note:** The fallback path is validated to prevent path traversal attacks. If an invalid path is detected, Statiker will fall back to `index.html` and log a warning.

### Assets Cache

Asset caching configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable asset caching |
| `max_age` | duration | `3600s` | Cache max age (supports formats like `7d`, `1h`, `3600s`) |
| `etag` | boolean | `true` | Enable ETag support (currently not implemented) |

**Example:**

```yaml
assets:
  cache:
    enabled: true
    max_age: 7d
    etag: false
```

**Note:** Asset caching applies to files with common asset extensions (CSS, JS, images, fonts, media files). See [Features](features.md#asset-caching) for details.

### Compression

Response compression configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enable` | boolean | `false` | Enable compression |
| `gzip` | boolean | `true` | Enable gzip compression |
| `br` | boolean | `true` | Enable Brotli compression |

**Example:**

```yaml
compression:
  enable: true
  gzip: true
  br: true
```

**Note:** Compression is only enabled if at least one method (gzip or br) is enabled.

### Security

Security features configuration.

#### CORS

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable CORS |
| `allowed_origins` | array | `[]` | Allowed origins (empty = all origins) |
| `allowed_methods` | array | `[]` | Allowed HTTP methods (empty = GET, POST, PUT, DELETE, OPTIONS) |

**Example:**

```yaml
security:
  cors:
    enabled: true
    allowed_origins:
      - https://app.example.com
      - https://www.example.com
    allowed_methods:
      - GET
      - POST
      - OPTIONS
```

#### Rate Limiting

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable rate limiting |
| `requests_per_min` | number | `60` | Maximum requests per minute per IP |

**Example:**

```yaml
security:
  rate_limit:
    enabled: true
    requests_per_min: 100
```

**Note:** Rate limiting uses IP-based tracking. If the client IP cannot be determined, requests are tracked under a fallback IP (`0.0.0.0`) to prevent bypassing rate limits.

#### Security Headers

Custom security headers as key-value pairs.

**Example:**

```yaml
security:
  headers:
    Strict-Transport-Security: "max-age=63072000; includeSubDomains; preload"
    X-Frame-Options: "DENY"
    X-Content-Type-Options: "nosniff"
    X-XSS-Protection: "1; mode=block"
    Referrer-Policy: "strict-origin-when-cross-origin"
```

### Observability

Logging configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `level` | string | `info` | Log level (`debug`, `info`, `warn`, `error`) |

**Example:**

```yaml
obs:
  level: debug
```

**Note:** You can also set the log level using the `RUST_LOG` environment variable, which takes precedence over the config file setting.

## Duration Format

Duration values (used in `timeout` and `max_age`) support human-readable formats:

- `5s` - 5 seconds
- `1m` - 1 minute
- `30s` - 30 seconds
- `1h` - 1 hour
- `7d` - 7 days
- `3600s` - 3600 seconds

## Configuration Validation

Statiker validates the configuration at startup:

- **TLS**: If enabled, both `cert_path` and `key_path` must be provided and files must exist
- **Routes**: Routes with both `serve: static` and `proxy` will log a warning (proxy is ignored)
- **SPA Fallback**: Path traversal attempts in the fallback path are detected and rejected

## Startup Output

When Statiker starts, it prints a summary of the active configuration:

```
=== Configuration ===
Server: 0.0.0.0:8080
Root: static
Index: index.html
Auto-index: true
Routes: default (serve static at /)
Compression: enabled (gzip, brotli)
CORS: enabled
Rate limit: 100 req/min
Security headers: 3 configured
Asset cache: enabled (max-age: 604800s)
Log level: info
====================
```

