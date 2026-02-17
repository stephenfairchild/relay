# Relay

Relay is an open-source HTTP cache that brings simplicity to HTTP caching. You know how Caddy made reverse proxies so easy out of the box over nginx? This is what Relay does for HTTP caching. It makes it out of the box easy.

## What is Relay?

Relay sits between your users and your backend servers, storing (caching) responses to speed up your application and reduce load on your servers.

**The Problem:** Every time a user visits your website or app, your backend server has to do work - query databases, render pages, process data. When thousands of users make the same requests, your server does the same work over and over. This is slow and expensive.

**The Solution:** Relay saves responses from your backend and serves them directly to users. Instead of your server handling 10,000 requests, it might only handle 1 - and Relay serves the other 9,999 instantly from its cache.

**Real Benefits:**
- **Faster response times** - Cached responses are served in microseconds instead of milliseconds
- **Lower server costs** - Your backend handles a fraction of the traffic
- **Better reliability** - If your backend goes down, Relay can still serve cached content
- **Simple setup** - One binary, one config file, and you're running

## Features

- **HTTP/1 Support** - Full HTTP/1.1 protocol support
- **Stale-While-Revalidate** - Serve stale content while fetching fresh data
- **Stale-If-Error** - Keep your site up even when backends fail - automatically serve cached content during outages
- **Conditional Requests** - Support for If-None-Match and If-Modified-Since
- **Smart Cache Keys** - Ignore irrelevant query params and normalize cache keys
- **Flexible Invalidation** - TTL-based, pattern-based purging, and cache warming
- **Multiple Storage Backends** - In-memory, Redis, or disk storage
- **Prometheus Metrics** - Built-in monitoring with detailed cache metrics
- **Built with Rust** - Memory-safe and blazingly fast

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
