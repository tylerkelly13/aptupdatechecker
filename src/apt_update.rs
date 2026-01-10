//! APT package update checking and notification
//!
//! Handles checking for available APT package updates through:
//! 1. Updating the APT package cache (equivalent to `apt update`)
//! 2. Querying the cache for upgradable packages
//! 3. Sending desktop notifications when updates are available or errors occur

use crate::common::{App, NotificationType, notify, notify_error};
use rust_apt::cache::PackageSort;
use rust_apt::new_cache;
use rust_apt::progress::AcquireProgress;

/// Formats an update message based on the number of available updates.
///
/// # Arguments
///
/// * `count` - The number of available updates
///
/// # Returns
///
/// A formatted message string describing the updates and how to install them.
/// Uses singular form for 1 update, plural for multiple updates.
///
/// # Examples
///
/// ```
/// # use aptupdatechecker::apt_update::format_update_message;
/// assert_eq!(
///     format_update_message(1),
///     "1 software upgrade available\nRun `apt upgrade` to install"
/// );
/// assert_eq!(
///     format_update_message(5),
///     "5 software upgrades available\nRun `apt upgrade` to install"
/// );
/// ```
pub fn format_update_message(count: usize) -> String {
    if count == 1 {
        "1 software upgrade available\nRun `apt upgrade` to install".to_string()
    } else {
        format!(
            "{} software upgrades available\nRun `apt upgrade` to install",
            count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_update_message_singular() {
        let msg = format_update_message(1);
        assert_eq!(
            msg,
            "1 software upgrade available\nRun `apt upgrade` to install"
        );
        assert!(msg.contains("upgrade")); // singular form
        assert!(!msg.contains("upgrades")); // not plural
    }

    #[test]
    fn test_format_update_message_plural() {
        let msg = format_update_message(2);
        assert_eq!(
            msg,
            "2 software upgrades available\nRun `apt upgrade` to install"
        );
        assert!(msg.contains("upgrades")); // plural form
    }

    #[test]
    fn test_format_update_message_zero() {
        let msg = format_update_message(0);
        assert_eq!(
            msg,
            "0 software upgrades available\nRun `apt upgrade` to install"
        );
    }

    #[test]
    fn test_format_update_message_large_count() {
        let msg = format_update_message(9999);
        assert!(msg.contains("9999"));
        assert!(msg.contains("upgrades"));
        assert!(msg.contains("Run `apt upgrade` to install"));
    }

    #[test]
    fn test_format_update_message_contains_instruction() {
        // All messages should contain installation instructions
        for count in [0, 1, 5, 100] {
            let msg = format_update_message(count);
            assert!(msg.contains("Run `apt upgrade` to install"));
        }
    }

    #[test]
    fn test_format_update_message_multiline() {
        // Messages should be multiline with newline separator
        let msg = format_update_message(5);
        assert!(msg.contains('\n'));
        let lines: Vec<&str> = msg.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_format_update_message_count_in_message() {
        // The count should appear in the message
        for count in [1, 2, 10, 50, 100] {
            let msg = format_update_message(count);
            assert!(msg.contains(&count.to_string()));
        }
    }
}

/// Checks for available APT package updates and sends notifications.
///
/// Performs a two-step process:
/// 1. Updates the APT package cache to get the latest package information
/// 2. Counts how many packages can be upgraded
///
/// # Notifications
///
/// - When updates are available, sends an informational notification with the count
/// - When errors occur, sends error notifications with details
/// - When no updates are available, prints to stdout (no notification)
///
/// # Errors
///
/// Errors during cache initialization or update operations result in error
/// notifications being sent. The function returns on errors.
pub fn update_and_check() {
    // Update the APT cache
    if !update_apt_cache() {
        return;
    }

    // Create a cache for checking upgrades
    let cache = match new_cache!() {
        Ok(cache) => cache,
        Err(e) => {
            notify_error(
                App::Apt,
                "APT Cache Initialization Failed",
                &format!("Failed to initialize APT cache: {}", e),
            );
            return;
        }
    };

    // Create a sort that filters only upgradable packages
    let sort = PackageSort::default().upgradable();

    // Count upgradable packages
    let upgrade_count = cache.packages(&sort).count();

    if upgrade_count > 0 {
        let message = format_update_message(upgrade_count);

        notify(
            NotificationType::Info,
            App::Apt,
            "Software Updates Available",
            &message,
        );
    } else {
        println!("No software updates available.");
    }
}

/// Updates the APT package cache (equivalent to `apt update`).
///
/// Refreshes the package lists from all configured repositories,
/// similar to running `apt update` on the command line.
///
/// # Returns
///
/// * `true` - Cache update completed successfully
/// * `false` - Cache update failed (error notification sent)
///
/// # Behavior
///
/// Uses the APT progress handler to track the update operation. Errors
/// encountered during the update are filtered and presented through
/// an error notification.
///
/// # Errors
///
/// Errors are handled internally by sending notifications. The function does
/// not panic or propagate errors to the caller.
fn update_apt_cache() -> bool {
    // Create a cache
    let cache = match new_cache!() {
        Ok(cache) => cache,
        Err(e) => {
            notify_error(
                App::Apt,
                "APT Cache Initialization Failed",
                &format!("Failed to initialize APT cache: {}", e),
            );
            return false;
        }
    };

    // Create a progress handler
    let mut progress = AcquireProgress::apt();

    // Update the package lists
    if let Err(e) = cache.update(&mut progress) {
        let error_msgs: Vec<String> = e
            .iter()
            .filter(|err| err.is_error)
            .map(|err| err.msg.clone())
            .collect();

        notify_error(
            App::Apt,
            "APT Update Failed",
            &format!("Failed to update package lists: {}", error_msgs.join(", ")),
        );
        false
    } else {
        true
    }
}
