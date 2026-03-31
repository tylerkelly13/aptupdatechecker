//! Build script for aptupdatechecker

fn main() {
    // Register the tarpaulin_include cfg for code coverage
    println!("cargo::rustc-check-cfg=cfg(tarpaulin_include)");
}
