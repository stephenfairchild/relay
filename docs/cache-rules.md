# Cache Rules

Define specific caching behavior for different URL patterns.

## Basic Rules

```toml
[cache.rules]
"/api/*" = { ttl = "30s" }
"/static/*" = { ttl = "1d" }
```

## Rule Options

### TTL (Time To Live)

How long to cache the response:

```toml
"/api/users" = { ttl = "5m" }
```

### Stale While Revalidate

Serve stale content while fetching fresh data in the background:

```toml
"/api/*" = {
  ttl = "30s",
  stale = "5m"  # Serve stale for up to 5 minutes
}
```

### Bypass Cache

Never cache specific paths:

```toml
"/admin/*" = { bypass = true }
"/auth/*" = { bypass = true }
```

## Pattern Matching

Relay supports glob patterns:

- `*` - Match any characters
- `/api/*` - Match all paths under /api
- `/*.jpg` - Match all .jpg files
- `/users/*/profile` - Match nested paths

## Examples

### Static Assets

```toml
# Cache images for a week
"/*.jpg" = { ttl = "7d" }
"/*.png" = { ttl = "7d" }
"/*.webp" = { ttl = "7d" }

# Cache CSS/JS for a day
"/*.css" = { ttl = "1d" }
"/*.js" = { ttl = "1d" }
```

### API Endpoints

```toml
# Short cache for frequently updated data
"/api/live/*" = { ttl = "10s", stale = "30s" }

# Longer cache for stable data
"/api/products/*" = { ttl = "5m", stale = "1h" }

# Never cache authentication
"/api/auth/*" = { bypass = true }
```

### Content Pages

```toml
# Cache pages with stale-while-revalidate
"/blog/*" = { ttl = "10m", stale = "1h" }
"/docs/*" = { ttl = "30m", stale = "2h" }
```

## Query Parameters

Relay can normalize cache keys by ignoring certain query parameters:

```toml
[cache.query_params]
ignore = ["utm_source", "utm_medium", "utm_campaign"]
```

This ensures `page?utm_source=twitter` and `page?utm_source=facebook` use the same cache entry.
