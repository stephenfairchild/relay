# Quick Start

Get Relay up and running in minutes.

## 1. Create a Configuration File

Create a `config.toml` file:

```toml
[upstream]
url = "http://localhost:8000"

[cache]
default_ttl = "5m"
stale_while_revalidate = "1h"
stale_if_error = "24h"

[storage]
in_memory = true
```

## 2. Run Relay

```bash
relay --config config.toml
```

Relay will start listening on `http://localhost:8080` by default.

## 3. Test It Out

```bash
# Make a request through Relay
curl http://localhost:8080/api/data

# Check cache headers
curl -I http://localhost:8080/api/data
```

## 4. Configure Cache Rules

Add specific rules for different paths:

```toml
[cache.rules]
"/api/*" = { ttl = "30s", stale = "5m" }
"/static/*" = { ttl = "1d" }
"/*.jpg" = { ttl = "7d" }
"/*.png" = { ttl = "7d" }
```

## Next Steps

- [Learn about configuration options](configuration.md)
- [Set up monitoring](monitoring.md)
- [Deploy with Docker](docker.md)
