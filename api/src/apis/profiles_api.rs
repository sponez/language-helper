//! Profile management API.
//!
//! This module provides the trait definition for profile-specific database operations.

use async_trait::async_trait;
use crate::errors::api_error::ApiError;

/// API for managing learning profile databases and content.
///
/// This API handles profile-specific databases where learning content (vocabulary cards,
/// progress, etc.) is stored. Each profile has its own database file at
/// `data/{username}/{target_language}_profile.db`.
///
/// Profile metadata (list of profiles, creation dates) is managed by UsersApi.
/// This API only handles operations on the profile's learning database.
#[async_trait]
pub trait ProfilesApi: Send + Sync {
    /// Creates a profile database file.
    async fn create_profile_database(&self, username: &str, target_language: &str) -> Result<(), ApiError>;

    /// Deletes a profile database file.
    async fn delete_profile_database(&self, username: &str, target_language: &str) -> Result<bool, ApiError>;

    /// Deletes the entire user data folder.
    async fn delete_user_folder(&self, username: &str) -> Result<bool, ApiError>;

    // Future methods will be added here for:
    // - Adding vocabulary cards
    // - Tracking learning progress
    // - Managing flashcard decks
    // - etc.
}
