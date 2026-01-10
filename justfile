# List available commands
default:
    @just --list

# Run all CI checks (check + clippy + fmt + test)
ci:
    cargo check
    cargo clippy -- -D warnings
    cargo fmt -- --check
    cargo test

# Run compiler checks without building
check:
    cargo check

# Run clippy linter with warnings as errors
clippy:
    cargo clippy -- -D warnings

# Format all code
fmt:
    cargo fmt

# Check code formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Run all tests
test:
    cargo test

# Run tests with output visible
test-verbose:
    cargo test -- --nocapture

# Build release binary
build:
    cargo build --release

# Install binary locally to ~/.cargo/bin
install:
    cargo install --path .

# Clean build artifacts
clean:
    cargo clean

