//! User management API.
//!
//! This module provides the trait definition for user-related operations.

use crate::{errors::api_error::ApiError, models::{profile::ProfileDto, user::UserDto}};

/// API for managing and retrieving user data.
///
/// This trait defines the interface for user-related operations such as
/// retrieving usernames and fetching user details.
///
/// # Examples
///
/// ```no_run
/// use lh_api::apis::user_api::UsersApi;
///
/// fn list_all_users(api: &dyn UsersApi) -> Result<(), Box<dyn std::error::Error>> {
///     let usernames = api.get_usernames()?;
///     for username in usernames {
///         println!("User: {}", username);
///     }
///     Ok(())
/// }
/// ```
pub trait UsersApi {
    /// Retrieves a list of all usernames.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - A vector containing all usernames
    /// * `Err(ApiError)` - If an error occurs while retrieving the usernames
    ///
    /// # Errors
    ///
    /// This function may return an error if the underlying data source is unavailable
    /// or if there's a permission issue.
    fn get_usernames(&self) -> Result<Vec<String>, ApiError>;

    /// Retrieves a user by their username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the user to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(UserDto)` - The user data if found
    /// * `None` - If no user with the given username exists
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::user_api::UsersApi;
    /// fn find_user(api: &dyn UsersApi, name: &str) {
    ///     match api.get_user_by_username(name) {
    ///         Some(user) => println!("Found user: {:?}", user),
    ///         None => println!("User not found"),
    ///     }
    /// }
    /// ```
    fn get_user_by_username(&self, username: &str) -> Option<UserDto>;

    /// Creates a new user with the given username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the new user
    ///
    /// # Returns
    ///
    /// * `Ok(UserDto)` - The created user
    /// * `Err(ApiError)` - If an error occurs (e.g., username already exists or is invalid)
    ///
    /// # Errors
    ///
    /// This function may return an error if:
    /// - The username is empty or invalid
    /// - A user with the same username already exists
    /// - There's a database or internal error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::user_api::UsersApi;
    /// fn add_user(api: &dyn UsersApi, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    ///     match api.create_user(name) {
    ///         Ok(user) => println!("Created user: {:?}", user),
    ///         Err(e) => eprintln!("Failed to create user: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn create_user(&self, username: &str) -> Result<UserDto, ApiError>;

    /// Updates a user's theme preference.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the user to update
    /// * `theme` - The new theme value
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update was successful
    /// * `Err(ApiError)` - If an error occurs during the update
    ///
    /// # Errors
    ///
    /// This function may return an error if:
    /// - The user doesn't exist
    /// - There's a database or internal error
    fn update_user_theme(&self, username: &str, theme: &str) -> Result<(), ApiError>;

    /// Updates a user's language preference.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the user to update
    /// * `language` - The new language value
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update was successful
    /// * `Err(ApiError)` - If an error occurs during the update
    ///
    /// # Errors
    ///
    /// This function may return an error if:
    /// - The user doesn't exist
    /// - There's a database or internal error
    fn update_user_language(&self, username: &str, language: &str) -> Result<(), ApiError>;

    /// Deletes a user and all associated data.
    ///
    /// This will delete the user's settings, profiles, and all profile databases.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the user to delete
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the user was successfully deleted
    /// * `Ok(false)` - If no user with the given username exists
    /// * `Err(ApiError)` - If an error occurs during deletion
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if there's a problem accessing the data store
    /// or deleting associated files.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::user_api::UsersApi;
    /// fn remove_user(api: &dyn UsersApi, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    ///     match api.delete_user(name) {
    ///         Ok(true) => println!("User deleted successfully"),
    ///         Ok(false) => println!("User not found"),
    ///         Err(e) => eprintln!("Failed to delete user: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn delete_user(&self, username: &str) -> Result<bool, ApiError>;

    /// Creates a new learning profile for a user.
    ///
    /// This will create both the profile metadata and the profile database file.
    ///
    /// # Arguments
    ///
    /// * `username` - The username this profile belongs to
    /// * `target_language` - The language being learned in this profile
    ///
    /// # Returns
    ///
    /// * `Ok(ProfileDto)` - The created profile
    /// * `Err(ApiError)` - If an error occurs
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::user_api::UsersApi;
    /// fn add_profile(api: &dyn UsersApi, username: &str) -> Result<(), Box<dyn std::error::Error>> {
    ///     match api.create_profile(username, "spanish") {
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

    /// Deletes a profile and its associated database file.
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
    fn delete_profile(&self, username: &str, target_language: &str) -> Result<bool, ApiError>;
}
