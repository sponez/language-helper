//! Profile persistence entity.
//!
//! This module defines the ProfileEntity struct which represents a learning profile
//! stored in the database with a one-to-many relationship to users.

/// Persistence entity for learning profiles.
///
/// This struct represents a learning profile as stored in the database.
/// Each profile is linked to a user and represents learning progress for a specific
/// target language.
///
/// # Fields
///
/// * `profile_id` - Unique identifier for the profile
/// * `username` - Foreign key reference to users table
/// * `target_language` - The language being learned (code, e.g., "es", "fr")
/// * `created_at` - Unix timestamp of creation
/// * `last_activity_at` - Unix timestamp of last activity
///
/// # Examples
///
/// ```
/// use lh_persistence::models::ProfileEntity;
///
/// let entity = ProfileEntity::new(
///     "user1".to_string(),
///     "My Spanish".to_string(),
///     "spanish".to_string()
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileEntity {
    /// Username (foreign key to users table).
    pub username: String,
    /// Profile name (primary key within user's profiles).
    pub profile_name: String,
    /// Target language code.
    pub target_language: String,
    /// Unix timestamp (seconds) of when the profile was created.
    pub created_at: i64,
    /// Unix timestamp (seconds) of last activity.
    pub last_activity_at: i64,
}

impl ProfileEntity {
    /// Creates a new ProfileEntity with current timestamps.
    ///
    /// # Arguments
    ///
    /// * `username` - The username this profile belongs to
    /// * `profile_name` - The name of the profile (user-defined)
    /// * `target_language` - The language being learned
    ///
    /// # Returns
    ///
    /// A new `ProfileEntity` with generated ID and current timestamps.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_persistence::models::ProfileEntity;
    ///
    /// let entity = ProfileEntity::new(
    ///     "john_doe".to_string(),
    ///     "My French Profile".to_string(),
    ///     "french".to_string()
    /// );
    /// assert!(entity.created_at > 0);
    /// ```
    pub fn new<U, PN, TL>(username: U, profile_name: PN, target_language: TL) -> Self
    where
        U: AsRef<str> + Into<String>,
        PN: AsRef<str> + Into<String>,
        TL: AsRef<str> + Into<String>,
    {
        let now = chrono::Utc::now().timestamp();

        Self {
            username: username.into(),
            profile_name: profile_name.into(),
            target_language: target_language.into(),
            created_at: now,
            last_activity_at: now,
        }
    }

    /// Creates a ProfileEntity with all fields specified.
    ///
    /// This is useful when loading from the database.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `profile_name` - The profile name
    /// * `target_language` - The target language
    /// * `created_at` - Creation timestamp
    /// * `last_activity_at` - Last activity timestamp
    ///
    /// # Returns
    ///
    /// A new `ProfileEntity` instance.
    pub fn with_fields<U, PN, TL>(
        username: U,
        profile_name: PN,
        target_language: TL,
        created_at: i64,
        last_activity_at: i64,
    ) -> Self
    where
        U: AsRef<str> + Into<String>,
        PN: AsRef<str> + Into<String>,
        TL: AsRef<str> + Into<String>,
    {
        Self {
            username: username.into(),
            profile_name: profile_name.into(),
            target_language: target_language.into(),
            created_at,
            last_activity_at,
        }
    }

    /// Updates the last_activity_at timestamp to the current time.
    ///
    /// This method modifies the entity in place.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_persistence::models::ProfileEntity;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let mut entity = ProfileEntity::new(
    ///     "test".to_string(),
    ///     "My Profile".to_string(),
    ///     "spanish".to_string()
    /// );
    /// let original_time = entity.last_activity_at;
    ///
    /// thread::sleep(Duration::from_secs(1));
    /// entity.update_last_activity();
    ///
    /// assert!(entity.last_activity_at > original_time);
    /// ```
    pub fn update_last_activity(&mut self) {
        self.last_activity_at = chrono::Utc::now().timestamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_entity_creation() {
        let entity = ProfileEntity::new(
            "test_user".to_string(),
            "My Spanish".to_string(),
            "spanish".to_string(),
        );

        assert_eq!(entity.username, "test_user");
        assert_eq!(entity.profile_name, "My Spanish");
        assert_eq!(entity.target_language, "spanish");
        assert!(entity.created_at > 0);
        assert!(entity.last_activity_at > 0);
        assert_eq!(entity.created_at, entity.last_activity_at);
    }

    #[test]
    fn test_profile_entity_with_fields() {
        let entity = ProfileEntity::with_fields(
            "user1".to_string(),
            "My French".to_string(),
            "french".to_string(),
            1000,
            2000,
        );

        assert_eq!(entity.username, "user1");
        assert_eq!(entity.profile_name, "My French");
        assert_eq!(entity.target_language, "french");
        assert_eq!(entity.created_at, 1000);
        assert_eq!(entity.last_activity_at, 2000);
    }

    #[test]
    fn test_update_last_activity() {
        let mut entity = ProfileEntity::new(
            "test_user".to_string(),
            "My Italian".to_string(),
            "italian".to_string(),
        );
        let original_created = entity.created_at;
        let original_last_activity = entity.last_activity_at;

        std::thread::sleep(std::time::Duration::from_secs(1));
        entity.update_last_activity();

        assert_eq!(entity.created_at, original_created);
        assert!(entity.last_activity_at > original_last_activity);
    }

    #[test]
    fn test_clone() {
        let entity = ProfileEntity::new(
            "test_user".to_string(),
            "My Portuguese".to_string(),
            "portuguese".to_string(),
        );
        let cloned = entity.clone();

        assert_eq!(entity, cloned);
    }
}
