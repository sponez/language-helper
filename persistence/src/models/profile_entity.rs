//! Profile persistence entity.
//!
//! This module defines the ProfileEntity struct which represents a learning profile
//! stored in the database with a one-to-many relationship to users.

use lh_core::domain::profile::Profile;

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
///     "spanish".to_string()
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileEntity {
    /// Unique profile identifier.
    pub profile_id: String,
    /// Username (foreign key to users table).
    pub username: String,
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
    ///     "french".to_string()
    /// );
    /// assert!(entity.created_at > 0);
    /// ```
    pub fn new(username: String, target_language: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        let profile_id = format!("{}_{}", username, uuid::Uuid::new_v4());

        Self {
            profile_id,
            username,
            target_language,
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
    /// * `profile_id` - The profile identifier
    /// * `username` - The username
    /// * `target_language` - The target language
    /// * `created_at` - Creation timestamp
    /// * `last_activity_at` - Last activity timestamp
    ///
    /// # Returns
    ///
    /// A new `ProfileEntity` instance.
    pub fn with_fields(
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

    /// Converts this persistence entity to a domain Profile.
    ///
    /// # Returns
    ///
    /// A `Profile` domain model.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_persistence::models::ProfileEntity;
    ///
    /// let entity = ProfileEntity::new(
    ///     "test_user".to_string(),
    ///     "german".to_string()
    /// );
    /// let profile = entity.to_domain();
    /// assert_eq!(profile.username, "test_user");
    /// ```
    pub fn to_domain(&self) -> Profile {
        Profile::new_unchecked(
            self.profile_id.clone(),
            self.username.clone(),
            self.target_language.clone(),
            self.created_at,
            self.last_activity_at,
        )
    }

    /// Creates a ProfileEntity from a domain Profile.
    ///
    /// # Arguments
    ///
    /// * `profile` - The domain Profile to convert
    ///
    /// # Returns
    ///
    /// A new `ProfileEntity`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::domain::profile::Profile;
    /// use lh_persistence::models::ProfileEntity;
    ///
    /// let profile = Profile::new(
    ///     "test".to_string(),
    ///     "italian".to_string()
    /// ).unwrap();
    /// let entity = ProfileEntity::from_domain(profile);
    /// assert_eq!(entity.username, "test");
    /// ```
    pub fn from_domain(profile: Profile) -> Self {
        Self {
            profile_id: profile.profile_id,
            username: profile.username,
            target_language: profile.target_language,
            created_at: profile.created_at,
            last_activity_at: profile.last_activity_at,
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
    /// let mut entity = ProfileEntity::new("test".to_string(), "spanish".to_string());
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
        let entity = ProfileEntity::new("test_user".to_string(), "spanish".to_string());

        assert_eq!(entity.username, "test_user");
        assert_eq!(entity.target_language, "spanish");
        assert!(entity.created_at > 0);
        assert!(entity.last_activity_at > 0);
        assert_eq!(entity.created_at, entity.last_activity_at);
        assert!(entity.profile_id.contains("test_user"));
    }

    #[test]
    fn test_profile_entity_with_fields() {
        let entity = ProfileEntity::with_fields(
            "profile_123".to_string(),
            "user1".to_string(),
            "french".to_string(),
            1000,
            2000,
        );

        assert_eq!(entity.profile_id, "profile_123");
        assert_eq!(entity.username, "user1");
        assert_eq!(entity.target_language, "french");
        assert_eq!(entity.created_at, 1000);
        assert_eq!(entity.last_activity_at, 2000);
    }

    #[test]
    fn test_to_domain() {
        let entity = ProfileEntity::new("test_user".to_string(), "german".to_string());
        let profile = entity.to_domain();

        assert_eq!(profile.profile_id, entity.profile_id);
        assert_eq!(profile.username, entity.username);
        assert_eq!(profile.target_language, entity.target_language);
    }

    #[test]
    fn test_update_last_activity() {
        let mut entity = ProfileEntity::new("test_user".to_string(), "italian".to_string());
        let original_created = entity.created_at;
        let original_last_activity = entity.last_activity_at;

        std::thread::sleep(std::time::Duration::from_secs(1));
        entity.update_last_activity();

        assert_eq!(entity.created_at, original_created);
        assert!(entity.last_activity_at > original_last_activity);
    }

    #[test]
    fn test_clone() {
        let entity = ProfileEntity::new("test_user".to_string(), "portuguese".to_string());
        let cloned = entity.clone();

        assert_eq!(entity, cloned);
    }

    #[test]
    fn test_unique_profile_ids() {
        let entity1 = ProfileEntity::new("user1".to_string(), "spanish".to_string());
        let entity2 = ProfileEntity::new("user1".to_string(), "spanish".to_string());

        assert_ne!(entity1.profile_id, entity2.profile_id);
    }
}
