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

## Systemd Service

Create `/etc/systemd/system/relay.service`:

```ini
[Unit]
Description=Relay HTTP Cache
After=network.target

[Service]
Type=simple
User=relay
Group=relay
WorkingDirectory=/opt/relay
ExecStart=/usr/local/bin/relay --config /etc/relay/config.toml
Restart=on-failure
RestartSec=5s

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/cache/relay

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable relay
sudo systemctl start relay
sudo systemctl status relay
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

### Load Balancing

Nginx load balancer config:

```nginx
upstream relay_cluster {
    least_conn;
    server relay1:8080 max_fails=3 fail_timeout=30s;
    server relay2:8080 max_fails=3 fail_timeout=30s;
    server relay3:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;

    location / {
        proxy_pass http://relay_cluster;
    }
}
```

## Security

### Rate Limiting

```toml
[rate_limit]
enabled = true
requests_per_second = 100
burst = 200
```

### Access Control

```toml
[security]
# Restrict admin endpoints
admin_allow_ips = ["10.0.0.0/8", "192.168.0.0/16"]

# Enable CORS if needed
cors_enabled = true
cors_origins = ["https://example.com"]
```

## Monitoring

Set up monitoring stack:

1. **Prometheus** - Metrics collection
2. **Grafana** - Visualization
3. **Alertmanager** - Alerts

See [Monitoring](monitoring.md) for details.

## Backups

### Configuration

- Backup `config.toml`
- Store in version control
- Use secrets management for sensitive values

### Cache Data

If using Redis:

```bash
# Backup Redis data
redis-cli BGSAVE

# Copy dump file
cp /var/lib/redis/dump.rdb /backup/
```

## Updates

### Zero-Downtime Updates

1. Start new instance with new version
2. Wait for health check to pass
3. Update load balancer to route to new instance
4. Drain old instance
5. Shut down old instance

### Rolling Updates with Docker

```bash
docker compose up -d --no-deps --build relay
```

## Performance Tuning

### OS Tuning

```bash
# Increase file descriptor limit
echo "* soft nofile 65536" >> /etc/security/limits.conf
echo "* hard nofile 65536" >> /etc/security/limits.conf

# TCP tuning
sysctl -w net.core.somaxconn=4096
sysctl -w net.ipv4.tcp_max_syn_backlog=8192
```

### Relay Tuning

```toml
[server]
workers = 8  # Match CPU cores
backlog = 2048

[cache]
max_object_size = "10MB"
min_object_size = "1KB"
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

### Profiling

```bash
# Install profiling tools
cargo install --force flamegraph

# Profile Relay
sudo flamegraph -o relay.svg -- relay --config config.toml
```
