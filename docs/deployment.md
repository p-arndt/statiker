# Deployment Guide

This guide covers deploying Statiker in various environments.

## Docker Deployment

### Basic Docker Run

```bash
docker run -d \
  -p 8080:8080 \
  -v /path/to/your/files:/app/static \
  -v /path/to/statiker.yaml:/app/statiker.yaml \
  padi2312/statiker
```

### Docker Compose

Create a `docker-compose.yml`:

```yaml
services:
  statiker:
    image: padi2312/statiker
    ports:
      - "8080:8080"
    volumes:
      - ./statiker.yaml:/app/statiker.yaml
      - ./static:/app/static
    restart: unless-stopped
```

Run:
```bash
docker-compose up -d
```

### Custom Dockerfile

If you need to build a custom image:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/statiker /usr/local/bin/statiker
WORKDIR /app
CMD ["statiker"]
```

## Systemd Service

Create `/etc/systemd/system/statiker.service`:

```ini
[Unit]
Description=Statiker Static File Server
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/var/www
ExecStart=/usr/local/bin/statiker
Restart=always
RestartSec=5
Environment="CONFIG=/etc/statiker/statiker.yaml"

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable statiker
sudo systemctl start statiker
sudo systemctl status statiker
```

## Reverse Proxy Setup

### Nginx

Example Nginx configuration:

```nginx
server {
    listen 80;
    server_name example.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Caddy

Caddyfile configuration:

```
example.com {
    reverse_proxy localhost:8080
}
```

### Traefik

Docker Compose with Traefik:

```yaml
services:
  statiker:
    image: padi2312/statiker
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.statiker.rule=Host(`example.com`)"
      - "traefik.http.services.statiker.loadbalancer.server.port=8080"
    volumes:
      - ./statiker.yaml:/app/statiker.yaml
      - ./static:/app/static
```

## TLS/HTTPS Setup

### Using Let's Encrypt with Certbot

1. Install Certbot:
```bash
sudo apt-get install certbot
```

2. Obtain certificate:
```bash
sudo certbot certonly --standalone -d example.com
```

3. Configure Statiker:
```yaml
server:
  port: 443

tls:
  enabled: true
  cert_path: /etc/letsencrypt/live/example.com/fullchain.pem
  key_path: /etc/letsencrypt/live/example.com/privkey.pem
```

4. Set up auto-renewal (optional, if using reverse proxy):
```bash
sudo certbot renew --dry-run
```

### Self-Signed Certificate (Development)

Generate self-signed certificate:

```bash
openssl req -x509 -newkey rsa:4096 \
  -keyout key.pem \
  -out cert.pem \
  -days 365 \
  -nodes \
  -subj "/CN=localhost"
```

Configure:
```yaml
tls:
  enabled: true
  cert_path: ./cert.pem
  key_path: ./key.pem
```

## Production Checklist

- [ ] Use TLS/HTTPS for all production deployments
- [ ] Set appropriate security headers
- [ ] Enable rate limiting
- [ ] Configure CORS properly (don't allow all origins in production)
- [ ] Enable compression for better performance
- [ ] Set up proper logging and monitoring
- [ ] Use process manager (systemd, supervisor, etc.)
- [ ] Set up reverse proxy if needed
- [ ] Configure firewall rules
- [ ] Set up automated backups
- [ ] Monitor disk space and resource usage

## Performance Tuning

### Enable Compression

```yaml
compression:
  enable: true
  gzip: true
  br: true
```

### Enable Asset Caching

```yaml
assets:
  cache:
    enabled: true
    max_age: 7d
```

### Optimize Logging

For production, use `info` or `warn` level:

```yaml
obs:
  level: info
```

## Monitoring

### Health Check Endpoint

Statiker doesn't have a built-in health check endpoint, but you can:

1. Create a simple `health.html` file in your static directory
2. Monitor the server port directly
3. Use a reverse proxy health check

### Log Monitoring

Statiker logs to stdout/stderr. For production:

1. Use a log aggregation service
2. Configure log rotation
3. Set up alerts for errors

## Scaling

Statiker is a single-process server. For horizontal scaling:

1. Run multiple instances behind a load balancer
2. Use sticky sessions if needed (though Statiker is stateless)
3. Ensure shared storage if serving files from disk

Example with multiple instances:

```yaml
# Load balancer configuration
upstream statiker {
    least_conn;
    server 127.0.0.1:8080;
    server 127.0.0.1:8081;
    server 127.0.0.1:8082;
}
```

## Security Best Practices

1. **Run as non-root user**: Use a dedicated user account
2. **File permissions**: Ensure proper file permissions on static files
3. **Firewall**: Only expose necessary ports
4. **Rate limiting**: Enable rate limiting to prevent abuse
5. **Security headers**: Configure appropriate security headers
6. **TLS**: Always use TLS in production
7. **Path traversal**: Statiker protects against path traversal, but ensure file permissions are correct

## Troubleshooting

### Server won't start

- Check if port is already in use
- Verify configuration file syntax (YAML)
- Check file permissions
- Review logs for error messages

### Files not serving

- Verify `server.root` path is correct
- Check file permissions
- Ensure files exist in the root directory
- Check if path traversal protection is blocking legitimate requests

### TLS errors

- Verify certificate and key files exist
- Check file permissions (must be readable)
- Ensure certificate is not expired
- Verify certificate format (must be PEM)

### Performance issues

- Enable compression
- Enable asset caching
- Check network configuration
- Monitor resource usage (CPU, memory, disk I/O)

