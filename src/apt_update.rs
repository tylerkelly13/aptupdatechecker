//! APT package update checking and notification
//!
//! This module handles checking for available APT package updates by:
//! 1. Updating the APT package cache (equivalent to `apt update`)
//! 2. Querying the cache for upgradable packages
//! 3. Sending desktop notifications when updates are available or errors occur

use crate::common::{App, NotificationType, notify, notify_error};
use rust_apt::cache::PackageSort;
use rust_apt::new_cache;
use rust_apt::progress::AcquireProgress;

/// Checks for available APT package updates and notifies the user.
///
/// This function performs a two-step process:
/// 1. Updates the APT package cache to get the latest package information
/// 2. Counts how many packages can be upgraded
///
/// # Notifications
///
/// - If updates are available, sends an informational notification with the count
/// - If errors occur, sends error notifications with details
/// - If no updates are available, prints to stdout (no notification)
///
/// # Errors
///
/// Errors during cache initialization or update operations result in error
/// notifications being sent to the user. The function returns early on errors.
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
        let message = if upgrade_count == 1 {
            "1 software upgrade available\nRun `apt upgrade` to install".to_string()
        } else {
            format!(
                "{} software upgrades available\nRun `apt upgrade` to install",
                upgrade_count
            )
        };

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
/// This function refreshes the package lists from all configured repositories,
/// similar to running `apt update` on the command line.
///
/// # Returns
///
/// * `true` - Cache update completed successfully
/// * `false` - Cache update failed (error notification sent to user)
///
/// # Behavior
///
/// Uses the APT progress handler to track the update operation. Any errors
/// encountered during the update are filtered and presented to the user via
/// an error notification.
///
/// # Errors
///
/// Errors are handled internally by sending notifications. This function does
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
