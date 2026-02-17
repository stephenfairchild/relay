# Relay - Simple HTTP Cache Server

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://github.com/stephenfairchild/relay/pkgs/container/relay)

> **A fast, modern HTTP caching proxy built in Rust. Like Caddy simplified reverse proxies, Relay simplifies HTTP caching.**

Relay is an open-source HTTP cache server and caching proxy that brings simplicity to HTTP caching. Think of it as **Caddy for HTTP caching** - if Caddy made reverse proxies easy, Relay does the same for HTTP caching. One binary, one config file, production-ready out of the box.

**Perfect for:** API caching, CDN edge caching, reducing backend load, improving response times, and building resilient web services.

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


## Quick Start

Create a `config.toml`:

```toml
[upstream]
url = "http://host.docker.internal:8000"

[cache]
default_ttl = "5m"
stale_while_revalidate = "1h"
stale_if_error = "24h"

[storage]
in_memory = true
```

Run with Docker:

```bash
docker run -p 8080:8080 \
  -v $(pwd)/config.toml:/etc/relay/config.toml \
  ghcr.io/stephenfairchild/relay:latest
```

Test it:

```bash
curl http://localhost:8080/api/data
```

## Advanced Configuration

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
