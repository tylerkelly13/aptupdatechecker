//! Common notification utilities
//!
//! Provides shared types and functions for sending desktop notifications
//! using the freedesktop notification system through notify-rust.

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

/// Returns the icon name for a notification type.
///
/// # Arguments
///
/// * `notification_type` - The type of notification
///
/// # Returns
///
/// * `"info"` for Info notifications
/// * `"error"` for Error notifications
pub fn get_icon(notification_type: &NotificationType) -> &'static str {
    match notification_type {
        NotificationType::Info => "info",
        NotificationType::Error => "error",
    }
}

/// Returns the urgency level for a notification type.
///
/// # Arguments
///
/// * `notification_type` - The type of notification
///
/// # Returns
///
/// * `Urgency::Normal` for Info notifications
/// * `Urgency::Critical` for Error notifications
pub fn get_urgency(notification_type: &NotificationType) -> Urgency {
    match notification_type {
        NotificationType::Info => Urgency::Normal,
        NotificationType::Error => Urgency::Critical,
    }
}

/// Returns the application name for a given app context.
///
/// # Arguments
///
/// * `app` - The application context
///
/// # Returns
///
/// * `"Apt Update Checker"` for Apt
/// * `"Firmware Update Checker"` for Fwupd
pub fn get_appname(app: &App) -> &'static str {
    match app {
        App::Apt => "Apt Update Checker",
        App::Fwupd => "Firmware Update Checker",
    }
}

/// Returns the notification timeout in milliseconds.
///
/// # Returns
///
/// 30000 milliseconds (30 seconds)
pub const fn get_timeout() -> i32 {
    30000
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
    let icon = get_icon(&notification_type);
    let urgency = get_urgency(&notification_type);
    let appname = get_appname(&app);
    let timeout = get_timeout();

    if let Err(e) = Notification::new()
        .summary(title)
        .appname(appname)
        .body(body)
        .icon(icon)
        .timeout(timeout)
        .urgency(urgency)
        .hint(Hint::Category("system".to_string()))
        .show()
    {
        eprintln!("Failed to send notification: {}", e);
    }
}

/// Sends an error notification with critical urgency.
///
/// Convenience wrapper around [`notify`] that always uses
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_icon_info() {
        assert_eq!(get_icon(&NotificationType::Info), "info");
    }

    #[test]
    fn test_get_icon_error() {
        assert_eq!(get_icon(&NotificationType::Error), "error");
    }

    #[test]
    fn test_get_urgency_info() {
        assert_eq!(get_urgency(&NotificationType::Info), Urgency::Normal);
    }

    #[test]
    fn test_get_urgency_error() {
        assert_eq!(get_urgency(&NotificationType::Error), Urgency::Critical);
    }

    #[test]
    fn test_get_appname_apt() {
        assert_eq!(get_appname(&App::Apt), "Apt Update Checker");
    }

    #[test]
    fn test_get_appname_fwupd() {
        assert_eq!(get_appname(&App::Fwupd), "Firmware Update Checker");
    }

    #[test]
    fn test_get_timeout() {
        assert_eq!(get_timeout(), 30000);
    }

    #[test]
    fn test_icon_and_urgency_consistency() {
        // Info notifications should have normal urgency
        let info_icon = get_icon(&NotificationType::Info);
        let info_urgency = get_urgency(&NotificationType::Info);
        assert_eq!(info_icon, "info");
        assert_eq!(info_urgency, Urgency::Normal);

        // Error notifications should have critical urgency
        let error_icon = get_icon(&NotificationType::Error);
        let error_urgency = get_urgency(&NotificationType::Error);
        assert_eq!(error_icon, "error");
        assert_eq!(error_urgency, Urgency::Critical);
    }

    #[test]
    fn test_appname_uniqueness() {
        // Each app should have a distinct name
        let apt_name = get_appname(&App::Apt);
        let fwupd_name = get_appname(&App::Fwupd);
        assert_ne!(apt_name, fwupd_name);
    }

    #[test]
    fn test_appname_non_empty() {
        assert!(!get_appname(&App::Apt).is_empty());
        assert!(!get_appname(&App::Fwupd).is_empty());
    }

    #[test]
    fn test_icon_non_empty() {
        assert!(!get_icon(&NotificationType::Info).is_empty());
        assert!(!get_icon(&NotificationType::Error).is_empty());
    }

    #[test]
    fn test_timeout_positive() {
        assert!(get_timeout() > 0);
    }
}
