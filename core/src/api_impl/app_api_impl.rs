//! AppApi trait implementation.
//!
//! This module provides the concrete implementation of the AppApi trait,
//! aggregating all API implementations.

use lh_api::apis::app_settings_api::AppSettingsApi;
use lh_api::apis::profiles_api::ProfilesApi;
use lh_api::apis::user_api::UsersApi;
use lh_api::app_api::AppApi;

use crate::api_impl::app_settings_api_impl::AppSettingsApiImpl;
use crate::api_impl::profiles_api_impl::ProfilesApiImpl;
use crate::api_impl::users_api_impl::UsersApiImpl;
use crate::repositories::app_settings_repository::AppSettingsRepository;
use crate::repositories::profile_repository::ProfileRepository;
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
/// * `PR` - Profile metadata repository implementation type
/// * `PDR` - Profile database repository implementation type
pub struct AppApiImpl<
    UR: UserRepository,
    ASR: AppSettingsRepository,
    USR: UserSettingsRepository,
    PR: UserProfilesRepository,
    PDR: ProfileRepository,
> {
    users_api: UsersApiImpl<UR, USR, ASR, PR>,
    app_settings_api: AppSettingsApiImpl<ASR>,
    profiles_api: ProfilesApiImpl<PDR>,
}

impl<
        UR: UserRepository,
        ASR: AppSettingsRepository,
        USR: UserSettingsRepository,
        PR: UserProfilesRepository,
        PDR: ProfileRepository,
    > AppApiImpl<UR, ASR, USR, PR, PDR>
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
        profiles_api: ProfilesApiImpl<PDR>,
    ) -> Self {
        Self {
            users_api,
            app_settings_api,
            profiles_api,
        }
    }
}

impl<
        UR: UserRepository + 'static,
        ASR: AppSettingsRepository + 'static,
        USR: UserSettingsRepository + 'static,
        PR: UserProfilesRepository + 'static,
        PDR: ProfileRepository + 'static,
    > AppApi for AppApiImpl<UR, ASR, USR, PR, PDR>
{
    fn users_api(&self) -> &dyn UsersApi {
        &self.users_api
    }

    fn app_settings_api(&self) -> &dyn AppSettingsApi {
        &self.app_settings_api
    }

    fn profile_api(&self) -> &dyn ProfilesApi {
        &self.profiles_api
    }
}
