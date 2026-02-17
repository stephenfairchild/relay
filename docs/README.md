# Relay

Relay is an open-source HTTP cache that brings the simplicity and ease-of-use of Caddy to HTTP caching. It's designed to be a drop-in replacement for Varnish, but with a focus on simplicity and modern features.

## Features

- **HTTP/1 Support** - Full HTTP/1.1 protocol support
- **Stale-While-Revalidate** - Serve stale content while fetching fresh data
- **Stale-If-Error** - Serve stale content on backend failures
- **Conditional Requests** - Support for If-None-Match and If-Modified-Since
- **Smart Cache Keys** - Ignore irrelevant query params and normalize cache keys
- **Flexible Invalidation** - TTL-based, pattern-based purging, and cache warming
- **Multiple Storage Backends** - In-memory, Redis, or disk storage
- **Built with Rust** - Memory-safe and blazingly fast

## Quick Start

```bash
# Run with default config
relay

# Run with custom config
relay --config relay.toml
```

## Example Configuration

```toml
# relay.toml
[upstream]
url = "http://localhost:8000"

[cache]
default_ttl = "5m"
stale_while_revalidate = "1h"
stale_if_error = "24h"

[cache.rules]
"/api/*" = { ttl = "30s", stale = "5m" }
"/static/*" = { ttl = "1d" }

[storage]
in_memory = false
redis = "http://redis:9000"
```

## Why Relay?

- **Simple Configuration** - No complex VCL, just clean TOML
- **Memory Safe** - Built with async Rust
- **Easy Deployment** - Single binary, single config file
- **Modern Features** - Designed for today's web applications

## Get Started

- [Installation](installation.md)
- [Configuration](configuration.md)
- [Docker Setup](docker.md)
- [Contributing](contributing.md)
