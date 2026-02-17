# Relay - Simple HTTP Cache Server

> A fast, modern HTTP caching proxy built in Rust. Like Caddy simplified reverse proxies, Relay simplifies HTTP caching.

Relay is an open-source HTTP cache server that brings simplicity to HTTP caching. Think of it as **Caddy for HTTP caching** - easy to configure, powerful, and production-ready out of the box. If you need to cache HTTP responses, reduce backend load, or build a CDN-like layer for your API, Relay makes it simple.

## What is Relay?

Relay sits between your users and your backend servers, storing (caching) responses to speed up your application and reduce load on your servers.

**The Problem:** Every time a user visits your website or app, your backend server has to do work - query databases, render pages, process data. When thousands of users make the same requests, your server does the same work over and over. This is slow and expensive.

**The Solution:** Relay saves responses from your backend and serves them directly to users. Instead of your server handling 10,000 requests, it might only handle 1 - and Relay serves the other 9,999 instantly from its cache.

**Real Benefits:**
- **Faster response times** - Cached responses are served in microseconds instead of milliseconds
- **Lower server costs** - Your backend handles a fraction of the traffic
- **Better reliability** - If your backend goes down, Relay can still serve cached content
- **Simple setup** - One binary, one config file, and you're running

## Why Choose Relay?

**Modern Varnish Alternative** - Relay is a drop-in replacement for Varnish with simpler configuration and better developer experience.

**Production-Ready** - Powers real applications with features like stale-while-revalidate, automatic failover, and comprehensive monitoring.

**Developer-Friendly** - One binary, one config file. No VCL, no complex syntax. Just straightforward TOML configuration.

## Key Features

### Caching & Performance
- **HTTP/1.1 Support** - Full RFC 7234 HTTP caching specification compliance
- **Stale-While-Revalidate** - Serve cached content while fetching fresh data in the background for zero-latency updates
- **Stale-If-Error** - Automatic failover to cached content when backend servers are down or returning errors
- **Conditional Requests** - Efficient validation with If-None-Match and If-Modified-Since headers
- **Smart Cache Keys** - Ignore irrelevant query parameters and normalize cache keys for better hit rates

### Storage & Scalability
- **Multiple Storage Backends** - Choose between in-memory (fastest), Redis (distributed), or disk storage (persistent)
- **Flexible Invalidation** - TTL-based expiration, pattern-based cache purging, tag-based invalidation, and startup cache warming
- **Cache Rules** - Path-based caching policies with glob pattern matching

### Operations & Monitoring
- **Prometheus Metrics** - Built-in `/metrics` endpoint with detailed cache hit/miss ratios, latency, and throughput stats
- **Production Ready** - Memory-safe Rust implementation with predictable performance
- **Simple Deployment** - Docker images, static binaries, and systemd support included

### Developer Experience
- **Easy Configuration** - Human-readable TOML configuration, no complex DSL or scripting language required
- **Fast Startup** - Sub-second startup time with optional cache warming
- **Zero Dependencies** - Single static binary with no runtime dependencies

## Installation

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

## Get Started

- [Quick Start Guide](quick-start.md)
- [Configuration](configuration.md)
- [Docker Setup](docker.md)
- [Monitoring](monitoring.md)
- [Contributing](contributing.md)
- [Changelog](https://github.com/stephenfairchild/relay/blob/main/CHANGELOG.md)

## Other Installation Methods

### Binary Installation

Download the latest release from [GitHub Releases](https://github.com/stephenfairchild/relay/releases):

```bash
curl -L https://github.com/stephenfairchild/relay/releases/latest/download/relay-$(uname -s)-$(uname -m) -o relay
chmod +x relay
sudo mv relay /usr/local/bin/
```

### Build from Source

```bash
git clone https://github.com/stephenfairchild/relay.git
cd relay
cargo build --release
sudo cp target/release/relay /usr/local/bin/
```

See the [full installation guide](installation.md) for more details.
