# Redis Storage Example

This example demonstrates how to use Relay with Redis as the storage backend.

## Prerequisites

- Docker and Docker Compose installed

## Running the Example

1. Start the services:
   ```bash
   docker-compose up -d
   ```

2. Test the relay:
   ```bash
   # First request (cache miss)
   curl http://localhost:3000/get

   # Second request (cache hit from Redis)
   curl http://localhost:3000/get
   ```

3. Check metrics:
   ```bash
   curl http://localhost:3000/metrics
   ```

4. Inspect Redis to see cached entries:
   ```bash
   docker-compose exec redis redis-cli
   > KEYS *
   > GET "/get:body"
   ```

5. Stop the services:
   ```bash
   docker-compose down
   ```

## Configuration

The `config.toml` file configures Relay to use Redis as the storage backend:

```toml
[storage]
backend = "redis"

[storage.redis]
url = "redis://redis:6379"
```

## Available Storage Backends

- `memory` - In-memory HashMap storage (default)
- `redis` - Redis storage with persistence
