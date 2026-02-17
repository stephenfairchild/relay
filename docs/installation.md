# Installation

## Binary Installation

Download the latest release from [GitHub Releases](https://github.com/stephenfairchild/relay/releases):

```bash
# Download and install (Linux/macOS)
curl -L https://github.com/stephenfairchild/relay/releases/latest/download/relay-$(uname -s)-$(uname -m) -o relay
chmod +x relay
sudo mv relay /usr/local/bin/
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

## Docker

```bash
# Pull the image
docker pull ghcr.io/stephenfairchild/relay:latest

# Run with config
docker run -v $(pwd)/config.toml:/etc/relay/config.toml \
  -p 8080:8080 \
  ghcr.io/stephenfairchild/relay:latest
```

## Verify Installation

```bash
relay --version
```
