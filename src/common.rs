//! Common notification utilities
//!
//! Provides shared types and functions for sending desktop notifications
//! using the freedesktop notification system (via notify-rust).

use notify_rust::{Hint, Notification, Urgency};

/// The type of notification to display.
///
/// Determines the urgency level and icon used for the notification.
pub enum NotificationType {
    /// Informational notification (normal urgency)
    Info,
    /// Error notification (critical urgency)
    Error,
}

/// The application context for the notification.
///
/// Used to set the application name and customize notification appearance.
pub enum App {
    /// APT package manager update checker
    Apt,
    /// Firmware update checker (fwupd)
    Fwupd,
}

/// Sends a desktop notification with the specified parameters.
///
/// # Arguments
///
/// * `notification_type` - The type of notification (Info or Error)
/// * `app` - The application context (Apt or Fwupd)
/// * `title` - The notification title/summary
/// * `body` - The notification body text
///
/// # Notification Behavior
///
/// - Info notifications use normal urgency and the "info" icon
/// - Error notifications use critical urgency and the "error" icon
/// - All notifications have a 30-second timeout
/// - Notifications are categorized as "system" notifications
///
/// # Example
///
/// ```no_run
/// # use aptupdatechecker::common::{notify, NotificationType, App};
/// notify(
///     NotificationType::Info,
///     App::Apt,
///     "Updates Available",
///     "3 packages can be upgraded"
/// );
/// ```
pub fn notify(notification_type: NotificationType, app: App, title: &str, body: &str) {
    let icon = match notification_type {
        NotificationType::Info => "info",
        NotificationType::Error => "error",
    };
    let urgency = match notification_type {
        NotificationType::Info => Urgency::Normal,
        NotificationType::Error => Urgency::Critical,
    };
    let appname = match app {
        App::Apt => "Apt Update Checker",
        App::Fwupd => "Firmware Update Checker",
    };
    let timeout = 30000; // 30 seconds in milliseconds

    let _ = Notification::new()
        .summary(title)
        .appname(appname)
        .body(body)
        .icon(icon)
        .timeout(timeout)
        .urgency(urgency)
        .hint(Hint::Category("system".to_string()))
        .show();
}

/// Sends an error notification with critical urgency.
///
/// This is a convenience wrapper around [`notify`] that always uses
/// [`NotificationType::Error`].
///
/// # Arguments
///
/// * `app` - The application context (Apt or Fwupd)
/// * `title` - The error notification title
/// * `message` - The error message body
///
/// # Example
///
/// ```no_run
/// # use aptupdatechecker::common::{notify_error, App};
/// notify_error(
///     App::Apt,
///     "Update Failed",
///     "Failed to update package lists"
/// );
/// ```
pub fn notify_error(app: App, title: &str, message: &str) {
    notify(NotificationType::Error, app, title, message);
}
