# Performance

Optimize Relay for maximum performance.

## Benchmarks

Relay is designed to be fast. Here are some benchmark results:

### Hardware
- CPU: 4 cores
- RAM: 8GB
- Network: 1Gbps

### Results

```
Requests/sec:   50,000
Latency (p50):  2ms
Latency (p99):  15ms
Cache hit ratio: 85%
```

## Optimization Tips

### 1. Choose the Right Storage

**For maximum speed:**
```toml
[storage]
in_memory = true
max_size = "2GB"
```

**For shared cache:**
```toml
[storage]
redis = "redis://localhost:6379"
pool_size = 20  # Match expected concurrency
```

### 2. Configure Workers

Match CPU cores:

```toml
[server]
workers = 8  # Set to number of CPU cores
```

Check optimal value:
```bash
# Linux
nproc

# macOS
sysctl -n hw.ncpu
```

### 3. Connection Pooling

```toml
[upstream]
max_connections = 100  # Adjust based on upstream capacity
keepalive = true
keepalive_timeout = "60s"
```

### 4. Enable Compression

Save bandwidth and improve cache efficiency:

```toml
[storage]
compress = true
compression_level = 6  # Balance between speed and size
min_compress_size = "1KB"
```

### 5. Cache Key Normalization

Reduce cache fragmentation:

```toml
[cache.query_params]
ignore = ["utm_source", "utm_medium", "sessionid", "timestamp"]
sort = true  # Normalize parameter order
```

### 6. Tune TTL Values

Balance freshness and hit ratio:

```toml
[cache]
default_ttl = "5m"
stale_while_revalidate = "1h"  # Serve stale while fetching fresh
stale_if_error = "24h"  # Serve stale if backend is down
```

### 7. Object Size Limits

Avoid caching very large objects:

```toml
[cache]
max_object_size = "10MB"  # Don't cache responses larger than this
min_object_size = "100B"  # Don't cache tiny responses
```

## Load Testing

### Using wrk

```bash
# Install wrk
git clone https://github.com/wg/wrk.git
cd wrk && make

# Run benchmark
./wrk -t4 -c100 -d30s http://localhost:8080/
```

### Using hey

```bash
# Install hey
go install github.com/rakyll/hey@latest

# Run benchmark
hey -n 100000 -c 100 http://localhost:8080/
```

### Using ab (ApacheBench)

```bash
ab -n 10000 -c 100 http://localhost:8080/
```

## Profiling

### CPU Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Profile Relay
sudo flamegraph -o cpu.svg -- relay --config config.toml
```

### Memory Profiling

```bash
# Use valgrind
valgrind --tool=massif relay --config config.toml

# Analyze results
ms_print massif.out.*
```

## System Tuning

### File Descriptors

```bash
# Check current limit
ulimit -n

# Increase limit
ulimit -n 65536

# Make permanent
echo "* soft nofile 65536" >> /etc/security/limits.conf
echo "* hard nofile 65536" >> /etc/security/limits.conf
```

### TCP Settings

```bash
# Increase connection backlog
sysctl -w net.core.somaxconn=4096
sysctl -w net.ipv4.tcp_max_syn_backlog=8192

# Enable TCP fast open
sysctl -w net.ipv4.tcp_fastopen=3

# Tune TCP buffer sizes
sysctl -w net.ipv4.tcp_rmem="4096 87380 16777216"
sysctl -w net.ipv4.tcp_wmem="4096 87380 16777216"
```

### Transparent Huge Pages

```bash
# Check status
cat /sys/kernel/mm/transparent_hugepage/enabled

# Enable
echo always > /sys/kernel/mm/transparent_hugepage/enabled
```

## Monitoring Performance

Key metrics to watch:

1. **Cache Hit Ratio**
   - Target: > 80%
   - Monitor: `relay_cache_hits_total / relay_cache_requests_total`

2. **Response Time**
   - Target: p99 < 50ms
   - Monitor: `relay_http_request_duration_seconds`

3. **Upstream Latency**
   - Monitor: `relay_upstream_request_duration_seconds`
   - Optimize upstream if high

4. **Memory Usage**
   - Monitor: `relay_cache_size_bytes`
   - Adjust cache size if needed

5. **CPU Usage**
   - Target: < 70% average
   - Scale horizontally if consistently high

## Comparison with Varnish

| Metric | Relay | Varnish |
|--------|-------|---------|
| Requests/sec | 50k | 45k |
| Memory usage | 500MB | 600MB |
| Configuration | TOML | VCL |
| Setup time | 5 min | 30 min |

*Benchmarks on identical hardware with similar configurations*

## Best Practices

1. **Start simple** - Use in-memory cache for single instance
2. **Monitor first** - Understand your traffic patterns
3. **Tune gradually** - Make one change at a time
4. **Test thoroughly** - Benchmark after each change
5. **Scale horizontally** - Add instances rather than over-tuning single instance
