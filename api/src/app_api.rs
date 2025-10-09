//! Application API trait definition.
//!
//! This module defines the main application API trait that aggregates all API endpoints.

use crate::apis::user_api::UsersApi;

/// The main application API trait.
///
/// This trait serves as the primary entry point for accessing various API endpoints.
/// It must be thread-safe (`Send + Sync`) to support concurrent access.
///
/// # Examples
///
/// ```no_run
/// use lh_api::app_api::AppApi;
///
/// fn process_users(api: &dyn AppApi) {
///     let usernames = api.users_api().get_usernames();
///     // Process usernames...
/// }
/// ```
pub trait AppApi: Send + Sync {
    /// Returns a reference to the Users API.
    ///
    /// # Returns
    ///
    /// A reference to an object implementing the `UsersApi` trait.
    fn users_api(&self) -> &dyn UsersApi;
}
