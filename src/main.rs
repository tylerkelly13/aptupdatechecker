//! APT Update Checker
//!
//! A system service that checks for available APT package updates and firmware updates,
//! then sends desktop notifications to inform the user.
//!
//! This application runs two main checks:
//! - APT package updates via `apt update` and cache inspection
//! - Firmware updates via `fwupdmgr`
//!
//! Notifications are sent using the freedesktop notification system.

mod apt_update;
mod common;
mod fw_update;

/// Main entry point for the application.
///
/// Executes both APT and firmware update checks sequentially.
/// Each check will send a desktop notification if updates are available
/// or if errors occur during the check.
#[cfg(not(tarpaulin_include))]
fn main() {
    apt_update::update_and_check();
    fw_update::update_and_check();
}
