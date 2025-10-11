//! Profile view model for GUI presentation.

/// View model for displaying learning profile information in the GUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileView {
    /// Unique profile identifier
    pub profile_id: String,
    /// The username this profile belongs to
    pub username: String,
    /// Target language being learned
    pub target_language: String,
    /// Human-readable creation date/time
    pub created_at_display: String,
    /// Human-readable last activity date/time
    pub last_activity_display: String,
}

impl ProfileView {
    /// Creates a new ProfileView.
    pub fn new(
        profile_id: String,
        username: String,
        target_language: String,
        created_at_display: String,
        last_activity_display: String,
    ) -> Self {
        Self {
            profile_id,
            username,
            target_language,
            created_at_display,
            last_activity_display,
        }
    }
}
