//! User domain model.
//!
//! This module defines the core User entity representing a user in the system.

use crate::errors::CoreError;

/// Core user entity.
///
/// This struct represents a user in the domain layer. It contains
/// the essential business data for a user entity.
///
/// # Examples
///
/// ```
/// use lh_core::domain::user::User;
///
/// let user = User {
///     username: "john_doe".to_string()
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// Unique username for the user.
    pub username: String,
}

impl User {
    /// Minimum allowed username length.
    pub const MIN_USERNAME_LENGTH: usize = 1;

    /// Maximum allowed username length.
    pub const MAX_USERNAME_LENGTH: usize = 50;

    /// Creates a new User instance with validation.
    ///
    /// This method enforces business rules:
    /// - Username must not be empty
    /// - Username must not exceed 50 characters
    /// - Username must contain only alphanumeric characters and underscores
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the user
    ///
    /// # Returns
    ///
    /// * `Ok(User)` - If validation passes
    /// * `Err(CoreError)` - If validation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::domain::user::User;
    ///
    /// let user = User::new("jane_smith".to_string()).unwrap();
    /// assert_eq!(user.username, "jane_smith");
    ///
    /// // Invalid usernames
    /// assert!(User::new("".to_string()).is_err());
    /// assert!(User::new("a".repeat(51)).is_err());
    /// assert!(User::new("invalid-name!".to_string()).is_err());
    /// ```
    pub fn new(username: String) -> Result<Self, CoreError> {
        Self::validate_username(&username)?;
        Ok(Self { username })
    }

    /// Creates a User without validation (for internal use only).
    ///
    /// This should only be used when loading from trusted sources like the database
    /// where validation has already occurred, or in tests.
    ///
    /// # Safety
    ///
    /// This bypasses all validation. Only use this when you're certain the data is valid.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the user
    ///
    /// # Returns
    ///
    /// A new `User` instance without validation.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::domain::user::User;
    ///
    /// // Only use for trusted data (e.g., from database)
    /// let user = User::new_unchecked("test_user".to_string());
    /// assert_eq!(user.username, "test_user");
    /// ```
    pub fn new_unchecked(username: String) -> Self {
        Self { username }
    }

    /// Validates a username according to business rules.
    ///
    /// # Business Rules
    ///
    /// - Must not be empty
    /// - Must not exceed MAX_USERNAME_LENGTH characters
    /// - Must contain only alphanumeric characters and underscores
    ///
    /// # Arguments
    ///
    /// * `username` - The username to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If validation passes
    /// * `Err(CoreError)` - If validation fails with a descriptive error
    fn validate_username(username: &str) -> Result<(), CoreError> {
        if username.is_empty() {
            return Err(CoreError::validation_error("Username cannot be empty"));
        }

        if username.len() > Self::MAX_USERNAME_LENGTH {
            return Err(CoreError::validation_error(format!(
                "Username cannot exceed {} characters",
                Self::MAX_USERNAME_LENGTH
            )));
        }

        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(CoreError::validation_error(
                "Username must contain only alphanumeric characters and underscores",
            ));
        }

        Ok(())
    }

    /// Returns the username.
    ///
    /// # Returns
    ///
    /// A reference to the username string.
    pub fn username(&self) -> &str {
        &self.username
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation_valid() {
        let user = User::new("test_user".to_string()).unwrap();
        assert_eq!(user.username, "test_user");
    }

    #[test]
    fn test_user_creation_empty_username() {
        let result = User::new("".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_user_creation_username_too_long() {
        let long_username = "a".repeat(51);
        let result = User::new(long_username);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_user_creation_max_length_allowed() {
        let max_username = "a".repeat(50);
        let result = User::new(max_username.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().username, max_username);
    }

    #[test]
    fn test_user_creation_invalid_characters() {
        let invalid_usernames = vec![
            "user-name",
            "user.name",
            "user name",
            "user@name",
            "user!name",
            "user#name",
        ];

        for username in invalid_usernames {
            let result = User::new(username.to_string());
            assert!(result.is_err(), "Username '{}' should be invalid", username);
            assert!(matches!(
                result.unwrap_err(),
                CoreError::ValidationError { .. }
            ));
        }
    }

    #[test]
    fn test_user_creation_valid_characters() {
        let valid_usernames = vec![
            "user",
            "user123",
            "user_name",
            "User_Name_123",
            "123",
            "_",
            "a",
        ];

        for username in valid_usernames {
            let result = User::new(username.to_string());
            assert!(result.is_ok(), "Username '{}' should be valid", username);
        }
    }

    #[test]
    fn test_user_clone() {
        let user = User::new("original".to_string()).unwrap();
        let cloned = user.clone();
        assert_eq!(user, cloned);
    }

    #[test]
    fn test_user_username_getter() {
        let user = User::new("test_user".to_string()).unwrap();
        assert_eq!(user.username(), "test_user");
    }

    #[test]
    fn test_user_new_unchecked() {
        // This should work even with invalid username (for internal use)
        let user = User::new_unchecked("invalid-name!".to_string());
        assert_eq!(user.username, "invalid-name!");
    }
}
