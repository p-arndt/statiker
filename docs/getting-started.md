# Getting Started with Statiker

Statiker is a fast, efficient static file server written in Rust. This guide will help you get up and running quickly.

## Installation

### Pre-built Binaries

Download pre-built binaries from the [releases section](https://github.com/padi2312/statiker/releases) for your platform.

### Build from Source

```bash
git clone https://github.com/padi2312/statiker.git
cd statiker
cargo build --release

# The binary will be at ./target/release/statiker
```

### Docker

```bash
docker pull padi2312/statiker
```

## Quick Start

### Zero Configuration

Statiker works out of the box with sensible defaults. Simply run:

```bash
statiker
```

This will:
- Listen on `0.0.0.0:8080`
- Serve files from the current directory (`.`)
- Use `index.html` as the default index file
- Automatically serve static files at `/`

### With Configuration File

Create a `statiker.yaml` file:

```yaml
server:
  root: ./static
  port: 8080
```

Then run:

```bash
statiker
```

## Command Line Options

```bash
statiker [OPTIONS]
```

**Options:**
- `-h, --help`: Display help information
- `-c, --config <PATH>`: Path to configuration file (default: `statiker.yaml`)

**Environment Variables:**
- `CONFIG`: Path to configuration file (alternative to `-c` flag)

**Examples:**

```bash
# Use default config file (statiker.yaml)
statiker

# Specify custom config file
statiker --config /path/to/config.yaml
statiker -c custom.yaml

# Use environment variable
CONFIG=my-config.yaml statiker
```

## Default Behavior

When no configuration file is found, Statiker uses these defaults:

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

## Next Steps

- Read the [Configuration Reference](configuration.md) for detailed configuration options
- Check out [Examples](examples.md) for common use cases
- Learn about [Features](features.md) for advanced functionality

