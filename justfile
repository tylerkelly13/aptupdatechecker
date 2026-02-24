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

# Set up vendored rust-apt from cargo registry cache or crates.io
setup-rust-apt:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -f vendor/rust-apt/Cargo.toml ]; then
        exit 0
    fi
    echo "Setting up vendor/rust-apt..."
    mkdir -p vendor/rust-apt
    RUST_APT_DIR=$(ls -d ~/.cargo/registry/src/index.crates.io-*/rust-apt-0.8.0/ 2>/dev/null | head -1 || true)
    if [ -n "${RUST_APT_DIR}" ]; then
        cp -r "${RUST_APT_DIR}"/* vendor/rust-apt/
    else
        echo "Downloading rust-apt 0.8.0 from crates.io..."
        TMPTAR=$(mktemp /tmp/rust-apt-XXXXX.tar.gz)
        curl -fsSL "https://crates.io/api/v1/crates/rust-apt/0.8.0/download" -o "$TMPTAR"
        tar -xz -f "$TMPTAR" -C /tmp
        cp -r /tmp/rust-apt-0.8.0/* vendor/rust-apt/
        rm -rf "$TMPTAR" /tmp/rust-apt-0.8.0
    fi
    echo "vendor/rust-apt set up successfully"

# Vendor dependencies for Debian packaging
vendor-deps: setup-rust-apt
    cargo vendor --versioned-dirs

# Build Debian package
deb: setup-rust-apt
    cargo deb

