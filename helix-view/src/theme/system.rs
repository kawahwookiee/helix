//! OS-level system theme detection.
//!
//! Provides fallback theme detection when the terminal doesn't support mode 2031.
//! Supports macOS and Linux (GNOME/GTK, KDE).

use super::Mode;
use std::process::Command;

/// Detects the system theme preference from the OS.
///
/// Returns `Some(Mode::Dark)` or `Some(Mode::Light)` if detection succeeds,
/// or `None` if the system theme cannot be determined.
pub fn detect() -> Option<Mode> {
    #[cfg(target_os = "macos")]
    {
        detect_macos()
    }

    #[cfg(target_os = "linux")]
    {
        detect_linux()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// Detect theme on macOS using `defaults read`.
#[cfg(target_os = "macos")]
fn detect_macos() -> Option<Mode> {
    // `defaults read -g AppleInterfaceStyle` returns "Dark" in dark mode,
    // and fails (non-zero exit) in light mode.
    let output = Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().eq_ignore_ascii_case("dark") {
            return Some(Mode::Dark);
        }
    }
    // If the command fails or returns something else, assume light mode
    Some(Mode::Light)
}

/// Detect theme on Linux using various desktop environment methods.
#[cfg(target_os = "linux")]
fn detect_linux() -> Option<Mode> {
    // Try GNOME/GTK first (most common)
    if let Some(mode) = detect_gnome() {
        return Some(mode);
    }

    // Try KDE Plasma
    if let Some(mode) = detect_kde() {
        return Some(mode);
    }

    // Try freedesktop portal (works for Flatpak apps and some DEs)
    if let Some(mode) = detect_freedesktop_portal() {
        return Some(mode);
    }

    None
}

/// Detect theme using GNOME/GTK settings.
#[cfg(target_os = "linux")]
fn detect_gnome() -> Option<Mode> {
    // Try the newer color-scheme setting first (GNOME 42+)
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let scheme = stdout.trim().trim_matches('\'');
        if scheme.contains("dark") {
            return Some(Mode::Dark);
        } else if scheme.contains("light") || scheme == "default" {
            return Some(Mode::Light);
        }
    }

    // Fall back to gtk-theme for older GNOME versions
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let theme = stdout.trim().trim_matches('\'').to_lowercase();
        if theme.contains("dark") {
            return Some(Mode::Dark);
        } else {
            return Some(Mode::Light);
        }
    }

    None
}

/// Detect theme using KDE Plasma settings.
#[cfg(target_os = "linux")]
fn detect_kde() -> Option<Mode> {
    // Try kreadconfig5 (KDE 5) or kreadconfig6 (KDE 6)
    for cmd in ["kreadconfig6", "kreadconfig5"] {
        let output = Command::new(cmd)
            .args(["--group", "General", "--key", "ColorScheme"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let scheme = stdout.trim().to_lowercase();
                if scheme.contains("dark") {
                    return Some(Mode::Dark);
                } else if !scheme.is_empty() {
                    return Some(Mode::Light);
                }
            }
        }
    }

    None
}

/// Detect theme using the freedesktop portal interface.
#[cfg(target_os = "linux")]
fn detect_freedesktop_portal() -> Option<Mode> {
    // Use busctl to query the portal
    let output = Command::new("busctl")
        .args([
            "--user",
            "call",
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Settings",
            "Read",
            "ss",
            "org.freedesktop.appearance",
            "color-scheme",
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // The output format is like: v u 1
        // where 1 = prefer dark, 2 = prefer light, 0 = no preference
        if stdout.contains(" 1") || stdout.ends_with("1") {
            return Some(Mode::Dark);
        } else if stdout.contains(" 2") || stdout.ends_with("2") {
            return Some(Mode::Light);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_returns_valid_mode() {
        // This test just ensures detect() doesn't panic
        let _ = detect();
    }
}
