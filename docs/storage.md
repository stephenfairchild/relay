# Storage Backends

Relay supports multiple storage backends for cached content.

## In-Memory Storage

Fastest option, stored in RAM (default):

```toml
[storage]
backend = "memory"
```

**Pros:**
- Extremely fast
- No external dependencies
- Simple setup

**Cons:**
- Lost on restart
- Limited by RAM
- Not shared across instances

## Redis Storage

Shared cache across multiple instances:

```toml
[storage]
backend = "redis"

[storage.redis]
url = "redis://localhost:6379"
```

**Pros:**
- Shared across instances
- Persistent (if Redis is configured for persistence)
- Scalable

**Cons:**
- Network latency
- Requires Redis server

### Example

See the complete Redis example in `examples/redis-storage/` which includes:
- Docker Compose setup with Redis
- Sample configuration file
- Instructions for testing

## Future Storage Backends

The following backends are planned for future releases:

- **Disk Storage**: Store cache on filesystem for persistence without external dependencies
- **Hybrid Configuration**: Multi-tier caching with L1 (memory) and L2 (Redis/disk)
- **Eviction Policies**: LRU, LFU, FIFO when storage limits are reached
- **Compression**: Automatic compression for large responses

## Monitoring Storage

Check storage metrics at the `/metrics` endpoint:

```
relay_cache_size_bytes
relay_cache_items
relay_cache_hit_ratio
```
