//! User management API.
//!
//! This module provides the trait definition for user-related operations.

use crate::{
    errors::api_error::ApiError,
    models::{profile::ProfileDto, user::UserDto},
};
use async_trait::async_trait;

/// API for managing and retrieving user data.
///
/// This trait defines the interface for user-related operations such as
/// retrieving usernames and fetching user details.
#[async_trait]
pub trait UsersApi: Send + Sync {
    /// Retrieves a list of all usernames.
    async fn get_usernames(&self) -> Result<Vec<String>, ApiError>;

    /// Retrieves a user by their username.
    async fn get_user_by_username(&self, username: &str) -> Option<UserDto>;

    /// Creates a new user with the given username and language preference.
    async fn create_user(&self, username: &str, language: &str) -> Result<UserDto, ApiError>;

    /// Updates a user's theme preference.
    async fn update_user_theme(&self, username: &str, theme: &str) -> Result<(), ApiError>;

    /// Updates a user's language preference.
    async fn update_user_language(&self, username: &str, language: &str) -> Result<(), ApiError>;

    /// Deletes a user and all associated data.
    async fn delete_user(&self, username: &str) -> Result<bool, ApiError>;

    /// Creates a new learning profile for a user.
    async fn create_profile(
        &self,
        username: &str,
        profile_name: &str,
        target_language: &str,
    ) -> Result<ProfileDto, ApiError>;

    /// Deletes a profile and its associated database file.
    async fn delete_profile(&self, username: &str, profile_name: &str) -> Result<bool, ApiError>;
}
