//! Profile management API.
//!
//! This module provides the trait definition for profile-specific database operations.

use crate::errors::api_error::ApiError;

/// API for managing learning profile databases and content.
///
/// This API handles profile-specific databases where learning content (vocabulary cards,
/// progress, etc.) is stored. Each profile has its own database file at
/// `data/{username}/{target_language}_profile.db`.
///
/// Profile metadata (list of profiles, creation dates) is managed by UsersApi.
/// This API only handles operations on the profile's learning database.
///
/// # Examples
///
/// ```no_run
/// use lh_api::apis::profiles_api::ProfilesApi;
///
/// fn initialize_profile(api: &dyn ProfilesApi, username: &str, language: &str) -> Result<(), Box<dyn std::error::Error>> {
///     api.create_profile_database(username, language)?;
///     Ok(())
/// }
/// ```
pub trait ProfilesApi: Send + Sync {
    /// Creates a profile database file.
    ///
    /// This creates the database file at `data/{username}/{target_language}_profile.db`
    /// and initializes it with the required schema.
    ///
    /// # Arguments
    ///
    /// * `username` - The username this profile belongs to
    /// * `target_language` - The target language for this profile
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the database was created successfully
    /// * `Err(ApiError)` - If an error occurs
    fn create_profile_database(&self, username: &str, target_language: &str) -> Result<(), ApiError>;

    /// Deletes a profile database file.
    ///
    /// This removes the database file at `data/{username}/{target_language}_profile.db`.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the database was deleted
    /// * `Ok(false)` - If the database file didn't exist
    /// * `Err(ApiError)` - If an error occurs
    fn delete_profile_database(&self, username: &str, target_language: &str) -> Result<bool, ApiError>;

    // Future methods will be added here for:
    // - Adding vocabulary cards
    // - Tracking learning progress
    // - Managing flashcard decks
    // - etc.
}
