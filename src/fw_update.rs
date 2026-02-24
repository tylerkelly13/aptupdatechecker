//! Firmware update checking and notification
//!
//! Handles checking for available firmware updates using the
//! fwupd daemon (`fwupdmgr`). Refreshes the firmware metadata cache
//! and queries for available updates, then sends notifications with the results.

use crate::common::{App, NotificationType, notify, notify_error};
use regex::Regex;
use std::process::Command;

/// Returns the regex pattern for matching firmware update device IDs.
///
/// # Returns
///
/// A compiled regex pattern that matches "Device ID: <identifier>" lines
/// in fwupdmgr output.
fn get_device_id_pattern() -> Regex {
    Regex::new(r"'Device ID:\s+\w+'").unwrap()
}

/// Counts the number of available firmware updates from fwupdmgr output.
///
/// # Arguments
///
/// * `output` - The stdout from `fwupdmgr get-updates`
///
/// # Returns
///
/// The number of devices with available firmware updates.
///
/// # Examples
///
/// ```
/// # use aptupdatechecker::fw_update::count_firmware_updates;
/// let output = "'Device ID: 12345'\n'Device ID: 67890'";
/// assert_eq!(count_firmware_updates(output), 2);
/// ```
pub fn count_firmware_updates(output: &str) -> usize {
    let pattern = get_device_id_pattern();
    pattern.find_iter(output).count()
}

/// Formats a firmware update notification message.
///
/// # Arguments
///
/// * `count` - The number of available firmware updates
///
/// # Returns
///
/// A formatted message string describing the updates and how to install them.
///
/// # Examples
///
/// ```
/// # use aptupdatechecker::fw_update::format_firmware_message;
/// assert_eq!(
///     format_firmware_message(2),
///     "2 firmware updates are available.\nRun 'fwupdmgr update' to install them."
/// );
/// ```
pub fn format_firmware_message(count: usize) -> String {
    format!(
        "{} firmware updates are available.\nRun 'fwupdmgr update' to install them.",
        count
    )
}

/// Refreshes firmware metadata and checks for available updates.
///
/// Main entry point for firmware update checking. Performs
/// a two-step process:
/// 1. Refreshes the firmware metadata cache (through `fwupdmgr refresh --force`)
/// 2. Queries for available firmware updates (through `fwupdmgr get-updates`)
///
/// # Notifications
///
/// - When updates are available, sends an informational notification with the count
/// - When errors occur, sends error notifications with details
/// - When no updates are available, prints to stdout (no notification)
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
/// refreshes regardless of last update time.
///
/// # Returns
///
/// * `true` - Cache refresh completed successfully
/// * `false` - Cache refresh failed (error notification sent)
///
/// # Panics
///
/// Panics when executing the `fwupdmgr` command fails (for example, not installed
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

/// Checks for available firmware updates and sends notifications.
///
/// Executes `fwupdmgr get-updates` to query for available firmware updates.
/// Parses the output using a regex pattern to count how many devices have
/// updates available.
///
/// # Notification Behavior
///
/// - Sends an info notification when one or more firmware updates are available
/// - Sends an error notification when the check fails
/// - Prints to stdout when no updates are available (no notification)
///
/// # Implementation Details
///
/// Uses a regex pattern to match "Device ID:" lines in the output, which
/// correspond to devices with available updates.
/// `fwupdmgr` does not provide machine-readable output.
///
/// # Panics
///
/// Panics when the `fwupdmgr` command fails.
fn check_for_fw_updates() {
    let output = Command::new("fwupdmgr")
        .arg("get-updates")
        .output()
        .expect("Failed to execute fwupdmgr get-updates command");

    // fwupdmgr exits with code 0 when updates are available, code 2 when no updates available
    let exit_code = output.status.code().unwrap_or(1);

    if output.status.success() || exit_code == 2 {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let match_count = count_firmware_updates(&stdout);
        if match_count > 0 {
            let message = format_firmware_message(match_count);
            notify(
                NotificationType::Info,
                App::Fwupd,
                "Firmware Updates Available",
                &message,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_firmware_updates_none() {
        let output = "No updates available";
        assert_eq!(count_firmware_updates(output), 0);
    }

    #[test]
    fn test_count_firmware_updates_single() {
        let output = "Some text\n'Device ID: abc123'\nMore text";
        assert_eq!(count_firmware_updates(output), 1);
    }

    #[test]
    fn test_count_firmware_updates_multiple() {
        let output = "'Device ID: abc123'\n'Device ID: def456'\n'Device ID: ghi789'";
        assert_eq!(count_firmware_updates(output), 3);
    }

    #[test]
    fn test_count_firmware_updates_empty() {
        let output = "";
        assert_eq!(count_firmware_updates(output), 0);
    }

    #[test]
    fn test_count_firmware_updates_with_whitespace() {
        let output = "'Device ID:   xyz999'";
        assert_eq!(count_firmware_updates(output), 1);
    }

    #[test]
    fn test_count_firmware_updates_mixed_content() {
        let output = r#"
            Firmware updates available:
            'Device ID: device1'
            Some other text
            'Device ID: device2'
            Random content
        "#;
        assert_eq!(count_firmware_updates(output), 2);
    }

    #[test]
    fn test_format_firmware_message_single() {
        let msg = format_firmware_message(1);
        assert_eq!(
            msg,
            "1 firmware updates are available.\nRun 'fwupdmgr update' to install them."
        );
    }

    #[test]
    fn test_format_firmware_message_multiple() {
        let msg = format_firmware_message(5);
        assert_eq!(
            msg,
            "5 firmware updates are available.\nRun 'fwupdmgr update' to install them."
        );
        assert!(msg.contains("5"));
    }

    #[test]
    fn test_format_firmware_message_zero() {
        let msg = format_firmware_message(0);
        assert_eq!(
            msg,
            "0 firmware updates are available.\nRun 'fwupdmgr update' to install them."
        );
    }

    #[test]
    fn test_format_firmware_message_contains_instruction() {
        for count in [0, 1, 5, 100] {
            let msg = format_firmware_message(count);
            assert!(msg.contains("Run 'fwupdmgr update' to install them."));
        }
    }

    #[test]
    fn test_format_firmware_message_multiline() {
        let msg = format_firmware_message(3);
        assert!(msg.contains('\n'));
        let lines: Vec<&str> = msg.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_format_firmware_message_count_in_message() {
        for count in [1, 2, 10, 50] {
            let msg = format_firmware_message(count);
            assert!(msg.contains(&count.to_string()));
        }
    }

    #[test]
    fn test_get_device_id_pattern_valid() {
        let pattern = get_device_id_pattern();
        assert!(pattern.is_match("'Device ID: abc123'"));
        assert!(pattern.is_match("'Device ID:   xyz'"));
    }

    #[test]
    fn test_get_device_id_pattern_invalid() {
        let pattern = get_device_id_pattern();
        assert!(!pattern.is_match("Device ID: abc123")); // Missing quotes
        assert!(!pattern.is_match("'Device ID: '")); // Missing identifier
        assert!(!pattern.is_match("Random text"));
    }

    #[test]
    fn test_device_id_pattern_consistency() {
        // The pattern should match the same format used in real fwupdmgr output
        let sample_output = "'Device ID: 12a3b4c5'";
        assert_eq!(count_firmware_updates(sample_output), 1);
    }
}
