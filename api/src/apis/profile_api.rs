//! Profile management API.
//!
//! This module provides the trait definition for profile-related operations.

use crate::{errors::api_error::ApiError, models::profile::ProfileDto};

/// API for managing learning profiles.
///
/// This trait defines the interface for profile-related operations such as
/// retrieving profiles, creating new profiles, updating activity, and deletion.
///
/// # Examples
///
/// ```no_run
/// use lh_api::apis::profile_api::ProfileApi;
///
/// fn list_user_profiles(api: &dyn ProfileApi, username: &str) -> Result<(), Box<dyn std::error::Error>> {
///     let profiles = api.get_profiles(username)?;
///     for profile in profiles {
///         println!("Profile: {} - {}", profile.id, profile.target_language);
///     }
///     Ok(())
/// }
/// ```
pub trait ProfileApi {
    /// Retrieves all profiles for a specific user.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ProfileDto>)` - A vector containing all profiles for the user
    /// * `Err(ApiError)` - If an error occurs while retrieving the profiles
    ///
    /// # Errors
    ///
    /// This function may return an error if the underlying data source is unavailable.
    fn get_profiles(&self, username: &str) -> Result<Vec<ProfileDto>, ApiError>;

    /// Retrieves a profile by its ID.
    ///
    /// # Arguments
    ///
    /// * `profile_id` - The profile ID to search for
    ///
    /// # Returns
    ///
    /// * `Ok(ProfileDto)` - The profile data if found
    /// * `Err(ApiError)` - If the profile doesn't exist or an error occurs
    ///
    /// # Errors
    ///
    /// This function may return:
    /// - `ApiError::NotFound` if the profile doesn't exist
    /// - `ApiError::InternalError` if there's a database error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::profile_api::ProfileApi;
    /// fn find_profile(api: &dyn ProfileApi, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    ///     match api.get_profile_by_id(id) {
    ///         Ok(profile) => println!("Found profile: {:?}", profile),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn get_profile_by_id(&self, profile_id: &str) -> Result<ProfileDto, ApiError>;

    /// Creates a new learning profile for a user.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the profile
    /// * `target_language` - The target language being learned
    ///
    /// # Returns
    ///
    /// * `Ok(ProfileDto)` - The created profile
    /// * `Err(ApiError)` - If an error occurs (e.g., user doesn't exist or validation fails)
    ///
    /// # Errors
    ///
    /// This function may return an error if:
    /// - The user doesn't exist
    /// - The username or target language is invalid
    /// - There's a database or internal error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::profile_api::ProfileApi;
    /// fn add_profile(api: &dyn ProfileApi) -> Result<(), Box<dyn std::error::Error>> {
    ///     match api.create_profile("john_doe".to_string(), "spanish".to_string()) {
    ///         Ok(profile) => println!("Created profile: {:?}", profile),
    ///         Err(e) => eprintln!("Failed to create profile: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn create_profile(
        &self,
        username: String,
        target_language: String,
    ) -> Result<ProfileDto, ApiError>;

    /// Updates the last activity timestamp for a profile.
    ///
    /// This should be called whenever a user interacts with a profile.
    ///
    /// # Arguments
    ///
    /// * `profile_id` - The profile ID to update
    ///
    /// # Returns
    ///
    /// * `Ok(ProfileDto)` - The updated profile
    /// * `Err(ApiError)` - If an error occurs (e.g., profile doesn't exist)
    ///
    /// # Errors
    ///
    /// This function may return:
    /// - `ApiError::NotFound` if the profile doesn't exist
    /// - `ApiError::InternalError` if there's a database error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::profile_api::ProfileApi;
    /// fn record_activity(api: &dyn ProfileApi, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    ///     match api.update_profile_activity(id) {
    ///         Ok(profile) => println!("Updated activity: {:?}", profile),
    ///         Err(e) => eprintln!("Failed to update: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn update_profile_activity(&self, profile_id: &str) -> Result<ProfileDto, ApiError>;

    /// Deletes a profile by ID.
    ///
    /// # Arguments
    ///
    /// * `profile_id` - The profile ID to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the profile was successfully deleted
    /// * `Err(ApiError)` - If an error occurs (e.g., profile doesn't exist)
    ///
    /// # Errors
    ///
    /// This function may return:
    /// - `ApiError::NotFound` if the profile doesn't exist
    /// - `ApiError::InternalError` if there's a database error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::profile_api::ProfileApi;
    /// fn remove_profile(api: &dyn ProfileApi, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    ///     match api.delete_profile(id) {
    ///         Ok(()) => println!("Profile deleted successfully"),
    ///         Err(e) => eprintln!("Failed to delete: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn delete_profile(&self, profile_id: &str) -> Result<(), ApiError>;
}
