//! Mapper for converting between Profile core model and ProfileView.

use crate::models::ProfileView;
use lh_core::models::profile::Profile;

/// Converts a core Profile model to a GUI ProfileView.
///
/// This includes formatting timestamps into human-readable strings.
///
/// # Arguments
///
/// * `profile` - The core Profile model
///
/// # Returns
///
/// A ProfileView for display in the GUI
pub fn model_to_view(profile: &Profile) -> ProfileView {
    // Format timestamps into human-readable strings
    let created_at_display = format_timestamp(profile.created_at);
    let last_activity_display = format_timestamp(profile.last_activity_at);

    ProfileView::new(
        profile.target_language.clone(),
        created_at_display,
        last_activity_display,
    )
}

/// Formats a Unix timestamp into a human-readable string.
fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, Utc};

    match DateTime::<Utc>::from_timestamp(timestamp, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        None => "Invalid date".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_to_view() {
        let profile = Profile::new("spanish").unwrap();
        let view = model_to_view(&profile);

        assert_eq!(view.target_language, profile.target_language);
        assert!(!view.created_at_display.is_empty());
        assert!(!view.last_activity_display.is_empty());
    }

    #[test]
    fn test_format_timestamp() {
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let formatted = format_timestamp(timestamp);
        assert!(formatted.contains("2021"));
    }
}
