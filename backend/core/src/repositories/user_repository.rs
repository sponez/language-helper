//! User repository trait.
//!
//! This module defines the repository trait for user persistence operations.
//! The actual implementation will be provided by the persistence layer.

use crate::errors::CoreError;
use crate::models::user::User;
use async_trait::async_trait;

/// Repository trait for user persistence operations.
///
/// This trait defines the interface for persisting and retrieving user data.
/// Implementations of this trait will be provided by the persistence layer,
/// allowing the core business logic to remain independent of the specific
/// persistence mechanism.
#[async_trait]
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
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, CoreError>;

    /// Retrieves all users.
    async fn find_all(&self) -> Result<Vec<User>, CoreError>;

    /// Saves a user to the repository.
    async fn save(&self, user: User) -> Result<User, CoreError>;

    /// Deletes a user by username.
    async fn delete(&self, username: &str) -> Result<bool, CoreError>;
}
