# Contributing

Thank you for your interest in contributing to Relay!

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- Cargo

### Setup

```bash
# Clone the repository
git clone https://github.com/stephenfairchild/relay.git
cd relay

# Build
cargo build

# Run tests
cargo test

# Run locally
cargo run -- --config config.toml
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/my-feature
# or
git checkout -b fix/my-bugfix
```

### 2. Make Changes

- Write code
- Add tests
- Update documentation

### 3. Test

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test
```

### 4. Format and Lint

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

### 5. Commit

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git commit -m "feat: add new cache eviction policy"
git commit -m "fix: resolve race condition in cache"
git commit -m "docs: update configuration examples"
```

### 6. Push and Create PR

```bash
git push origin feature/my-feature
```

Then create a Pull Request on GitHub.

## Code Style

- Follow Rust conventions
- Use `cargo fmt` for formatting
- Pass `cargo clippy` with no warnings
- Add documentation for public APIs
- Write tests for new features

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit() {
        // Test implementation
    }
}
```

### Integration Tests

```bash
cargo test --test integration_tests
```

### Performance Tests

```bash
cargo bench
```

## Documentation

- Update relevant docs in `docs/`
- Add code comments for complex logic
- Update README if needed
- Add examples for new features

## Pull Request Guidelines

1. **Title**: Clear and descriptive
2. **Description**: Explain what and why
3. **Tests**: Include tests for changes
4. **Documentation**: Update if needed
5. **Small PRs**: Keep changes focused

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How the changes were tested

## Checklist
- [ ] Tests pass
- [ ] Code formatted (cargo fmt)
- [ ] Lints pass (cargo clippy)
- [ ] Documentation updated
```

## Architecture

### Project Structure

```
relay/
├── src/
│   ├── main.rs          # Entry point
│   ├── config.rs        # Configuration
│   ├── cache/           # Cache implementation
│   ├── storage/         # Storage backends
│   ├── upstream/        # Upstream communication
│   └── metrics/         # Metrics and monitoring
├── tests/               # Integration tests
├── benches/             # Benchmarks
└── docs/                # Documentation
```

### Key Components

1. **Cache** - Core caching logic
2. **Storage** - Backend storage abstraction
3. **Upstream** - Communication with origin
4. **Metrics** - Prometheus metrics
5. **Config** - Configuration parsing

## Adding Features

### Example: New Storage Backend

1. Implement `Storage` trait:

```rust
pub trait Storage {
    async fn get(&self, key: &str) -> Result<Option<CachedResponse>>;
    async fn set(&self, key: &str, value: CachedResponse) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}
```

2. Add configuration:

```rust
#[derive(Deserialize)]
pub struct MyStorageConfig {
    pub url: String,
}
```

3. Write tests
4. Update documentation
5. Add example configuration

## Community

- **GitHub Issues**: Bug reports and feature requests
- **Discussions**: Questions and ideas
- **Discord**: (Coming soon)

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

## Questions?

Feel free to open an issue or discussion if you have questions!
