//! User settings persistence entity.
//!
//! This module defines the UserSettingsEntity struct which represents user-specific
//! settings stored in the database with a one-to-one relationship to users.

/// Persistence entity for user settings.
///
/// This struct represents user-specific settings as stored in the database.
/// It has a one-to-one relationship with the users table via username foreign key.
///
/// # Fields
///
/// * `username` - Foreign key reference to users table
/// * `ui_theme` - User's UI theme preference (Light, Dark, System)
/// * `ui_language` - User's UI language code (e.g., "en", "es")
///
/// # Examples
///
/// ```
/// use lh_persistence::models::UserSettingsEntity;
///
/// let entity = UserSettingsEntity::new(
///     "john_doe".to_string(),
///     "Dark".to_string(),
///     "en".to_string()
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSettingsEntity {
    /// Username (foreign key to users table).
    pub username: String,
    /// UI theme preference.
    pub ui_theme: String,
    /// UI language code.
    pub ui_language: String,
}

impl UserSettingsEntity {
    /// Creates a new UserSettingsEntity.
    ///
    /// # Arguments
    ///
    /// * `username` - The username this settings belongs to
    /// * `ui_theme` - The UI theme preference
    /// * `ui_language` - The UI language code
    ///
    /// # Returns
    ///
    /// A new `UserSettingsEntity` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_persistence::models::UserSettingsEntity;
    ///
    /// let entity = UserSettingsEntity::new(
    ///     "jane_smith".to_string(),
    ///     "Light".to_string(),
    ///     "es".to_string()
    /// );
    /// assert_eq!(entity.username, "jane_smith");
    /// ```
    pub fn new(username: String, ui_theme: String, ui_language: String) -> Self {
        Self {
            username,
            ui_theme,
            ui_language,
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_settings_entity_creation() {
        let entity = UserSettingsEntity::new(
            "test_user".to_string(),
            "Dark".to_string(),
            "en".to_string(),
        );

        assert_eq!(entity.username, "test_user");
        assert_eq!(entity.ui_theme, "Dark");
        assert_eq!(entity.ui_language, "en");
    }

    #[test]
    fn test_clone() {
        let entity =
            UserSettingsEntity::new("test".to_string(), "Dark".to_string(), "en".to_string());
        let cloned = entity.clone();

        assert_eq!(entity, cloned);
    }
}
