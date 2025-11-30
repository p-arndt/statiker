# Features

Statiker provides a comprehensive set of features for serving static files and proxying requests. This document details each feature.

## Static File Serving

Statiker's core feature is serving static files with proper MIME type detection and HTTP compliance.

### Default Behavior

If no routes are configured, Statiker automatically serves static files from `server.root` at the root path (`/`). This makes it work like Caddy - just point it at a directory and it works!

### Features

- **Automatic MIME Type Detection**: Files are served with correct `Content-Type` headers based on file extensions
- **Index File Support**: Automatically serves `index.html` (or configured index file) for directory requests
- **Directory Listings**: Optional automatic directory listings when `auto_index` is enabled
- **Path Traversal Protection**: Requests with `..` components are rejected with `403 Forbidden`
- **HTTP Compliance**: Proper `Content-Length` headers for both GET and HEAD requests
- **Method Support**: Supports GET and HEAD requests (other methods return `405 Method Not Allowed`)

### Example

```yaml
server:
  root: ./public
  index: index.html
  auto_index: true
```

## Proxy Support

Forward requests to backend services with configurable timeouts and custom headers.

### Features

- **Path-based Routing**: Proxy specific paths (e.g., `/api/*`) to backend services
- **Timeout Configuration**: Configurable request timeouts
- **Custom Headers**: Add custom headers to proxied requests
- **Client IP Forwarding**: Automatic `X-Forwarded-For` header support with `{client_ip}` placeholder
- **HTTPS Support**: Proxies to both HTTP and HTTPS backends

### Example

```yaml
routing:
  - path: /api/*
    proxy:
      url: https://api.example.com
      timeout: 10s
      add_headers:
        X-Forwarded-For: "{client_ip}"
        X-API-Key: "your-api-key"
```

### Client IP Placeholder

The `{client_ip}` placeholder in `add_headers` is automatically replaced with the client's IP address. The IP is extracted from:
1. `X-Forwarded-For` header (first IP in the list)
2. Socket address from the connection

## Single Page Application (SPA) Support

Enable fallback routing for Single Page Applications.

### Features

- **Fallback Routing**: All non-file requests fall back to a specified file (typically `index.html`)
- **Path Traversal Protection**: Fallback paths are validated to prevent directory traversal
- **Automatic Fallback**: If an invalid fallback path is detected, defaults to `index.html`

### Example

```yaml
spa:
  enabled: true
  fallback: /index.html
```

This configuration ensures that routes like `/dashboard`, `/settings`, etc. all serve `index.html`, allowing the client-side router to handle the routing.

## TLS/HTTPS

Full TLS support for secure connections.

### Features

- **PEM Certificate Support**: Uses standard PEM format certificates
- **Startup Validation**: Validates certificate and key files exist and are readable at startup
- **Error Handling**: Graceful error handling if TLS configuration is invalid

### Example

```yaml
tls:
  enabled: true
  cert_path: /etc/ssl/certs/fullchain.pem
  key_path: /etc/ssl/private/privkey.pem
```

### Certificate Generation

For development, you can generate self-signed certificates:

```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

## Compression

Automatic response compression to reduce bandwidth usage.

### Features

- **Gzip Compression**: Standard gzip compression
- **Brotli Compression**: Modern Brotli compression (better compression ratio)
- **Automatic Negotiation**: Server automatically selects the best compression method based on client support
- **Selective Compression**: Only compresses when enabled and at least one method is selected

### Example

```yaml
compression:
  enable: true
  gzip: true
  br: true
```

## CORS (Cross-Origin Resource Sharing)

Configure CORS headers for cross-origin requests.

### Features

- **Origin Whitelisting**: Specify allowed origins
- **Method Control**: Control which HTTP methods are allowed
- **Flexible Configuration**: Empty origins list allows all origins (useful for development)

### Example

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
      - PUT
      - DELETE
      - OPTIONS
```

## Rate Limiting

Protect your server from abuse with IP-based rate limiting.

### Features

- **IP-based Tracking**: Rate limits are applied per IP address
- **Configurable Limits**: Set requests per minute
- **Fallback Protection**: Unknown IPs are tracked under a fallback IP to prevent bypassing limits
- **Automatic Headers**: Extracts client IP from `X-Forwarded-For` header or socket address

### Example

```yaml
security:
  rate_limit:
    enabled: true
    requests_per_min: 100
```

When rate limit is exceeded, Statiker returns `429 Too Many Requests`.

## Security Headers

Add custom security headers to all responses.

### Common Security Headers

```yaml
security:
  headers:
    Strict-Transport-Security: "max-age=63072000; includeSubDomains; preload"
    X-Frame-Options: "DENY"
    X-Content-Type-Options: "nosniff"
    X-XSS-Protection: "1; mode=block"
    Referrer-Policy: "strict-origin-when-cross-origin"
    Content-Security-Policy: "default-src 'self'"
```

### Header Descriptions

- **Strict-Transport-Security**: Forces HTTPS connections
- **X-Frame-Options**: Prevents clickjacking attacks
- **X-Content-Type-Options**: Prevents MIME type sniffing
- **X-XSS-Protection**: Enables XSS filtering in browsers
- **Referrer-Policy**: Controls referrer information
- **Content-Security-Policy**: Controls resource loading

## Asset Caching

Automatic cache headers for static assets.

### Features

- **Automatic Detection**: Automatically applies to files with asset extensions (CSS, JS, images, fonts, media)
- **Configurable Max-Age**: Set cache duration (supports human-readable formats like `7d`, `1h`)
- **Immutable Assets**: Assets are marked as immutable for optimal caching

### Supported Asset Extensions

- **Stylesheets**: `css`
- **JavaScript**: `js`, `mjs`, `map`
- **Images**: `png`, `jpg`, `jpeg`, `gif`, `webp`, `svg`, `ico`
- **Fonts**: `ttf`, `otf`, `woff`, `woff2`
- **Media**: `mp4`, `webm`, `mp3`

### Example

```yaml
assets:
  cache:
    enabled: true
    max_age: 7d
```

This adds `Cache-Control: public, max-age=604800, immutable` headers to asset files.

## Auto-Indexing

Automatic directory listings when no index file is found.

### Features

- **HTML Directory Listings**: Generates clean HTML directory listings
- **Parent Directory Links**: Includes ".." link for navigation
- **Sorted Display**: Directories first, then files, both alphabetically
- **Security**: Path traversal protection prevents accessing parent directories

### Example

```yaml
server:
  auto_index: true
```

When enabled and a directory is requested without an index file, Statiker generates an HTML listing of the directory contents.

## Logging

Structured logging with configurable levels.

### Features

- **Log Levels**: `debug`, `info`, `warn`, `error`
- **Environment Variable Support**: Can be overridden with `RUST_LOG` environment variable
- **Startup Logging**: Early initialization ensures all log messages are captured
- **Structured Output**: Clean, readable log format

### Example

```yaml
obs:
  level: debug
```

Or via environment variable:

```bash
RUST_LOG=debug statiker
```

## Route Configuration

Flexible routing with static file serving and proxying.

### Route Types

1. **Static File Serving**: Serve files from the configured root directory
2. **Proxy**: Forward requests to backend services

### Route Matching

- Routes are matched in order
- Path patterns support wildcards (e.g., `/api/*`)
- Routes are mutually exclusive (cannot have both `serve: static` and `proxy`)

### Default Route

If no routes are configured, Statiker automatically creates a static file route at `/` serving from `server.root`.

