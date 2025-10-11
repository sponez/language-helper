//! User repository trait.
//!
//! This module defines the repository trait for user persistence operations.
//! The actual implementation will be provided by the persistence layer.

use crate::models::user::User;
use crate::errors::CoreError;

/// Repository trait for user persistence operations.
///
/// This trait defines the interface for persisting and retrieving user data.
/// Implementations of this trait will be provided by the persistence layer,
/// allowing the core business logic to remain independent of the specific
/// persistence mechanism.
///
/// # Type Safety
///
/// This trait uses `async_trait` to support asynchronous operations. The actual
/// implementation will be in the persistence layer.
///
/// # Examples
///
/// ```no_run
/// use lh_core::repositories::user_repository::UserRepository;
/// use lh_core::models::user::User;
///
/// fn example(repo: &dyn UserRepository) {
///     match repo.find_by_username("john_doe") {
///         Ok(Some(user)) => println!("Found user: {}", user.username),
///         Ok(None) => println!("User not found"),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub trait UserRepository: Send + Sync {
    /// Finds a user by username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(User))` - The user if found
    /// * `Ok(None)` - If no user with the given username exists
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    fn find_by_username(&self, username: &str) -> Result<Option<User>, CoreError>;

    /// Retrieves all users.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<User>)` - A vector containing all users
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    fn find_all(&self) -> Result<Vec<User>, CoreError>;

    /// Saves a user to the repository.
    ///
    /// This method will create a new user if one with the given username
    /// doesn't exist, or update an existing user if it does.
    ///
    /// # Arguments
    ///
    /// * `user` - The user to save
    ///
    /// # Returns
    ///
    /// * `Ok(User)` - The saved user
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store, or `CoreError::ValidationError` if the
    /// user data is invalid.
    fn save(&self, user: User) -> Result<User, CoreError>;

    /// Deletes a user by username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the user to delete
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the user was successfully deleted
    /// * `Ok(false)` - If no user with the given username exists
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    fn delete(&self, username: &str) -> Result<bool, CoreError>;
}
