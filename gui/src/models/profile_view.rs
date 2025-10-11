//! Profile view model for GUI presentation.

/// View model for displaying learning profile information in the GUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileView {
    /// Target language being learned
    pub target_language: String,
    /// Human-readable creation date/time
    pub created_at_display: String,
    /// Human-readable last activity date/time
    pub last_activity_display: String,
}

impl ProfileView {
    /// Creates a new ProfileView.
    pub fn new<TL, C, LA>(
        target_language: TL,
        created_at_display: C,
        last_activity_display: LA,
    ) -> Self
    where
        TL: AsRef<str> + Into<String>,
        C: AsRef<str> + Into<String>,
        LA: AsRef<str> + Into<String>,
    {
        Self {
            target_language: target_language.into(),
            created_at_display: created_at_display.into(),
            last_activity_display: last_activity_display.into(),
        }
    }
}
