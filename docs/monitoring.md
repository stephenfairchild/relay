# Monitoring

Relay provides built-in monitoring and observability features.

## Prometheus Metrics

Relay exposes Prometheus metrics at `/metrics`:

```bash
curl http://localhost:8080/metrics
```

### Available Metrics

#### Cache Metrics

```
# Cache hits and misses
relay_cache_hits_total
relay_cache_misses_total

# Cache size
relay_cache_size_bytes
relay_cache_items_total

# Cache operations
relay_cache_set_duration_seconds
relay_cache_get_duration_seconds
```

#### HTTP Metrics

```
# Request count by status
relay_http_requests_total{status="200"}
relay_http_requests_total{status="404"}

# Request duration
relay_http_request_duration_seconds

# Response sizes
relay_http_response_size_bytes
```

#### Upstream Metrics

```
# Upstream request duration
relay_upstream_request_duration_seconds

# Upstream errors
relay_upstream_errors_total
```

## Prometheus Configuration

Add Relay to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'relay'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

## Grafana Dashboard

Import the Relay dashboard:

1. Download dashboard JSON from the repo
2. Import in Grafana
3. Select your Prometheus datasource

Key panels:
- Cache hit ratio
- Request rate
- Response times
- Cache size and evictions
- Upstream health

## Logging

Configure logging level:

```bash
RUST_LOG=info relay
```

Levels:
- `error` - Only errors
- `warn` - Warnings and errors
- `info` - General information (default)
- `debug` - Detailed debugging
- `trace` - Very verbose

### Structured Logging

Relay outputs JSON logs for easy parsing:

```json
{
  "timestamp": "2024-02-17T12:00:00Z",
  "level": "info",
  "message": "Request completed",
  "path": "/api/users",
  "status": 200,
  "duration_ms": 45,
  "cache": "hit"
}
```

## Health Checks

Check Relay health:

```bash
curl http://localhost:8080/health
```

Response:

```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "cache": {
    "items": 1250,
    "size_bytes": 52428800,
    "hit_ratio": 0.85
  },
  "upstream": {
    "status": "up",
    "last_check": "2024-02-17T12:00:00Z"
  }
}
```

## Alerting

Example Prometheus alerts:

```yaml
groups:
  - name: relay
    rules:
      - alert: RelayCacheHitRatioLow
        expr: rate(relay_cache_hits_total[5m]) / rate(relay_cache_requests_total[5m]) < 0.5
        for: 10m
        annotations:
          summary: "Relay cache hit ratio is low"

      - alert: RelayUpstreamDown
        expr: relay_upstream_errors_total > 0
        for: 5m
        annotations:
          summary: "Relay upstream is experiencing errors"

      - alert: RelayHighLatency
        expr: histogram_quantile(0.95, relay_http_request_duration_seconds) > 1
        for: 5m
        annotations:
          summary: "Relay 95th percentile latency is high"
```

## Debugging

Enable debug logging for troubleshooting:

```bash
RUST_LOG=relay=debug relay
```

Or for specific modules:

```bash
RUST_LOG=relay::cache=trace,relay::upstream=debug relay
```
