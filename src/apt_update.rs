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
use std::fs;
use std::time::{Duration, SystemTime};

/// Maximum age (in hours) for the APT cache to be considered fresh.
const APT_CACHE_MAX_AGE_HOURS: u64 = 8;

/// Path to the APT package cache binary, regenerated on each `apt update`.
const APT_PKGCACHE_PATH: &str = "/var/cache/apt/pkgcache.bin";

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
    if !update_apt_cache(true) {
        return;
    }

    // Check for updates and notify
    check_for_updates();
}

/// Updates the APT cache only, without checking for updates or sending notifications.
///
/// This function is designed to run as root via a system service. It updates
/// the package lists but does not send any notifications.
///
/// # Errors
///
/// Errors are printed to stderr. The function does not send notifications.
pub fn update_cache_only() {
    if !update_apt_cache(false) {
        eprintln!("Failed to update APT cache");
        std::process::exit(1);
    } else {
        println!("APT cache updated successfully");
    }
}

/// Checks for available updates and sends notifications without updating the cache.
///
/// This function is designed to run as a regular user. It reads the existing
/// APT cache (which should have been updated by a separate root process) and
/// sends desktop notifications when updates are available.
///
/// # Notifications
///
/// - When updates are available, sends an informational notification with the count
/// - When errors occur, sends error notifications with details
/// - When no updates are available, prints to stdout (no notification)
pub fn check_only() {
    if !is_apt_cache_fresh() {
        println!("APT cache is stale, skipping update check.");
        return;
    }
    check_for_updates();
}

/// Checks whether the APT cache has been updated recently.
///
/// Returns `true` if the cache file at [`APT_PKGCACHE_PATH`] was modified
/// within the last [`APT_CACHE_MAX_AGE_HOURS`] hours, `false` otherwise.
fn is_apt_cache_fresh() -> bool {
    let max_age = Duration::from_secs(APT_CACHE_MAX_AGE_HOURS * 3600);

    let metadata = match fs::metadata(APT_PKGCACHE_PATH) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let modified = match metadata.modified() {
        Ok(t) => t,
        Err(_) => return false,
    };

    match SystemTime::now().duration_since(modified) {
        Ok(age) => age < max_age,
        Err(_) => true, // Modified time is in the future; treat as fresh
    }
}

/// Checks for upgradable packages and sends notifications.
///
/// Reads the APT cache and counts upgradable packages. Does not update the cache.
///
/// # Notifications
///
/// - When updates are available, sends an informational notification
/// - When errors occur, sends error notifications
/// - When no updates are available, prints to stdout
fn check_for_updates() {
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

/// Updates the APT package cache (runs `apt update`).
///
/// Refreshes the package lists from all configured repositories.
///
/// # Arguments
///
/// * `send_notifications` - Whether to send desktop notifications on errors
///
/// # Returns
///
/// * `true` - Cache update completed
/// * `false` - Cache update failed
///
/// # Behavior
///
/// When `send_notifications` is true, errors trigger desktop notifications.
/// When false, errors are printed to stderr.
fn update_apt_cache(send_notifications: bool) -> bool {
    // Create a cache
    let cache = match new_cache!() {
        Ok(cache) => cache,
        Err(e) => {
            if send_notifications {
                notify_error(
                    App::Apt,
                    "APT Cache Initialization Failed",
                    &format!("Failed to initialize APT cache: {}", e),
                );
            } else {
                eprintln!("Failed to initialize APT cache: {}", e);
            }
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

        let error_text = format!("Failed to update package lists: {}", error_msgs.join(", "));

        if send_notifications {
            notify_error(App::Apt, "APT Update Failed", &error_text);
        } else {
            eprintln!("{}", error_text);
        }
        false
    } else {
        true
    }
}
