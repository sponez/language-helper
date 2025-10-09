//! AppApi trait implementation.
//!
//! This module provides the concrete implementation of the AppApi trait,
//! aggregating all API implementations.

use lh_api::app_api::AppApi;
use lh_api::apis::user_api::UsersApi;

use crate::api_impl::users_api_impl::UsersApiImpl;
use crate::repositories::user_repository::UserRepository;

/// Implementation of the AppApi trait.
///
/// This struct holds all API implementations and provides access to them
/// through the AppApi trait interface.
pub struct AppApiImpl<R: UserRepository> {
    users_api: UsersApiImpl<R>,
}

impl<R: UserRepository> AppApiImpl<R> {
    /// Creates a new AppApiImpl instance.
    ///
    /// # Arguments
    ///
    /// * `users_api` - The users API implementation
    ///
    /// # Returns
    ///
    /// A new `AppApiImpl` instance.
    pub fn new(users_api: UsersApiImpl<R>) -> Self {
        Self { users_api }
    }
}

impl<R: UserRepository + 'static> AppApi for AppApiImpl<R> {
    fn users_api(&self) -> &dyn UsersApi {
        &self.users_api
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::User;
    use crate::repositories::user_repository::UserRepository;
    use crate::services::user_service::UserService;
    use crate::errors::CoreError;

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

    #[test]
    fn test_app_api_impl_creation() {
        let repo = MockUserRepository;
        let service = UserService::new(repo);
        let users_api = UsersApiImpl::new(service);
        let app_api = AppApiImpl::new(users_api);

        // Verify users_api is accessible
        let usernames = app_api.users_api().get_usernames();
        assert!(usernames.is_ok());
    }

    #[test]
    fn test_app_api_impl_users_api_integration() {
        let repo = MockUserRepository;
        let service = UserService::new(repo);
        let users_api = UsersApiImpl::new(service);
        let app_api = AppApiImpl::new(users_api);

        // Test that we can call methods through the trait
        let result = app_api.users_api().get_user_by_username("test");
        assert!(result.is_some());
        assert_eq!(result.unwrap().username, "test");
    }
}
