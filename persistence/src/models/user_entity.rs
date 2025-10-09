//! User persistence entity.
//!
//! This module defines the UserEntity struct which represents how user data
//! is stored in the database, including all persistence-specific fields like timestamps.

use lh_core::domain::user::User;

/// Persistence entity for user data.
///
/// This struct represents the complete user data as stored in the database,
/// including all timestamps and metadata. It uses database-friendly types
/// like i64 for Unix timestamps.
///
/// # Fields
///
/// * `username` - The unique username
/// * `created_at` - Unix timestamp (seconds since epoch) of creation
/// * `last_used_at` - Unix timestamp (seconds since epoch) of last use
///
/// # Examples
///
/// ```
/// use lh_persistence::models::UserEntity;
/// use chrono::Utc;
///
/// let entity = UserEntity {
///     username: "john_doe".to_string(),
///     created_at: Utc::now().timestamp(),
///     last_used_at: Utc::now().timestamp(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserEntity {
    /// Unique username for the user.
    pub username: String,
    /// Unix timestamp (seconds) of when the user was created.
    pub created_at: i64,
    /// Unix timestamp (seconds) of when the user was last used.
    pub last_used_at: i64,
}

impl UserEntity {
    /// Creates a new UserEntity with current timestamps.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the user
    ///
    /// # Returns
    ///
    /// A new `UserEntity` with created_at and last_used_at set to the current time.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_persistence::models::UserEntity;
    ///
    /// let entity = UserEntity::new("jane_smith".to_string());
    /// assert_eq!(entity.username, "jane_smith");
    /// assert!(entity.created_at > 0);
    /// assert_eq!(entity.created_at, entity.last_used_at);
    /// ```
    pub fn new(username: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            username,
            created_at: now,
            last_used_at: now,
        }
    }

    /// Converts this persistence entity to a domain User.
    ///
    /// This method maps from the persistence layer representation to the
    /// domain layer representation, stripping away persistence-specific
    /// details like timestamps.
    ///
    /// # Returns
    ///
    /// A `User` domain model.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_persistence::models::UserEntity;
    ///
    /// let entity = UserEntity::new("test_user".to_string());
    /// let user = entity.to_domain();
    /// assert_eq!(user.username, "test_user");
    /// ```
    pub fn to_domain(&self) -> User {
        // Use new_unchecked since data from DB is already validated
        User::new_unchecked(self.username.clone())
    }

    /// Creates a UserEntity from a domain User with current timestamps.
    ///
    /// This method maps from the domain layer representation to the
    /// persistence layer representation, adding persistence-specific
    /// metadata like timestamps.
    ///
    /// # Arguments
    ///
    /// * `user` - The domain User to convert
    ///
    /// # Returns
    ///
    /// A new `UserEntity` with timestamps set to the current time.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::domain::user::User;
    /// use lh_persistence::models::UserEntity;
    ///
    /// let user = User::new("test_user".to_string()).unwrap();
    /// let entity = UserEntity::from_domain(user);
    /// assert_eq!(entity.username, "test_user");
    /// assert!(entity.created_at > 0);
    /// ```
    pub fn from_domain(user: User) -> Self {
        Self::new(user.username)
    }

    /// Updates the last_used_at timestamp to the current time.
    ///
    /// This method modifies the entity in place, setting last_used_at
    /// to the current Unix timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_persistence::models::UserEntity;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let mut entity = UserEntity::new("test".to_string());
    /// let original_time = entity.last_used_at;
    ///
    /// // Sleep for at least 1 second since Unix timestamps are in seconds
    /// thread::sleep(Duration::from_secs(1));
    /// entity.update_last_used();
    ///
    /// assert!(entity.last_used_at > original_time);
    /// ```
    pub fn update_last_used(&mut self) {
        self.last_used_at = chrono::Utc::now().timestamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_entity_creation() {
        let entity = UserEntity::new("test_user".to_string());

        assert_eq!(entity.username, "test_user");
        assert!(entity.created_at > 0);
        assert!(entity.last_used_at > 0);
        assert_eq!(entity.created_at, entity.last_used_at);
    }

    #[test]
    fn test_to_domain() {
        let entity = UserEntity::new("test_user".to_string());
        let user = entity.to_domain();

        assert_eq!(user.username, entity.username);
    }

    #[test]
    fn test_from_domain() {
        let user = User::new_unchecked("test_user".to_string());
        let entity = UserEntity::from_domain(user.clone());

        assert_eq!(entity.username, user.username);
        assert!(entity.created_at > 0);
        assert!(entity.last_used_at > 0);
    }

    #[test]
    fn test_update_last_used() {
        let mut entity = UserEntity::new("test_user".to_string());
        let original_created = entity.created_at;
        let original_last_used = entity.last_used_at;

        // Sleep for at least 1 second since Unix timestamps are in seconds
        std::thread::sleep(std::time::Duration::from_secs(1));
        entity.update_last_used();

        assert_eq!(entity.created_at, original_created);
        assert!(entity.last_used_at > original_last_used);
    }

    #[test]
    fn test_clone() {
        let entity = UserEntity::new("test_user".to_string());
        let cloned = entity.clone();

        assert_eq!(entity, cloned);
    }
}
