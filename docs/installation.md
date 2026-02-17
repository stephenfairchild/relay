# Installation

## Docker (Recommended)

The easiest way to get started with Relay is using Docker:

```bash
# Pull the latest image
docker pull ghcr.io/stephenfairchild/relay:latest

# Run with config
docker run -p 8080:8080 \
  -v $(pwd)/config.toml:/etc/relay/config.toml \
  ghcr.io/stephenfairchild/relay:latest
```

For production deployments and advanced Docker setups, see the [Docker documentation](docker.md).

## Binary Installation

Download the latest release from [GitHub Releases](https://github.com/stephenfairchild/relay/releases):

```bash
# Download and install (Linux/macOS)
curl -L https://github.com/stephenfairchild/relay/releases/latest/download/relay-$(uname -s)-$(uname -m) -o relay
chmod +x relay
sudo mv relay /usr/local/bin/
```

Verify installation:

```bash
relay --version
```

## Build from Source

### Prerequisites

- Rust 1.70 or later
- Cargo

### Build Steps

```bash
# Clone the repository
git clone https://github.com/stephenfairchild/relay.git
cd relay

# Build release binary
cargo build --release

# The binary will be at target/release/relay
sudo cp target/release/relay /usr/local/bin/
```

Verify installation:

```bash
relay --version
```
