//! AppApi trait implementation.
//!
//! This module provides the concrete implementation of the AppApi trait,
//! aggregating all API implementations.

use lh_api::app_api::AppApi;
use lh_api::apis::app_settings_api::AppSettingsApi;
use lh_api::apis::user_api::UsersApi;

use crate::api_impl::app_settings_api_impl::AppSettingsApiImpl;
use crate::api_impl::users_api_impl::UsersApiImpl;
use crate::repositories::app_settings_repository::AppSettingsRepository;
use crate::repositories::profile_repository::ProfileRepository;
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
    PR: ProfileRepository,
> {
    users_api: UsersApiImpl<UR, USR, ASR, PR>,
    app_settings_api: AppSettingsApiImpl<ASR>,
}

impl<
        UR: UserRepository,
        ASR: AppSettingsRepository,
        USR: UserSettingsRepository,
        PR: ProfileRepository,
    > AppApiImpl<UR, ASR, USR, PR>
{
    /// Creates a new AppApiImpl instance.
    ///
    /// # Arguments
    ///
    /// * `users_api` - The users API implementation
    /// * `app_settings_api` - The app settings API implementation
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
        PR: ProfileRepository + 'static,
    > AppApi for AppApiImpl<UR, ASR, USR, PR>
{
    fn users_api(&self) -> &dyn UsersApi {
        &self.users_api
    }

    fn app_settings_api(&self) -> &dyn AppSettingsApi {
        &self.app_settings_api
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::app_settings::AppSettings;
    use crate::domain::profile::Profile;
    use crate::domain::user::User;
    use crate::domain::user_settings::UserSettings;
    use crate::errors::CoreError;
    use crate::repositories::app_settings_repository::AppSettingsRepository;
    use crate::repositories::profile_repository::ProfileRepository;
    use crate::repositories::user_repository::UserRepository;
    use crate::repositories::user_settings_repository::UserSettingsRepository;
    use crate::services::app_settings_service::AppSettingsService;
    use crate::services::profile_service::ProfileService;
    use crate::services::user_service::UserService;
    use crate::services::user_settings_service::UserSettingsService;

    /// Mock repository for testing
    struct MockUserRepository;

    impl UserRepository for MockUserRepository {
        fn find_all(&self) -> Result<Vec<User>, CoreError> {
            Ok(vec![User::new_unchecked("test".to_string())])
        }

        fn find_by_username(&self, username: &str) -> Result<Option<User>, CoreError> {
            if username == "test" {
                Ok(Some(User::new_unchecked("test".to_string())))
            } else {
                Ok(None)
            }
        }

        fn save(&self, user: User) -> Result<User, CoreError> {
            Ok(user)
        }

        fn delete(&self, _username: &str) -> Result<bool, CoreError> {
            Ok(true)
        }
    }

    struct MockAppSettingsRepository;

    impl AppSettingsRepository for MockAppSettingsRepository {
        fn get(&self) -> Result<AppSettings, CoreError> {
            Ok(AppSettings::default())
        }

        fn update(&self, settings: AppSettings) -> Result<AppSettings, CoreError> {
            Ok(settings)
        }
    }

    struct MockUserSettingsRepository;

    impl UserSettingsRepository for MockUserSettingsRepository {
        fn find_by_username(&self, _username: &str) -> Result<Option<UserSettings>, CoreError> {
            Ok(None)
        }

        fn save(&self, settings: UserSettings) -> Result<UserSettings, CoreError> {
            Ok(settings)
        }

        fn delete(&self, _username: &str) -> Result<bool, CoreError> {
            Ok(true)
        }
    }

    struct MockProfileRepository;

    impl ProfileRepository for MockProfileRepository {
        fn find_by_id(&self, _profile_id: &str) -> Result<Option<Profile>, CoreError> {
            Ok(None)
        }

        fn find_by_username(&self, _username: &str) -> Result<Vec<Profile>, CoreError> {
            Ok(vec![])
        }

        fn find_all(&self) -> Result<Vec<Profile>, CoreError> {
            Ok(vec![])
        }

        fn save(&self, profile: Profile) -> Result<Profile, CoreError> {
            Ok(profile)
        }

        fn delete(&self, _profile_id: &str) -> Result<bool, CoreError> {
            Ok(true)
        }
    }

    #[test]
    fn test_app_api_impl_creation() {
        let user_repo = MockUserRepository;
        let app_settings_repo = MockAppSettingsRepository;
        let user_settings_repo = MockUserSettingsRepository;
        let profile_repo = MockProfileRepository;

        let users_api = UsersApiImpl::new(
            UserService::new(user_repo),
            UserSettingsService::new(user_settings_repo, MockAppSettingsRepository, MockUserRepository),
            ProfileService::new(profile_repo, MockUserRepository),
        );
        let app_settings_api = AppSettingsApiImpl::new(AppSettingsService::new(app_settings_repo));

        let app_api = AppApiImpl::new(users_api, app_settings_api);

        // Verify all APIs are accessible
        let usernames = app_api.users_api().get_usernames();
        assert!(usernames.is_ok());

        let settings = app_api.app_settings_api().get_app_settings();
        assert!(settings.is_ok());
    }

    #[test]
    fn test_app_api_impl_users_api_integration() {
        let user_repo = MockUserRepository;
        let app_settings_repo = MockAppSettingsRepository;
        let user_settings_repo = MockUserSettingsRepository;
        let profile_repo = MockProfileRepository;

        let users_api = UsersApiImpl::new(
            UserService::new(user_repo),
            UserSettingsService::new(user_settings_repo, MockAppSettingsRepository, MockUserRepository),
            ProfileService::new(profile_repo, MockUserRepository),
        );
        let app_settings_api = AppSettingsApiImpl::new(AppSettingsService::new(app_settings_repo));

        let app_api = AppApiImpl::new(users_api, app_settings_api);

        // Test that we can call methods through the trait
        let result = app_api.users_api().get_user_by_username("test");
        assert!(result.is_some());
        let user_dto = result.unwrap();
        assert_eq!(user_dto.username, "test");
        assert_eq!(user_dto.settings.username, "test");
        assert!(user_dto.profiles.is_empty());
    }

    #[test]
    fn test_app_api_impl_all_apis_accessible() {
        let user_repo = MockUserRepository;
        let app_settings_repo = MockAppSettingsRepository;
        let user_settings_repo = MockUserSettingsRepository;
        let profile_repo = MockProfileRepository;

        let users_api = UsersApiImpl::new(
            UserService::new(user_repo),
            UserSettingsService::new(user_settings_repo, MockAppSettingsRepository, MockUserRepository),
            ProfileService::new(profile_repo, MockUserRepository),
        );
        let app_settings_api = AppSettingsApiImpl::new(AppSettingsService::new(app_settings_repo));

        let app_api = AppApiImpl::new(users_api, app_settings_api);

        // Test each API is accessible
        assert!(app_api.users_api().get_usernames().is_ok());
        assert!(app_api.app_settings_api().get_app_settings().is_ok());
    }
}
