//! Application API trait definition.
//!
//! This module defines the main application API trait that aggregates all API endpoints.

use crate::apis::ai_assistant_api::AiAssistantApi;
use crate::apis::app_settings_api::AppSettingsApi;
use crate::apis::profiles_api::ProfilesApi;
use crate::apis::system_requirements_api::SystemRequirementsApi;
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

    /// Returns a reference to the Application Settings API.
    ///
    /// # Returns
    ///
    /// A reference to an object implementing the `AppSettingsApi` trait.
    fn app_settings_api(&self) -> &dyn AppSettingsApi;

    /// Returns a reference to the Profiles API.
    ///
    /// # Returns
    ///
    /// A reference to an object implementing the `ProfilesApi` trait.
    fn profile_api(&self) -> &dyn ProfilesApi;

    /// Returns a reference to the System Requirements API.
    ///
    /// # Returns
    ///
    /// A reference to an object implementing the `SystemRequirementsApi` trait.
    fn system_requirements_api(&self) -> &dyn SystemRequirementsApi;

    /// Returns a reference to the AI Assistant API.
    ///
    /// # Returns
    ///
    /// A reference to an object implementing the `AiAssistantApi` trait.
    fn ai_assistant_api(&self) -> &dyn AiAssistantApi;
}
