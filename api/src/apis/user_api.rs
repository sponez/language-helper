//! User management API.
//!
//! This module provides the trait definition for user-related operations.

use crate::{errors::api_error::ApiError, models::user::UserDto};

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
}
