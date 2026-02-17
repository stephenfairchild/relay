# relay

A open-source fast and secure http cache. You know how Caddy changed the game for reverse proxies? It's just so easy compared to everything else. That's this but for http caching.

## What is Relay?

Relay sits between your users and your backend servers, storing (caching) responses to speed up your application and reduce load on your servers.

**The Problem:** Every time a user visits your website or app, your backend server has to do work - query databases, render pages, process data. When thousands of users make the same requests, your server does the same work over and over. This is slow and expensive.

**The Solution:** Relay saves responses from your backend and serves them directly to users. Instead of your server handling 10,000 requests, it might only handle 1 - and Relay serves the other 9,999 instantly from its cache.

**Real Benefits:**
- **Faster response times** - Cached responses are served in microseconds instead of milliseconds
- **Lower server costs** - Your backend handles a fraction of the traffic
- **Better reliability** - If your backend goes down, Relay can still serve cached content
- **Simple setup** - One binary, one config file, and you're running

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
