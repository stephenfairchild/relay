# Docker Setup

Run Relay with Docker for easy deployment.

## Quick Start

```bash
docker run -p 8080:8080 \
  -v $(pwd)/config.toml:/etc/relay/config.toml \
  ghcr.io/stephenfairchild/relay:latest
```

## Docker Compose

Create a `docker-compose.yml`:

```yaml
version: '3.8'

services:
  relay:
    image: ghcr.io/stephenfairchild/relay:latest
    ports:
      - "8080:8080"
    volumes:
      - ./config.toml:/etc/relay/config.toml
    environment:
      - RUST_LOG=info
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    restart: unless-stopped

volumes:
  redis-data:
```

Example `config.toml` for Docker setup:

```toml
[upstream]
url = "http://your-backend:8000"

[server]
host = "0.0.0.0"
port = 8080

[cache]
default_ttl = "5m"
stale_while_revalidate = "1h"
stale_if_error = "24h"

[storage]
redis = "redis://redis:6379"
```

Run with:

```bash
docker compose up -d
```

## Build Your Own Image

```dockerfile
FROM ghcr.io/stephenfairchild/relay:latest

# Copy your config
COPY config.toml /etc/relay/config.toml

# Expose port
EXPOSE 8080

CMD ["relay", "--config", "/etc/relay/config.toml"]
```

Build and run:

```bash
docker build -t my-relay .
docker run -p 8080:8080 my-relay
```

## Environment Variables

Override config with environment variables:

```bash
docker run -p 8080:8080 \
  -e RELAY_UPSTREAM_URL=http://backend:8000 \
  -e RELAY_CACHE_TTL=10m \
  ghcr.io/stephenfairchild/relay:latest
```

## Health Checks

Add health checks to your Docker setup:

```yaml
services:
  relay:
    image: ghcr.io/stephenfairchild/relay:latest
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 10s
```

## Production Deployment

For production, consider:

1. **Resource Limits**
```yaml
services:
  relay:
    image: ghcr.io/stephenfairchild/relay:latest
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G
```

2. **Logging**
```yaml
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

3. **Networks**
```yaml
networks:
  frontend:
  backend:

services:
  relay:
    networks:
      - frontend
      - backend
```
