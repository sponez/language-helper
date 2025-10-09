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
///     username: "john_doe".to_string()
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// Unique username for the user.
    pub username: String,
}

impl User {
    /// Creates a new User instance.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the user
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
    /// let user = User::new("jane_smith".to_string());
    /// assert_eq!(user.username, "jane_smith");
    /// ```
    pub fn new(username: String) -> Self {
        Self { username }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "test_user".to_string(),
        );

        assert_eq!(user.username, "test_user");
    }

    #[test]
    fn test_user_clone() {
        let user = User::new(
            "original".to_string(),
        );

        let cloned = user.clone();
        assert_eq!(user, cloned);
    }
}
