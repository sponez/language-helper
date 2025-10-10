//! Profile domain model.
//!
//! This module defines the learning profile entity.

use crate::errors::CoreError;

/// Learning profile.
///
/// This struct represents a learning profile in the domain layer.
/// Each profile is associated with a user and a target language being learned.
///
/// # Examples
///
/// ```
/// use lh_core::domain::profile::Profile;
///
/// let profile = Profile::new(
///     "john_doe".to_string(),
///     "spanish".to_string()
/// ).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// Unique profile identifier.
    pub profile_id: String,
    /// Username this profile belongs to.
    pub username: String,
    /// Target language being learned.
    pub target_language: String,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last activity.
    pub last_activity_at: i64,
}

impl Profile {
    /// Creates a new Profile instance with validation.
    ///
    /// This generates a new profile ID and sets timestamps to current time.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language being learned
    ///
    /// # Returns
    ///
    /// * `Ok(Profile)` - If validation passes
    /// * `Err(CoreError)` - If validation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::domain::profile::Profile;
    ///
    /// let profile = Profile::new("test_user".to_string(), "french".to_string()).unwrap();
    /// assert_eq!(profile.username, "test_user");
    /// assert_eq!(profile.target_language, "french");
    /// assert!(profile.created_at > 0);
    /// ```
    pub fn new(username: String, target_language: String) -> Result<Self, CoreError> {
        Self::validate_username(&username)?;
        Self::validate_target_language(&target_language)?;

        let now = chrono::Utc::now().timestamp();
        let profile_id = format!("{}_{}", username, uuid::Uuid::new_v4());

        Ok(Self {
            profile_id,
            username,
            target_language,
            created_at: now,
            last_activity_at: now,
        })
    }

    /// Creates a Profile without validation (for internal use only).
    ///
    /// This should only be used when loading from trusted sources like the database.
    ///
    /// # Arguments
    ///
    /// * `profile_id` - The profile identifier
    /// * `username` - The username
    /// * `target_language` - The target language
    /// * `created_at` - Creation timestamp
    /// * `last_activity_at` - Last activity timestamp
    ///
    /// # Returns
    ///
    /// A new `Profile` instance without validation.
    pub fn new_unchecked(
        profile_id: String,
        username: String,
        target_language: String,
        created_at: i64,
        last_activity_at: i64,
    ) -> Self {
        Self {
            profile_id,
            username,
            target_language,
            created_at,
            last_activity_at,
        }
    }

    /// Validates the username.
    fn validate_username(username: &str) -> Result<(), CoreError> {
        if username.is_empty() {
            return Err(CoreError::validation_error("Username cannot be empty"));
        }
        Ok(())
    }

    /// Validates the target language.
    fn validate_target_language(language: &str) -> Result<(), CoreError> {
        if language.is_empty() {
            return Err(CoreError::validation_error(
                "Target language cannot be empty",
            ));
        }
        if language.len() > 50 {
            return Err(CoreError::validation_error(
                "Target language cannot exceed 50 characters",
            ));
        }
        Ok(())
    }

    /// Updates the last_activity_at timestamp to the current time.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::domain::profile::Profile;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let mut profile = Profile::new("test".to_string(), "spanish".to_string()).unwrap();
    /// let original_time = profile.last_activity_at;
    ///
    /// thread::sleep(Duration::from_secs(1));
    /// profile.update_last_activity();
    ///
    /// assert!(profile.last_activity_at > original_time);
    /// ```
    pub fn update_last_activity(&mut self) {
        self.last_activity_at = chrono::Utc::now().timestamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation_valid() {
        let profile = Profile::new("test_user".to_string(), "spanish".to_string()).unwrap();
        assert_eq!(profile.username, "test_user");
        assert_eq!(profile.target_language, "spanish");
        assert!(profile.created_at > 0);
        assert!(profile.last_activity_at > 0);
        assert_eq!(profile.created_at, profile.last_activity_at);
        assert!(profile.profile_id.contains("test_user"));
    }

    #[test]
    fn test_empty_username() {
        let result = Profile::new("".to_string(), "spanish".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_empty_target_language() {
        let result = Profile::new("test_user".to_string(), "".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_target_language_too_long() {
        let result = Profile::new("test_user".to_string(), "a".repeat(51));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_valid_target_languages() {
        let valid_languages = vec!["spanish", "french", "german", "chinese", "japanese"];
        for language in valid_languages {
            let result = Profile::new("test_user".to_string(), language.to_string());
            assert!(result.is_ok(), "Language '{}' should be valid", language);
        }
    }

    #[test]
    fn test_update_last_activity() {
        let mut profile = Profile::new("test_user".to_string(), "italian".to_string()).unwrap();
        let original_created = profile.created_at;
        let original_last_activity = profile.last_activity_at;

        std::thread::sleep(std::time::Duration::from_secs(1));
        profile.update_last_activity();

        assert_eq!(profile.created_at, original_created);
        assert!(profile.last_activity_at > original_last_activity);
    }

    #[test]
    fn test_unique_profile_ids() {
        let profile1 = Profile::new("user1".to_string(), "spanish".to_string()).unwrap();
        let profile2 = Profile::new("user1".to_string(), "spanish".to_string()).unwrap();

        assert_ne!(profile1.profile_id, profile2.profile_id);
    }

    #[test]
    fn test_clone() {
        let profile = Profile::new("test_user".to_string(), "portuguese".to_string()).unwrap();
        let cloned = profile.clone();
        assert_eq!(profile, cloned);
    }

    #[test]
    fn test_new_unchecked() {
        let profile = Profile::new_unchecked(
            "profile_123".to_string(),
            "".to_string(),
            "".to_string(),
            1000,
            2000,
        );
        assert_eq!(profile.profile_id, "profile_123");
        assert_eq!(profile.username, "");
        assert_eq!(profile.created_at, 1000);
        assert_eq!(profile.last_activity_at, 2000);
    }
}
