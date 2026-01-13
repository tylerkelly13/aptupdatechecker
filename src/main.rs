//! APT Update Checker
//!
//! A system service that checks for available APT package updates and firmware updates,
//! then sends desktop notifications.
//!
//! The application supports multiple modes:
//! - `update`: Updates the APT cache (requires root, no notifications)
//! - `check`: Checks for updates and sends notifications (user mode)
//! - `apt`: Legacy mode - updates cache and checks (requires root + D-Bus)
//! - `fwupd`: Checks for firmware updates
//!
//! Notifications are sent using the freedesktop notification system.

use std::env;

mod apt_update;
mod common;
mod fw_update;

/// Main entry point for the application.
///
/// Parses command line arguments to determine the operation mode.
/// Supports separate update and check operations for privilege separation.
#[cfg(not(tarpaulin_include))]
fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("update") => {
            // Root mode: just update the APT cache, no notifications
            apt_update::update_cache_only();
        }
        Some("check") => {
            // User mode: check for APT updates and send notifications
            apt_update::check_only();
        }
        Some("apt") => {
            // Legacy mode: update cache and check (requires root + D-Bus access)
            apt_update::update_and_check();
        }
        Some("fwupd") => {
            // Firmware update check
            fw_update::update_and_check();
        }
        _ => {
            // Default: run both checks (legacy behavior)
            apt_update::update_and_check();
            fw_update::update_and_check();
        }
    }
}
