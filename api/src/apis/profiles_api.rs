//! Profile management API.
//!
//! This module provides the trait definition for profile-related operations.

use crate::{errors::api_error::ApiError, models::profile::ProfileDto};

/// API for managing learning profile databases and content.
///
/// This API handles profile-specific databases where learning content (vocabulary cards,
/// progress, etc.) is stored. Each profile has its own database file at
/// `data/{username}/{target_language}_profile.db`.
///
/// This is separate from profile metadata, which is managed by UsersApi via UserProfilesService.
///
/// # Examples
///
/// ```no_run
/// use lh_api::apis::profiles_api::ProfilesApi;
///
/// fn list_user_profiles(api: &dyn ProfilesApi, username: &str) -> Result<(), Box<dyn std::error::Error>> {
///     let profiles = api.get_profiles_by_username(username)?;
///     for profile in profiles {
///         println!("Profile: {}", profile.target_language);
///     }
///     Ok(())
/// }
/// ```
pub trait ProfilesApi {
    /// Retrieves all profiles for a specific user.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to get profiles for
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ProfileDto>)` - A vector containing all profiles for the user
    /// * `Err(ApiError)` - If an error occurs while retrieving the profiles
    ///
    /// # Errors
    ///
    /// This function may return an error if the underlying data source is unavailable
    /// or if there's a permission issue.
    fn get_profiles_by_username(&self, username: &str) -> Result<Vec<ProfileDto>, ApiError>;

    /// Retrieves a profile by username and target language (composite key).
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language of the profile
    ///
    /// # Returns
    ///
    /// * `Some(ProfileDto)` - The profile data if found
    /// * `None` - If no profile with the given composite key exists
    fn get_profile_by_username_and_target_language(
        &self,
        username: &str,
        target_language: &str,
    ) -> Option<ProfileDto>;

    /// Creates a new learning profile for a user.
    ///
    /// This will create a new database file at `data/{username}/{target_language}_profile.db`
    /// for storing the profile's learning data.
    ///
    /// # Arguments
    ///
    /// * `username` - The username this profile belongs to
    /// * `target_language` - The language being learned in this profile
    ///
    /// # Returns
    ///
    /// * `Ok(ProfileDto)` - The created profile
    /// * `Err(ApiError)` - If an error occurs (e.g., profile already exists or database creation fails)
    ///
    /// # Errors
    ///
    /// This function may return an error if:
    /// - A profile for this language already exists for the user
    /// - The target language is invalid
    /// - There's a database or filesystem error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::profiles_api::ProfilesApi;
    /// fn add_profile(api: &dyn ProfilesApi, username: &str, target_language: &str) -> Result<(), Box<dyn std::error::Error>> {
    ///     match api.create_profile(username, target_language) {
    ///         Ok(profile) => println!("Created profile: {:?}", profile),
    ///         Err(e) => eprintln!("Failed to create profile: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn create_profile(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<ProfileDto, ApiError>;

    /// Deletes a profile by username and target language.
    ///
    /// This will delete the associated profile database file.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language of the profile
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the profile was successfully deleted
    /// * `Ok(false)` - If no profile with the given composite key exists
    /// * `Err(ApiError)` - If an error occurs during deletion
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if there's a problem accessing the data store
    /// or deleting the database file.
    fn delete_profile(&self, username: &str, target_language: &str) -> Result<bool, ApiError>;

    /// Updates the last activity timestamp for a profile.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language of the profile
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update was successful
    /// * `Err(ApiError)` - If an error occurs during the update
    fn update_last_activity(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<(), ApiError>;
}
