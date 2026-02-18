# Relay + Express Backend Example

This example demonstrates how to use Relay as a caching proxy in front of an Express.js backend service.

## Architecture

```
Client → Relay (port 8080) → Express Backend (port 3000)
```

Relay sits in front of the Express backend and caches responses according to the rules defined in `config.toml`.

## What's Included

- **Express Backend**: Simple Node.js/Express server with multiple endpoints demonstrating different caching scenarios
- **Relay**: HTTP caching proxy configured to cache responses from the Express backend
- **Docker Compose**: Orchestrates both services with proper health checks and networking

## Getting Started

1. Start the services:
   ```bash
   docker-compose up
   ```

2. Access the cached endpoints through Relay at `http://localhost:8080`:

   ```bash
   # Fast endpoint - cached for 30s
   curl http://localhost:8080/api/data

   # Slow endpoint - cached for 2m (first request takes 2s, subsequent requests are instant)
   curl http://localhost:8080/api/slow

   # Static content - cached for 1 day
   curl http://localhost:8080/api/static/info

   # Custom cache headers
   curl http://localhost:8080/api/custom-cache

   # Not cached
   curl http://localhost:8080/api/no-cache
   ```

3. Compare with direct backend access (not cached):
   ```bash
   curl http://localhost:3001/api/data
   ```

## Observing Cache Behavior

Each endpoint returns a `requestCount` field that increments with each request to the backend. When you make requests through Relay:

1. **First request**: Cache MISS - Relay forwards to backend, increments `requestCount`
2. **Subsequent requests (within TTL)**: Cache HIT - Relay serves from cache, `requestCount` stays the same
3. **After TTL expires**: Cache MISS - Relay revalidates with backend, `requestCount` increments

### Example

```bash
# First request (cache miss)
curl http://localhost:8080/api/data
# {"requestCount": 1, "timestamp": "2024-02-17T..."}

# Second request within 30s (cache hit)
curl http://localhost:8080/api/data
# {"requestCount": 1, "timestamp": "2024-02-17T..."}  # Same count!

# Direct backend request (bypasses cache)
curl http://localhost:3001/api/data
# {"requestCount": 2, "timestamp": "2024-02-17T..."}  # Count increased
```

## Cache Configuration

The `config.toml` file defines different caching strategies:

- `/api/data`: 30 second TTL with 5 minute stale window
- `/api/slow`: 2 minute TTL with 10 minute stale window
- `/static/*`: 1 day TTL for static content
- Default: 5 minute TTL with 1 hour stale-while-revalidate

## Monitoring

Prometheus metrics are enabled and available at:
```bash
curl http://localhost:8080/metrics
```

## Stopping the Services

```bash
docker-compose down
```

## Customization

- Modify `app/index.js` to add more endpoints
- Update `config.toml` to change caching rules
- Check the [Relay documentation](https://relay-http.com) for advanced configuration options

## Why Use Relay?

This example demonstrates how Relay:
- **Reduces backend load**: The backend only handles cache misses
- **Improves response times**: Cached responses are served in microseconds
- **Handles slow endpoints**: The `/api/slow` endpoint takes 2s on first request, but is instant when cached
- **Provides resilience**: With `stale_if_error`, Relay can serve cached content even if the backend fails
