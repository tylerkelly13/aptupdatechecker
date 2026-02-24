//! Build script for aptupdatechecker
//!
//! This build script handles two main tasks:
//! 1. Configures C++ compilation settings for the rust-apt dependency
//! 2. Applies runtime patches to vendored dependencies for libapt-pkg 3.x compatibility

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Set C++17 standard for rust-apt dependency
    // Newer versions of libapt-pkg require C++17 features like std::string_view
    println!("cargo:rustc-env=CXXFLAGS=-std=c++17");

    // Register the tarpaulin_include cfg for code coverage
    println!("cargo::rustc-check-cfg=cfg(tarpaulin_include)");

    // Ensure vendored rust-apt has the necessary patches applied
    ensure_rust_apt_patches();
}

/// Ensures the vendored rust-apt dependency has compatibility patches applied.
///
/// Automatically patches the vendored rust-apt crate to work with
/// libapt-pkg 3.x (Debian testing/Trixie and later). Patches are idempotent
/// and only apply when needed.
///
/// # Patches Applied
///
/// 1. **libapt-pkg 3.x API compatibility**: Replaces `APT::StringView` with
///    `std::string_view` in `apt-pkg-c/cache.h` since the APT namespace wrapper
///    was removed in libapt-pkg 3.x
///
/// 2. **Lint configuration**: Adds lint allowances to `Cargo.toml` to suppress
///    warnings from the vendored code that would fail CI with `-D warnings`
///
/// # Build System Integration
///
/// Registers the patched files with cargo's change tracking system, so the build
/// script reruns if these files are modified externally.
fn ensure_rust_apt_patches() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let vendor_dir = PathBuf::from(&manifest_dir).join("vendor/rust-apt");

    // Tell cargo to rerun if vendor files change
    println!(
        "cargo:rerun-if-changed={}",
        vendor_dir.join("apt-pkg-c/cache.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        vendor_dir.join("Cargo.toml").display()
    );

    // Check if patches need to be applied
    let cache_h = vendor_dir.join("apt-pkg-c/cache.h");
    let cargo_toml = vendor_dir.join("Cargo.toml");

    // Apply libapt-pkg 3.x compatibility patch if needed
    if cache_h.exists()
        && let Ok(content) = fs::read_to_string(&cache_h)
        && content.contains("APT::StringView")
    {
        println!("cargo:warning=Applying libapt-pkg 3.x compatibility patch to cache.h");
        let patched = content.replace("APT::StringView", "std::string_view");
        fs::write(&cache_h, patched).expect("Failed to write patched cache.h");
    }

    // Apply lint configuration if needed
    if cargo_toml.exists()
        && let Ok(content) = fs::read_to_string(&cargo_toml)
        && !content.contains("[lints.rust]")
    {
        println!("cargo:warning=Adding lint configuration to rust-apt Cargo.toml");
        let mut updated = content;
        updated.push_str("\n[lints.rust]\n");
        updated.push_str("mismatched_lifetime_syntaxes = \"allow\"\n");
        updated.push_str("\n[lints.clippy]\n");
        updated.push_str("empty_line_after_doc_comments = \"allow\"\n");
        fs::write(&cargo_toml, updated).expect("Failed to write updated Cargo.toml");
    }
}
