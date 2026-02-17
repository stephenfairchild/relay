# Production Deployment

Best practices for running Relay in production.

## System Requirements

### Minimum

- 1 CPU core
- 512MB RAM
- 10GB disk space

### Recommended

- 2+ CPU cores
- 2GB+ RAM
- 50GB+ disk space (if using disk storage)
- SSD for better performance

## Configuration

### Production Config Example

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4  # Number of CPU cores

[upstream]
url = "http://backend:8000"
timeout = "30s"
max_connections = 100

[cache]
default_ttl = "5m"
stale_while_revalidate = "1h"
stale_if_error = "24h"

[cache.rules]
"/api/*" = { ttl = "30s", stale = "5m" }
"/static/*" = { ttl = "1d" }
"/admin/*" = { bypass = true }

[storage]
redis = "redis://redis:6379"
max_size = "2GB"

[storage.redis]
pool_size = 20
timeout = "1s"

[metrics]
enabled = true
path = "/metrics"
```

## Reverse Proxy

### Nginx

Put Relay behind Nginx for SSL termination:

```nginx
upstream relay {
    server localhost:8080;
}

server {
    listen 443 ssl http2;
    server_name cache.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://relay;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Caddy

Even simpler with Caddy:

```
cache.example.com {
    reverse_proxy localhost:8080
}
```

## High Availability

### Multiple Instances

Run multiple Relay instances with shared Redis:

```yaml
services:
  relay1:
    image: ghcr.io/stephenfairchild/relay:latest
    environment:
      - RELAY_STORAGE_REDIS=redis://redis:6379

  relay2:
    image: ghcr.io/stephenfairchild/relay:latest
    environment:
      - RELAY_STORAGE_REDIS=redis://redis:6379

  redis:
    image: redis:7-alpine
    volumes:
      - redis-data:/data

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    depends_on:
      - relay1
      - relay2
```

## Troubleshooting

### Common Issues

1. **High Memory Usage**
   - Reduce cache size
   - Enable compression
   - Check for memory leaks

2. **Low Cache Hit Ratio**
   - Review cache rules
   - Check TTL values
   - Analyze query parameters

3. **Upstream Timeouts**
   - Increase timeout values
   - Check backend health
   - Review connection pool size

### Debug Mode

```bash
RUST_LOG=debug relay --config config.toml
```
