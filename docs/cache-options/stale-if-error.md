# stale_if_error

The `stale_if_error` setting allows Relay to serve stale cached content when the upstream server is unavailable or returns an error.

## Configuration

```toml
[cache]
stale_if_error = "24h"
```

Supports standard [time format](../configuration.md#time-format): `1h`, `12h`, `24h`, `7d`

## What It Does

When the upstream server fails (network error, timeout, 5xx error), Relay can serve **stale** cached content instead of returning an error to the client. This provides graceful degradation and keeps your application available even when the backend is down.

### How It Works

1. **Normal Operation**: Relay caches responses with a timestamp
2. **Cache Freshness**: Fresh responses (age < `default_ttl`) are served normally
3. **Upstream Failure**: When the upstream server fails:
   - Relay checks if stale content exists in cache
   - If the stale content age is less than `ttl + stale_if_error`, it's served to the client
   - If no stale content exists or it's too old, the error is returned

### Headers Added

When serving stale content due to an error, Relay adds these headers:

```
X-Cache: STALE
X-Cache-Reason: upstream-error
```

## Default Value

If not specified, the default is **24 hours** (`24h`).

## When to Use

Stale-if-error is particularly valuable for:

- **High availability requirements**: Keep your site up even when backends fail
- **External API dependencies**: Gracefully handle third-party API outages
- **Maintenance windows**: Serve cached content during planned downtime
- **Spike protection**: Continue serving during upstream overload

## Example Scenarios

### Scenario 1: Short TTL, Long Error Window

```toml
[cache]
default_ttl = "5m"      # Fresh for 5 minutes
stale_if_error = "24h"  # Serve stale for up to 24 hours during errors
```

**Timeline:**
- **0-5 minutes**: Response is fresh, served from cache with `X-Cache: HIT`
- **After 5 minutes**: Response is stale, upstream is contacted
  - **If upstream succeeds**: New response cached and served
  - **If upstream fails**: Stale content served with `X-Cache: STALE` (up to 24 hours 5 minutes total)

### Scenario 2: API with Fallback

```toml
[cache]
default_ttl = "30s"
stale_if_error = "1h"

[cache.rules]
"/api/critical" = { ttl = "10s", stale_if_error = "5m" }
"/api/catalog" = { ttl = "5m", stale_if_error = "12h" }
```

- Critical API: Fresh for 10s, serves stale up to 5 minutes during errors
- Catalog API: Fresh for 5m, serves stale up to 12 hours during errors

## Errors That Trigger Stale Serving

Stale content is served when these errors occur:

- **Network errors**: Connection refused, timeout, DNS failures
- **Upstream errors**: Any error during the upstream request
- **Connection failures**: TCP handshake failures

## Metrics

Relay tracks stale responses with Prometheus metrics:

```
relay_cache_stale_served_total
```

This counter increments each time a stale response is served due to an upstream error.

## How It Works With Other Settings

### With default_ttl

The `stale_if_error` window starts **after** the `default_ttl` expires:

```
|<---- default_ttl ---->|<---- stale_if_error window ---->|
   Fresh content              Stale (only on error)          Expired
```

### With stale_while_revalidate

These two settings serve different purposes:

- **stale_while_revalidate**: Serve stale while fetching fresh (performance optimization)
- **stale_if_error**: Serve stale when upstream fails (availability optimization)

```toml
[cache]
default_ttl = "5m"
stale_while_revalidate = "1h"  # Serve stale while revalidating (performance)
stale_if_error = "24h"          # Serve stale if upstream is down (availability)
```

## Logging

When stale content is served due to an error, Relay logs:

```
Cache STALE (serving due to upstream error): /api/endpoint - error: connection refused
```

## Per-Path Override

Override `stale_if_error` for specific paths:

```toml
[cache]
stale_if_error = "24h"

[cache.rules]
"/api/realtime" = { stale_if_error = "5m" }  # Less stale tolerance
"/static/*" = { stale_if_error = "7d" }       # More stale tolerance
```

## Best Practices

1. **Set longer windows for static content**: Static assets can be served stale for days
2. **Balance freshness and availability**: Critical data may need shorter windows
3. **Monitor stale serving**: Track `relay_cache_stale_served_total` to detect upstream issues
4. **Test failure scenarios**: Verify your application works with stale data
5. **Consider data sensitivity**: User-specific data may need shorter or no stale windows

## Complete Example

```toml
[cache]
default_ttl = "10m"
stale_if_error = "24h"
stale_while_revalidate = "1h"

[cache.rules]
# Static assets: very stale-tolerant
"/static/*" = { ttl = "1d", stale_if_error = "30d" }

# API endpoints: balanced approach
"/api/catalog" = { ttl = "5m", stale_if_error = "12h" }

# Real-time data: minimal staleness
"/api/realtime" = { ttl = "10s", stale_if_error = "1m" }

# User-specific: no stale serving
"/api/user/*" = { ttl = "30s", stale_if_error = "0s" }
```

## Troubleshooting

### Stale Content Not Being Served

Check:
1. Is there cached content for the path? (Check cache metrics)
2. Has the stale window expired? (age > ttl + stale_if_error)
3. Is the error actually reaching Relay? (Check logs)
4. Is `stale_if_error` set to `0s` for that path?

### Too Much Stale Content

If clients receive stale content too often:
1. Check upstream health (may be frequently failing)
2. Review `relay_upstream_errors_total` metric
3. Consider reducing `stale_if_error` duration
4. Investigate why the upstream is failing

## See Also

- [default_ttl](default-ttl.md) - Configure cache freshness duration
- [stale_while_revalidate](stale-while-revalidate.md) - Serve stale while revalidating
- [Cache Rules](../cache-rules.md) - Configure per-path cache behavior
- [Monitoring](../monitoring.md) - Track cache and error metrics
