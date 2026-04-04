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

const APT_CACHE_MAX_AGE_HOURS: u64 = 8;
const APT_PKGCACHE_PATH: &str = "/var/cache/apt/pkgcache.bin";

/// Formats an update message with the count and installation instructions.
///
/// Uses singular form for 1 update, plural otherwise.
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

/// Updates the APT cache, then checks for upgradable packages and notifies.
///
/// Sends error notifications on failure, info notifications when updates
/// exist, and prints to stdout when none are available.
pub fn update_and_check() {
    if !update_apt_cache(true) {
        return;
    }

    check_for_updates();
}

/// Updates the APT cache without checking for updates or notifying.
///
/// Intended for the root system service. Prints errors to stderr
/// rather than sending notifications.
pub fn update_cache_only() {
    if !update_apt_cache(false) {
        eprintln!("Failed to update APT cache");
        std::process::exit(1);
    } else {
        println!("APT cache updated successfully");
        signal_user_sessions();
    }
}

/// Signals all active user sessions that the APT cache update completed.
///
/// Touches `/run/user/<uid>/apt-updates-available` for each session directory
/// found under `/run/user/`. Individual session failures do not abort the rest.
fn signal_user_sessions() {
    signal_sessions_in(std::path::Path::new("/run/user"));
}

/// Creates an `apt-updates-available` signal file in each subdirectory of `base_dir`.
///
/// Individual session failures print to stderr but do not abort the rest.
fn signal_sessions_in(base_dir: &std::path::Path) {
    let entries = match std::fs::read_dir(base_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to read {}: {}", base_dir.display(), e);
            return;
        }
    };
    for entry in entries.flatten() {
        let signal_file = entry.path().join("apt-updates-available");
        if let Err(e) = std::fs::File::create(&signal_file) {
            eprintln!("Failed to touch {}: {}", signal_file.display(), e);
        }
    }
}

/// Checks for upgradable packages and notifies, without updating the cache.
///
/// Intended for the unprivileged user service. Reads the existing cache
/// (updated by the root service) and skips the check if the cache is stale.
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
    is_cache_fresh(APT_PKGCACHE_PATH, APT_CACHE_MAX_AGE_HOURS)
}

/// Returns `true` if the file at `path` was modified within `max_age_hours`,
/// `false` if the file is missing, unreadable, or too old.
fn is_cache_fresh(path: &str, max_age_hours: u64) -> bool {
    let max_age = Duration::from_secs(max_age_hours * 3600);

    let metadata = match fs::metadata(path) {
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

/// Counts upgradable packages in the existing APT cache and notifies.
fn check_for_updates() {
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

    let sort = PackageSort::default().upgradable();
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

/// Refreshes APT package lists from all configured repositories.
///
/// - `send_notifications`: When `true`, errors trigger desktop notifications.
///   When `false`, errors print to stderr.
///
/// Returns `true` on success, `false` on failure.
fn update_apt_cache(send_notifications: bool) -> bool {
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

    let mut progress = AcquireProgress::apt();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_is_cache_fresh_recent_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cache.bin");
        std::fs::File::create(&path).unwrap();
        assert!(is_cache_fresh(path.to_str().unwrap(), 1));
    }

    #[test]
    fn test_is_cache_fresh_missing_file() {
        assert!(!is_cache_fresh("/nonexistent/path/cache.bin", 1));
    }

    #[test]
    fn test_is_cache_fresh_old_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cache.bin");
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(b"data").unwrap();
        // Set mtime to 10 hours ago
        let old_time = filetime::FileTime::from_system_time(
            SystemTime::now() - Duration::from_secs(10 * 3600),
        );
        filetime::set_file_mtime(&path, old_time).unwrap();
        assert!(!is_cache_fresh(path.to_str().unwrap(), 8));
    }

    #[test]
    fn test_is_cache_fresh_zero_max_age() {
        // A file created "now" should be stale when max_age_hours is 0,
        // because any non-zero age >= max_age (0 seconds) is false.
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cache.bin");
        std::fs::File::create(&path).unwrap();
        // max_age_hours=0 means max_age is 0 seconds, so age < 0s is always false
        assert!(!is_cache_fresh(path.to_str().unwrap(), 0));
    }

    #[test]
    fn test_is_cache_fresh_future_mtime() {
        // A file with mtime in the future should be treated as fresh
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cache.bin");
        std::fs::File::create(&path).unwrap();
        let future_time =
            filetime::FileTime::from_system_time(SystemTime::now() + Duration::from_secs(3600));
        filetime::set_file_mtime(&path, future_time).unwrap();
        assert!(is_cache_fresh(path.to_str().unwrap(), 1));
    }

    #[test]
    fn test_signal_sessions_in_creates_files() {
        let base = TempDir::new().unwrap();
        let user1 = base.path().join("1000");
        let user2 = base.path().join("1001");
        std::fs::create_dir(&user1).unwrap();
        std::fs::create_dir(&user2).unwrap();

        signal_sessions_in(base.path());

        assert!(user1.join("apt-updates-available").exists());
        assert!(user2.join("apt-updates-available").exists());
    }

    #[test]
    fn test_signal_sessions_in_missing_dir() {
        // Should not panic on missing directory
        signal_sessions_in(std::path::Path::new("/nonexistent/path"));
    }

    #[test]
    fn test_signal_sessions_in_unwritable_subdir() {
        use std::os::unix::fs::PermissionsExt;

        // Running as root bypasses POSIX permission checks, so we skip
        // this test unless run as an unprivileged user.
        if unsafe { libc::geteuid() } == 0 {
            return;
        }

        let base = TempDir::new().unwrap();
        let readonly_dir = base.path().join("1000");
        std::fs::create_dir(&readonly_dir).unwrap();
        // Remove write permission so File::create inside fails
        std::fs::set_permissions(&readonly_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

        // Should not panic — the error is logged to stderr and skipped
        signal_sessions_in(base.path());

        assert!(!readonly_dir.join("apt-updates-available").exists());

        // Restore permissions so TempDir cleanup succeeds
        std::fs::set_permissions(&readonly_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn test_signal_sessions_in_empty_dir() {
        let base = TempDir::new().unwrap();
        signal_sessions_in(base.path());
        // No panic, no files created
    }

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
