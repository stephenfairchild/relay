# Storage Backends

Relay supports multiple storage backends for cached content.

## In-Memory Storage

Fastest option, stored in RAM:

```toml
[storage]
in_memory = true
max_size = "1GB"  # Optional: limit memory usage
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
in_memory = false
redis = "redis://localhost:6379"
```

**Pros:**
- Shared across instances
- Persistent (if Redis is configured for persistence)
- Scalable

**Cons:**
- Network latency
- Requires Redis server

### Redis Configuration

```toml
[storage.redis]
url = "redis://localhost:6379"
pool_size = 10
timeout = "1s"
```

## Disk Storage

Store cache on filesystem:

```toml
[storage]
disk = "/var/cache/relay"
max_size = "10GB"
```

**Pros:**
- Persistent across restarts
- No external dependencies
- Large capacity

**Cons:**
- Slower than memory
- Disk I/O overhead

## Hybrid Configuration

Combine storage backends for optimal performance:

```toml
[storage]
# Use in-memory as L1 cache
in_memory = true
max_memory = "512MB"

# Use Redis as L2 cache
redis = "redis://localhost:6379"

# Fallback to disk
disk = "/var/cache/relay"
max_disk = "10GB"
```

## Storage Policies

### Eviction

When storage is full:

```toml
[storage.eviction]
policy = "lru"  # Least Recently Used (default)
# policy = "lfu"  # Least Frequently Used
# policy = "fifo" # First In First Out
```

### Compression

Save space with compression:

```toml
[storage]
compress = true
compression_level = 6  # 1-9, higher = better compression
min_compress_size = "1KB"  # Only compress responses larger than this
```

## Monitoring Storage

Check storage metrics at the `/metrics` endpoint:

```
relay_cache_size_bytes
relay_cache_items
relay_cache_hit_ratio
```
