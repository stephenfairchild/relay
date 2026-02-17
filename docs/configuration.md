# Configuration

Relay uses TOML for configuration. All options are documented here.

## Configuration File

By default, Relay looks for `config.toml` in the current directory. You can specify a different path:

```bash
relay --config /path/to/config.toml
```

## Upstream Configuration

Define your origin server:

```toml
[upstream]
url = "http://localhost:8000"
timeout = "30s"  # Optional: request timeout
```

## Cache Configuration

### Default Settings

```toml
[cache]
default_ttl = "5m"              # Default cache lifetime
stale_while_revalidate = "1h"   # Serve stale while fetching fresh
stale_if_error = "24h"          # Serve stale if backend is down
```

### Time Format

Time values support these units:
- `s` - seconds
- `m` - minutes
- `h` - hours
- `d` - days

Examples: `30s`, `5m`, `2h`, `7d`

## Server Configuration

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4  # Number of worker threads
```

## Next Steps

- [Configure cache rules](cache-rules.md)
- [Set up storage backends](storage.md)
