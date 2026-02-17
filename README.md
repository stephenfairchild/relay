# relay

A open-source fast and secure http cache. You know how Caddy changed the game for reverse proxies? It's just so easy compared to everything else. That's this but for http caching.

## Documentation

Full documentation is available at [https://relay-http.com](https://relay-http.com)

## Features
 - http/1 support
 - stale-while-revalidate 
 - stale-while-error (serve stale on backend failure)
 - conditional requests (If-None-Match, If-Modified-Since)
 - Ignore irrelevant query params for cache key. Normalize the cache key
 - Invalidation: TTL, purge by patterns and tags, cache warming on startup 
 - Storage: in memory, redis, disk

## Platforms
 - intel processors 
 - build for debian/bookworm platforms first

## Todo

- Write the main layer. Start a POC
- Expose a dashboard with observability
- Expose a prometheus endpoint by default at /metrics
- Run benchmarks compared against Varnish
- Social: logo, docs, website, discord
- Build and support for ARM

## Mission And Fundamental Guide

- async rust and memory safe as a feature
- embedded lua for request handling
- We use configuration over code. Make it easy for users to adopt with easy config. 
- Have easy deployable binaries that are easy to install and run. One CLI, one .toml. 
- Drop in Varnish Replacement. 


## Example Config

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
in_memory=false
redis = "http://redis:9000"
```
