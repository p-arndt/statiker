<p align="center">
  <img src="./docs/statiker_logo.png" alt="Statiker Logo" width="200">
</p>

<h1 align="center">Statiker</h1>

<p align="center">
  <strong>A simple, efficient static file hosting server written in Rust.</strong>
</p>

Statiker is a fast and efficient static file server with proxy support, built with Rust. It features sensible defaults (like Caddy) and powerful configuration options for advanced use cases.

## 🚀 Features

- **Simple & Fast**: Built with Rust for maximum performance
- **Sensible Defaults**: Works out of the box - just point it at a directory
- **Static File Serving**: Serve files from any directory with proper MIME types
- **Proxy Support**: Forward requests to backend services
- **TLS/HTTPS**: Full TLS support with automatic certificate handling
- **SPA Support**: Single Page Application fallback routing
- **Compression**: Gzip and Brotli compression support
- **Security**: CORS, rate limiting, and custom security headers
- **Auto-indexing**: Automatic directory listings
- **Docker Support**: Run with Docker or Docker Compose
- **Cross-Platform**: Available for Windows, macOS, and Linux

## 🛠️ Installation

### 📦 Pre-built Binaries

Download pre-built binaries from the [releases section](https://github.com/padi2312/statiker/releases).

### 🔨 Build from Source

```sh
git clone https://github.com/padi2312/statiker.git
cd statiker
cargo build --release

./target/release/statiker
```

### 🐳 Docker

```sh
docker run -d -p 8080:8080 -v /path/to/your/files:/app/static padi2312/statiker
```

### 📦 Docker Compose

See `docker-compose.yml` in the repository for a complete example.

## 📚 Usage

### Quick Start

**Option 1: Run without configuration (zero config)**

You can run Statiker without any configuration file! Just run:

```sh
statiker
```

This will use the following defaults:
- **Server**: Listen on `0.0.0.0:8080`
- **Root directory**: Current directory (`.`)
- **Index file**: `index.html`
- **Auto-index**: Disabled
- **Routing**: Automatically serves static files at `/`
- **Log level**: `info`

**Option 2: Use a configuration file**

Create a `statiker.yaml` file:

```yaml
server:
  root: static
  port: 8080
```

Then run:
```sh
statiker
```

Statiker will automatically serve files from the `static` directory at the root path (`/`).

> **Note**: If no `statiker.yaml` file is found, Statiker will fall back to built-in defaults and show a warning message. The server will still start and work perfectly fine!

### Command Line Options

```sh
statiker [OPTIONS]
```

**Options:**
- `-h, --help`: Display help information and exit
- `-c, --config <PATH>`: Path to configuration file (default: `statiker.yaml`)

**Environment Variables:**
- `CONFIG`: Path to configuration file (default: `statiker.yaml`)

### Help

Get help at any time:
```sh
statiker --help
# or
statiker -h
```

## ⚙️ Configuration

Statiker uses YAML configuration files. By default, it looks for `statiker.yaml` in the current directory, or you can specify a path with the `CONFIG` environment variable.

**No config file? No problem!** Statiker will use sensible defaults and work out of the box. You only need a config file if you want to customize the behavior.

### Default Configuration

When no config file is present, Statiker uses these defaults:

| Setting | Default Value |
|---------|---------------|
| Server host | `0.0.0.0` |
| Server port | `8080` |
| Root directory | `.` (current directory) |
| Index file | `index.html` |
| Auto-index | `false` |
| Routing | Serves static files at `/` (automatic) |
| TLS | Disabled |
| Compression | Disabled |
| CORS | Disabled |
| Rate limiting | Disabled |
| Log level | `info` |

### Minimal Configuration

If you want to customize just the root directory, create a minimal `statiker.yaml`:

```yaml
server:
  root: static
```

This will:
- Serve files from the `static` directory at `/`
- Listen on `0.0.0.0:8080` (default)
- Use `index.html` as the default index file (default)
- Auto-index disabled (default)

> **Tip**: Even this is optional! If you're in the directory you want to serve, you can just run `statiker` without any config file.

### Full Configuration Example

```yaml
server:
  host: 0.0.0.0
  port: 8080
  root: static
  index: index.html
  auto_index: true

# TLS/HTTPS (optional)
tls:
  enabled: true
  cert_path: ./tls/fullchain.pem
  key_path: ./tls/privkey.pem

# Routing (optional - defaults to serving static at /)
routing:
  - path: /
    serve: static
  - path: /api/*
    proxy:
      url: https://backend.example.com
      timeout: 5s
      add_headers:
        X-Forwarded-For: "{client_ip}"

# SPA Support (optional)
spa:
  enabled: true
  fallback: /index.html

# Asset Caching (optional)
assets:
  cache:
    enabled: true
    max_age: 7d
    etag: false

# Compression (optional)
compression:
  enable: true
  gzip: true
  br: true

# Security (optional)
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
    requests_per_min: 60
  headers:
    Strict-Transport-Security: "max-age=63072000; includeSubDomains; preload"
    X-Frame-Options: "DENY"
    X-Content-Type-Options: "nosniff"

# Observability (optional)
obs:
  level: info
```

### Configuration Reference

#### Server

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `host` | string | `0.0.0.0` | Server binding address |
| `port` | number | `8080` | Server port |
| `root` | string | `.` | Root directory for static files |
| `index` | string | `index.html` | Default index file name |
| `auto_index` | boolean | `false` | Enable directory listings |

#### TLS

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable TLS/HTTPS |
| `cert_path` | string | - | Path to certificate file |
| `key_path` | string | - | Path to private key file |

#### Routing

If no routes are configured, Statiker defaults to serving static files at `/`.

| Field | Type | Description |
|-------|------|-------------|
| `path` | string | Route path pattern (e.g., `/`, `/api/*`) |
| `serve` | string | Set to `"static"` to serve static files |
| `proxy` | object | Proxy configuration (see below) |

**Proxy Configuration:**
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `url` | string | - | Backend URL to proxy to |
| `timeout` | duration | `5s` | Request timeout |
| `add_headers` | object | - | Headers to add (supports `{client_ip}` placeholder) |

#### SPA (Single Page Application)

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable SPA fallback |
| `fallback` | string | `index.html` | Fallback file for SPA routing |

#### Compression

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enable` | boolean | `false` | Enable compression |
| `gzip` | boolean | `true` | Enable gzip compression |
| `br` | boolean | `true` | Enable Brotli compression |

#### Security

**CORS:**
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable CORS |
| `allowed_origins` | array | `[]` | Allowed origins (empty = all) |
| `allowed_methods` | array | `[]` | Allowed methods (empty = common methods) |

**Rate Limiting:**
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable rate limiting |
| `requests_per_min` | number | `60` | Requests per minute limit |

**Headers:**
Custom security headers as key-value pairs.

#### Assets Cache

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable asset caching |
| `max_age` | duration | `3600s` | Cache max age |
| `etag` | boolean | `true` | Enable ETag support |

#### Observability

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `level` | string | `info` | Log level (`debug`, `info`, `warn`, `error`) |

## 🎯 Use Cases

### Simple Static File Server

```yaml
server:
  root: ./public
  port: 8080
```

### Static Files with API Proxy

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
```

### SPA with Fallback

```yaml
server:
  root: ./dist

spa:
  enabled: true
  fallback: /index.html
```

### Production Setup with TLS

```yaml
server:
  host: 0.0.0.0
  port: 443
  root: ./public

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
  rate_limit:
    enabled: true
    requests_per_min: 100
  headers:
    Strict-Transport-Security: "max-age=63072000; includeSubDomains; preload"
    X-Frame-Options: "DENY"
    X-Content-Type-Options: "nosniff"

assets:
  cache:
    enabled: true
    max_age: 7d
```

## 📊 Startup Information

When Statiker starts, it prints a concise summary of the active configuration:

```
=== Configuration ===
Server: 0.0.0.0:8080
Root: static
Index: index.html
Auto-index: true
Routes: default (serve static at /)
Log level: info
====================
```

## 🔍 Features in Detail

### Default Static Serving

If no routes are configured, Statiker automatically serves static files from `server.root` at the root path (`/`). This makes it work like Caddy - just point it at a directory and it works!

### MIME Types

Statiker automatically detects and sets proper MIME types for all served files based on file extensions.

### Directory Listings

When `auto_index: true` is set, Statiker generates HTML directory listings for directories without an index file.

### Proxy Support

Forward requests to backend services with custom headers, timeouts, and path rewriting.

### Compression

Automatic compression of text-based files (HTML, CSS, JS, JSON, etc.) using gzip and/or Brotli.

## 📝 License

MIT

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
