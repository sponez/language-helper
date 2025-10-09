//! User domain model.
//!
//! This module defines the core User entity representing a user in the system.

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
///     username: "john_doe".to_string(),
///     email: "john@example.com".to_string(),
///     full_name: Some("John Doe".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// Unique username for the user.
    pub username: String,

    /// Email address of the user.
    pub email: String,

    /// Optional full name of the user.
    pub full_name: Option<String>,
}

impl User {
    /// Creates a new User instance.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the user
    /// * `email` - The email address for the user
    /// * `full_name` - Optional full name for the user
    ///
    /// # Returns
    ///
    /// A new `User` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::domain::user::User;
    ///
    /// let user = User::new(
    ///     "jane_smith".to_string(),
    ///     "jane@example.com".to_string(),
    ///     Some("Jane Smith".to_string())
    /// );
    /// ```
    pub fn new(username: String, email: String, full_name: Option<String>) -> Self {
        Self {
            username,
            email,
            full_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "test_user".to_string(),
            "test@example.com".to_string(),
            Some("Test User".to_string()),
        );

        assert_eq!(user.username, "test_user");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.full_name, Some("Test User".to_string()));
    }

    #[test]
    fn test_user_without_full_name() {
        let user = User::new(
            "test_user".to_string(),
            "test@example.com".to_string(),
            None,
        );

        assert_eq!(user.username, "test_user");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.full_name, None);
    }

    #[test]
    fn test_user_clone() {
        let user = User::new(
            "original".to_string(),
            "original@example.com".to_string(),
            Some("Original User".to_string()),
        );

        let cloned = user.clone();
        assert_eq!(user, cloned);
    }
}
