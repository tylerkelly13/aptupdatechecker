# Vendored Dependencies

This directory contains vendored copies of dependencies that require patches for compatibility.

## rust-apt

The `rust-apt` crate is vendored here with patches applied for compatibility with libapt-pkg 3.x (Debian testing/Trixie).

### Setup

The vendor directory is automatically set up by the build script (`build.rs`). The patches are applied automatically during the build process:

1. **libapt-pkg 3.x compatibility**: Replaces `APT::StringView` with `std::string_view` in `apt-pkg-c/cache.h`
2. **Lint configuration**: Adds lint allowances to `Cargo.toml` to suppress warnings from the vendored code

### Manual Setup

If you need to manually set up or reset the vendor directory:

```bash
# Remove existing vendor directory
rm -rf vendor

# Create fresh copy from cargo registry
mkdir -p vendor/rust-apt
cp -r ~/.cargo/registry/src/index.crates.io-*/rust-apt-0.8.0/* vendor/rust-apt/

# Build (patches will be applied automatically)
cargo build
```

### Patch Details

See [../rust-apt-libapt3.patch](../rust-apt-libapt3.patch) for the unified diff of changes.

The build script automatically detects if patches need to be applied and applies them idempotently.
