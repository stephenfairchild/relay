# Quick Start

Get Relay up and running in minutes with Docker.

## 1. Create a Configuration File

Create a `config.toml` file:

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

> **Note:** Use `host.docker.internal` to connect to services running on your host machine, or replace with your actual backend URL.

## 2. Run Relay with Docker

```bash
docker run -p 8080:8080 \
  -v $(pwd)/config.toml:/etc/relay/config.toml \
  ghcr.io/stephenfairchild/relay:latest
```

Relay will start listening on `http://localhost:8080`.

## 3. Test It Out

```bash
# Make a request through Relay
curl http://localhost:8080/api/data

# Check cache headers
curl -I http://localhost:8080/api/data
```

## 4. Configure Cache Rules

Add specific rules for different paths in your `config.toml`:

```toml
[cache.rules]
"/api/*" = { ttl = "30s", stale = "5m" }
"/static/*" = { ttl = "1d" }
"/*.jpg" = { ttl = "7d" }
"/*.png" = { ttl = "7d" }
```

Restart the container to apply changes:

```bash
docker restart <container-id>
```

## Alternative: Run Without Docker

If you have the Relay binary installed:

```bash
relay --config config.toml
```

## Next Steps

- [Advanced Docker setup with Redis](docker.md)
- [Learn about configuration options](configuration.md)
- [Set up monitoring](monitoring.md)
