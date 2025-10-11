//! AppApi trait implementation.
//!
//! This module provides the concrete implementation of the AppApi trait,
//! aggregating all API implementations.

use lh_api::apis::app_settings_api::AppSettingsApi;
use lh_api::apis::profiles_api::ProfilesApi;
use lh_api::apis::user_api::UsersApi;
use lh_api::app_api::AppApi;

use crate::api_impl::app_settings_api_impl::AppSettingsApiImpl;
use crate::api_impl::users_api_impl::UsersApiImpl;
use crate::repositories::app_settings_repository::AppSettingsRepository;
use crate::repositories::user_profiles_repository::UserProfilesRepository;
use crate::repositories::user_repository::UserRepository;
use crate::repositories::user_settings_repository::UserSettingsRepository;

/// Implementation of the AppApi trait.
///
/// This struct holds all API implementations and provides access to them
/// through the AppApi trait interface.
///
/// # Type Parameters
///
/// * `UR` - User repository implementation type
/// * `ASR` - App settings repository implementation type
/// * `USR` - User settings repository implementation type
/// * `PR` - Profile repository implementation type
pub struct AppApiImpl<
    UR: UserRepository,
    ASR: AppSettingsRepository,
    USR: UserSettingsRepository,
    PR: UserProfilesRepository,
> {
    users_api: UsersApiImpl<UR, USR, ASR, PR>,
    app_settings_api: AppSettingsApiImpl<ASR>,
}

impl<
        UR: UserRepository,
        ASR: AppSettingsRepository,
        USR: UserSettingsRepository,
        PR: UserProfilesRepository,
    > AppApiImpl<UR, ASR, USR, PR>
{
    /// Creates a new AppApiImpl instance.
    ///
    /// # Arguments
    ///
    /// * `users_api` - The users API implementation
    /// * `app_settings_api` - The app settings API implementation
    /// * `profiles_api` - The profiles API implementation
    ///
    /// # Returns
    ///
    /// A new `AppApiImpl` instance.
    pub fn new(
        users_api: UsersApiImpl<UR, USR, ASR, PR>,
        app_settings_api: AppSettingsApiImpl<ASR>,
    ) -> Self {
        Self {
            users_api,
            app_settings_api,
        }
    }
}

impl<
        UR: UserRepository + 'static,
        ASR: AppSettingsRepository + 'static,
        USR: UserSettingsRepository + 'static,
        PR: UserProfilesRepository + 'static,
    > AppApi for AppApiImpl<UR, ASR, USR, PR>
{
    fn users_api(&self) -> &dyn UsersApi {
        &self.users_api
    }

    fn app_settings_api(&self) -> &dyn AppSettingsApi {
        &self.app_settings_api
    }

    fn profile_api(&self) -> &dyn ProfilesApi {
        // TODO: Implement ProfileService for managing learning content in profile databases
        // ProfileService will handle operations on data/{username}/{language}_profile.db files
        // For now, profile metadata is managed through UsersApi -> UserProfilesService
        todo!("ProfileService not yet implemented - will manage vocabulary cards and learning content")
    }
}
