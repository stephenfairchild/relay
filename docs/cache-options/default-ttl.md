# default_ttl

The `default_ttl` setting controls how long responses are considered fresh in the cache.

## Configuration

```toml
[cache]
default_ttl = "5m"
```

Supports standard [time format](../configuration.md#time-format): `30s`, `5m`, `2h`, `7d`

## What It Does

The `default_ttl` (Time To Live) defines the duration that a cached response is considered **fresh** and can be served to clients without checking the upstream server.

- When a response is cached, Relay stores it along with a timestamp
- Requests for that resource will be served from cache while the age is less than the TTL
- Once the TTL expires, the cached response becomes **stale**
- Stale responses trigger different behavior based on other cache settings

## Default Value

If not specified, the default TTL is **5 minutes** (`5m`).

## When to Use

Choose your TTL based on how often your content changes:

| Content Type | Recommended TTL | Example |
|-------------|----------------|---------|
| Static assets (CSS, JS, images) | Hours to days | `24h` or `7d` |
| API responses (frequently changing) | Seconds to minutes | `30s` or `5m` |
| Semi-static content | Minutes to hours | `15m` or `2h` |
| User-specific data | Very short or none | `10s` or disable caching |

## Per-Path Override

You can override the default TTL for specific paths using [cache rules](../cache-rules.md):

```toml
[cache]
default_ttl = "5m"

[cache.rules]
"/api/*" = { ttl = "30s" }
"/static/*" = { ttl = "1d" }
```

## How It Works With Other Settings

The TTL works together with other cache options:

- **[stale_if_error](stale-if-error.md)**: When the upstream is down, serve stale content beyond the TTL
- **[stale_while_revalidate](stale-while-revalidate.md)**: Serve stale content while fetching fresh content in the background

## Example

```toml
[cache]
default_ttl = "10m"              # Fresh for 10 minutes
stale_if_error = "24h"           # Can serve stale for 24h if upstream is down
stale_while_revalidate = "1h"    # Can serve stale for 1h while revalidating
```

With this configuration:
- **0-10 minutes**: Response is fresh, served from cache
- **10-70 minutes**: Response is stale, triggers background revalidation (if `stale_while_revalidate` applies)
- **10 minutes - 24 hours 10 minutes**: Response is stale but can be served if upstream errors occur
- **After 24 hours 10 minutes**: Response cannot be served, even during errors

## See Also

- [stale_if_error](stale-if-error.md) - Serve stale content during upstream failures
- [stale_while_revalidate](stale-while-revalidate.md) - Serve stale content while revalidating
- [Cache Rules](../cache-rules.md) - Configure per-path cache behavior
