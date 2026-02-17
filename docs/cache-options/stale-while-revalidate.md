# stale_while_revalidate

The `stale_while_revalidate` setting allows Relay to serve stale content immediately while fetching fresh content in the background.

## Configuration

```toml
[cache]
stale_while_revalidate = "1h"
```

Supports standard [time format](../configuration.md#time-format): `30s`, `5m`, `1h`, `12h`

## What It Does

When cached content becomes stale (age > `default_ttl`), Relay can serve the stale content immediately to the client while simultaneously fetching fresh content from the upstream in the background. This provides the performance benefits of caching while ensuring content stays reasonably fresh.

### How It Works

1. **Fresh Content** (age < `default_ttl`): Served directly from cache
2. **Stale Content** (age between `default_ttl` and `default_ttl + stale_while_revalidate`):
   - Immediately serve stale content to the client (fast response)
   - Trigger background request to upstream for fresh content
   - Next request gets the fresh content
3. **Expired Content** (age > `default_ttl + stale_while_revalidate`):
   - Wait for fresh content from upstream
   - Cache and serve the fresh response

### Headers Added

When serving stale content during revalidation:

```
X-Cache: STALE
X-Cache-Reason: revalidating
```

## Default Value

If not specified, the default is **1 hour** (`1h`).

## When to Use

Stale-while-revalidate is ideal for:

- **Performance optimization**: Eliminate cache miss latency
- **High-traffic sites**: Keep responses fast even when cache expires
- **Slowly changing content**: Content that changes occasionally but needs to stay updated
- **User experience**: Prioritize response speed over absolute freshness

## Example Scenarios

### Scenario 1: Performance-Optimized API

```toml
[cache]
default_ttl = "5m"                # Fresh for 5 minutes
stale_while_revalidate = "1h"     # Serve stale for 1 hour while revalidating
```

**Timeline:**
- **0-5 minutes**: Response is fresh, served from cache
- **5-65 minutes**: Response is stale, served immediately while revalidating in background
- **After 65 minutes**: Wait for fresh content (blocking request)

### Scenario 2: High Performance with Fallback

```toml
[cache]
default_ttl = "10m"
stale_while_revalidate = "30m"
stale_if_error = "24h"

[cache.rules]
"/api/catalog" = { ttl = "5m", stale_while_revalidate = "2h" }
```

- Normal case: Serve stale while revalidating (performance)
- Error case: Serve stale without revalidating (availability)

## Performance Benefits

### Without stale_while_revalidate

```
Client request → Cache miss → Wait for upstream → Respond
                               (slow, 100-500ms)
```

### With stale_while_revalidate

```
Client request → Serve stale → Respond (fast, <10ms)
                    ↓
            Background revalidation
            (client doesn't wait)
```

## Difference From stale_if_error

| Feature | stale_while_revalidate | stale_if_error |
|---------|------------------------|----------------|
| **Purpose** | Performance optimization | Availability during failures |
| **When triggered** | Every time cache becomes stale | Only when upstream fails |
| **Revalidation** | Always revalidates in background | No revalidation (upstream is down) |
| **Client wait time** | Immediate (serves stale) | Immediate (serves stale) |
| **Use case** | Speed up cache misses | Handle upstream failures |

## How It Works With Other Settings

### With default_ttl

```
|<---- default_ttl ---->|<-- stale_while_revalidate -->|
   Fresh (cache hit)       Stale + revalidate              Wait for fresh
```

### Complete Caching Strategy

```toml
[cache]
default_ttl = "5m"                # Fresh window
stale_while_revalidate = "1h"     # Performance window
stale_if_error = "24h"            # Availability window
```

**Combined behavior:**

- **0-5 min**: Fresh cache hit
- **5-65 min**: Serve stale while revalidating (unless upstream fails)
  - If revalidation succeeds: Cache updated in background
  - If revalidation fails: Serve stale (falls back to `stale_if_error`)
- **65 min - 24h 5min**: Wait for upstream
  - If upstream succeeds: Serve fresh
  - If upstream fails: Serve stale (up to the `stale_if_error` window)
- **After 24h 5min**: Must get fresh content, error if upstream fails

## Per-Path Override

```toml
[cache]
stale_while_revalidate = "1h"

[cache.rules]
"/static/*" = { stale_while_revalidate = "12h" }       # Long revalidation window
"/api/realtime" = { stale_while_revalidate = "0s" }    # Disable (always wait for fresh)
```

## Background Revalidation

When background revalidation is triggered:

1. Stale content is served to the current client immediately
2. Async request is made to upstream
3. If successful, cache is updated
4. Next client request gets fresh content
5. If failed, stale content remains (and `stale_if_error` may apply)

## Logging

```
Cache STALE (revalidating in background): /api/endpoint
```

## Best Practices

1. **Balance freshness and performance**: Longer windows = faster responses but potentially staler content
2. **Use with predictable traffic**: Most effective with regular request patterns
3. **Monitor revalidation**: Track how often background revalidation occurs
4. **Combine with stale_if_error**: Get both performance and availability benefits
5. **Consider user expectations**: Some data (prices, inventory) may need strict freshness

## Common Patterns

### Pattern 1: Speed-Optimized

```toml
[cache]
default_ttl = "30s"
stale_while_revalidate = "10m"  # Long revalidation window for speed
stale_if_error = "1h"
```

Optimizes for response speed, tolerates slightly stale content.

### Pattern 2: Freshness-Optimized

```toml
[cache]
default_ttl = "5m"
stale_while_revalidate = "30s"  # Short revalidation window
stale_if_error = "24h"
```

Keeps content fresh but provides error resilience.

### Pattern 3: Tiered Strategy

```toml
[cache]
default_ttl = "5m"
stale_while_revalidate = "1h"
stale_if_error = "24h"

[cache.rules]
# Static: maximize performance
"/static/*" = {
  ttl = "1d",
  stale_while_revalidate = "7d",
  stale_if_error = "30d"
}

# API: balance performance and freshness
"/api/*" = {
  ttl = "1m",
  stale_while_revalidate = "10m",
  stale_if_error = "1h"
}

# Real-time: prioritize freshness
"/api/realtime" = {
  ttl = "10s",
  stale_while_revalidate = "0s",  # Disabled
  stale_if_error = "30s"
}
```

## Metrics

Track revalidation with Prometheus metrics:

```
relay_cache_stale_served_total{reason="revalidating"}
```

## Troubleshooting

### Content Never Updates

Check:
1. Is background revalidation succeeding? (Check upstream error metrics)
2. Is the revalidation window too long?
3. Are there enough requests to trigger revalidation?

### Too Many Background Requests

If background revalidation creates too much load:
1. Reduce `stale_while_revalidate` duration
2. Increase `default_ttl` to reduce revalidation frequency
3. Consider request coalescing (multiple clients trigger one revalidation)

## Implementation Status

> **Note**: Background revalidation is a planned feature. Currently, expired cache entries trigger blocking requests to the upstream. Check the [architecture documentation](../architecture.md) for implementation details.

## See Also

- [default_ttl](default-ttl.md) - Configure cache freshness duration
- [stale_if_error](stale-if-error.md) - Serve stale content during failures
- [Cache Rules](../cache-rules.md) - Configure per-path cache behavior
- [Performance](../performance.md) - Optimize cache performance
