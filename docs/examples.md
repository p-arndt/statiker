# Examples

This document provides practical examples for common Statiker use cases.

## Basic Static File Server

The simplest configuration - just serve files from a directory.

```yaml
server:
  root: ./public
  port: 8080
```

Run:
```bash
statiker
```

Access files at `http://localhost:8080/`.

## Static Files with Directory Listings

Enable automatic directory listings.

```yaml
server:
  root: ./public
  port: 8080
  auto_index: true
```

## Single Page Application (SPA)

Serve a React, Vue, or Angular app with client-side routing.

```yaml
server:
  root: ./dist
  index: index.html

spa:
  enabled: true
  fallback: /index.html
```

This ensures all routes (e.g., `/dashboard`, `/settings`) serve `index.html`, allowing the client-side router to handle routing.

## Static Files + API Proxy

Serve static files at the root and proxy API requests to a backend.

```yaml
server:
  root: ./static
  port: 8080

routing:
  - path: /
    serve: static
  - path: /api/*
    proxy:
      url: http://localhost:3000
      timeout: 10s
      add_headers:
        X-Forwarded-For: "{client_ip}"
```

- Static files: `http://localhost:8080/`
- API requests: `http://localhost:8080/api/*` → `http://localhost:3000/api/*`

## Production Setup with TLS

Full production configuration with HTTPS, compression, and security.

```yaml
server:
  host: 0.0.0.0
  port: 443
  root: ./public
  index: index.html

tls:
  enabled: true
  cert_path: /etc/ssl/certs/fullchain.pem
  key_path: /etc/ssl/private/privkey.pem

compression:
  enable: true
  gzip: true
  br: true

security:
  cors:
    enabled: true
    allowed_origins:
      - https://example.com
      - https://www.example.com
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
    X-XSS-Protection: "1; mode=block"

assets:
  cache:
    enabled: true
    max_age: 7d
```

## Development Server

Quick development setup with auto-indexing and debug logging.

```yaml
server:
  root: ./src
  port: 3000
  auto_index: true

obs:
  level: debug
```

## Multiple Proxy Routes

Proxy different paths to different backends.

```yaml
server:
  root: ./static
  port: 8080

routing:
  - path: /
    serve: static
  - path: /api/*
    proxy:
      url: http://api-backend:3000
      timeout: 5s
  - path: /auth/*
    proxy:
      url: http://auth-backend:4000
      timeout: 10s
      add_headers:
        X-Service: "auth"
```

## CDN-like Asset Serving

Optimized configuration for serving assets with aggressive caching.

```yaml
server:
  root: ./assets
  port: 8080

compression:
  enable: true
  gzip: true
  br: true

assets:
  cache:
    enabled: true
    max_age: 365d

security:
  headers:
    Cache-Control: "public, max-age=31536000, immutable"
```

## Docker Deployment

Example `statiker.yaml` for Docker:

```yaml
server:
  host: 0.0.0.0
  port: 8080
  root: /app/static
  auto_index: false

compression:
  enable: true
  gzip: true
  br: true

assets:
  cache:
    enabled: true
    max_age: 7d
```

Docker Compose:

```yaml
services:
  statiker:
    image: padi2312/statiker
    ports:
      - "8080:8080"
    volumes:
      - ./statiker.yaml:/app/statiker.yaml
      - ./static:/app/static
```

## Reverse Proxy Setup

Use Statiker as a reverse proxy with static file fallback.

```yaml
server:
  port: 8080
  root: ./public

routing:
  - path: /api/*
    proxy:
      url: https://backend.example.com
      timeout: 30s
      add_headers:
        X-Forwarded-For: "{client_ip}"
        X-Forwarded-Proto: "https"
  - path: /
    serve: static
```

## Microservices Gateway

Route different services based on path.

```yaml
server:
  port: 8080
  root: ./static

routing:
  - path: /users/*
    proxy:
      url: http://user-service:3001
  - path: /products/*
    proxy:
      url: http://product-service:3002
  - path: /orders/*
    proxy:
      url: http://order-service:3003
  - path: /
    serve: static
```

## High-Performance Static Server

Optimized for maximum performance with compression and caching.

```yaml
server:
  root: ./public
  port: 8080

compression:
  enable: true
  gzip: true
  br: true

assets:
  cache:
    enabled: true
    max_age: 30d

security:
  headers:
    Cache-Control: "public, max-age=2592000"
```

## Development with Hot Reload

For development, you might want to serve from a source directory with auto-indexing:

```yaml
server:
  root: ./src
  port: 3000
  auto_index: true

obs:
  level: debug

# Disable caching in development
assets:
  cache:
    enabled: false
```

## Zero Configuration

You can run Statiker without any configuration file:

```bash
statiker
```

This uses all defaults:
- Serves from current directory (`.`)
- Listens on `0.0.0.0:8080`
- Uses `index.html` as index file
- Auto-index disabled
- All other features disabled

Perfect for quick testing or when defaults are sufficient!

