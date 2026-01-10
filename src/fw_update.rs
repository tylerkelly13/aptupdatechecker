//! Firmware update checking and notification
//!
//! This module handles checking for available firmware updates using the
//! fwupd daemon (`fwupdmgr`). It refreshes the firmware metadata cache
//! and queries for available updates, then notifies the user of the results.

use crate::common::{App, NotificationType, notify, notify_error};
use regex::Regex;
use std::process::Command;

/// Refreshes firmware metadata and checks for available updates.
///
/// This is the main entry point for firmware update checking. It performs
/// a two-step process:
/// 1. Refreshes the firmware metadata cache (via `fwupdmgr refresh --force`)
/// 2. Queries for available firmware updates (via `fwupdmgr get-updates`)
///
/// # Notifications
///
/// - If updates are available, sends an informational notification with the count
/// - If errors occur, sends error notifications with details
/// - If no updates are available, prints to stdout (no notification)
///
/// # Dependencies
///
/// Requires `fwupdmgr` to be installed and accessible in the system PATH.
pub fn update_and_check() {
    if update_fw_update_cache() {
        check_for_fw_updates();
    }
}

/// Refreshes the firmware update metadata cache.
///
/// Executes `fwupdmgr refresh --force` to download the latest firmware
/// metadata from configured remotes. The `--force` flag ensures the cache
/// is refreshed even if it was recently updated.
///
/// # Returns
///
/// * `true` - Cache refresh completed successfully
/// * `false` - Cache refresh failed (error notification sent to user)
///
/// # Panics
///
/// Panics if the `fwupdmgr` command cannot be executed (e.g., not installed
/// or not in PATH).
fn update_fw_update_cache() -> bool {
    let output = Command::new("fwupdmgr")
        .arg("refresh")
        .arg("--force")
        .output()
        .expect("Failed to execute fwupdmgr refresh command");

    if output.status.success() {
        true
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        notify_error(
            App::Fwupd,
            "Firmware Update Cache Refresh Failed",
            &format!("Failed to refresh firmware update cache: {}", error_message),
        );
        false
    }
}

/// Checks for available firmware updates and notifies the user.
///
/// Executes `fwupdmgr get-updates` to query for available firmware updates.
/// Parses the output using a regex pattern to count how many devices have
/// updates available.
///
/// # Notification Behavior
///
/// - Sends an info notification if one or more firmware updates are available
/// - Sends an error notification if the check fails
/// - Prints to stdout if no updates are available (no notification)
///
/// # Implementation Details
///
/// Uses a regex pattern to match "Device ID:" lines in the output, which
/// correspond to devices with available updates. This is a heuristic
/// approach since `fwupdmgr` doesn't provide machine-readable output.
///
/// # Panics
///
/// Panics if the `fwupdmgr` command cannot be executed.
fn check_for_fw_updates() {
    let re = Regex::new(r"'Device ID:\s+\w+'").unwrap();

    let output = Command::new("fwupdmgr")
        .arg("get-updates")
        .output()
        .expect("Failed to execute fwupdmgr get-updates command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let match_count = re.find_iter(&stdout).count();
        if match_count > 0 {
            notify(
                NotificationType::Info,
                App::Fwupd,
                "Firmware Updates Available",
                &format!(
                    "{} firmware updates are available.\nRun 'fwupdmgr update' to install them.",
                    match_count
                ),
            );
        } else {
            println!("No firmware updates available.");
        }
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        notify_error(
            App::Fwupd,
            "Firmware Update Check Failed",
            &format!("Failed to check for firmware updates: {}", error_message),
        );
    }
}
