//! UsersApi trait implementation.
//!
//! This module provides the concrete implementation of the UsersApi trait
//! using the UserService from the core layer.

use lh_api::apis::user_api::UsersApi;
use lh_api::errors::api_error::ApiError;
use lh_api::models::user::UserDto;

use crate::domain::user::User;
use crate::errors::CoreError;
use crate::services::user_service::UserService;
use crate::repositories::user_repository::UserRepository;

/// Helper function to map CoreError to ApiError
fn map_core_error_to_api_error(error: CoreError) -> ApiError {
    match error {
        CoreError::NotFound { entity, id } => {
            ApiError::not_found(format!("{} '{}' not found", entity, id))
        }
        CoreError::ValidationError { message } => {
            ApiError::validation_error(message)
        }
        CoreError::RepositoryError { message } => {
            ApiError::internal_error(format!("Internal error: {}", message))
        }
    }
}

/// Helper function to map domain User to UserDto
fn map_user_to_dto(user: User) -> UserDto {
    UserDto {
        username: user.username,
    }
}

/// Implementation of the UsersApi trait.
///
/// This struct delegates to the UserService to fulfill API requests,
/// converting between domain models and DTOs as needed.
pub struct UsersApiImpl<R: UserRepository> {
    user_service: UserService<R>,
}

impl<R: UserRepository> UsersApiImpl<R> {
    /// Creates a new UsersApiImpl instance.
    ///
    /// # Arguments
    ///
    /// * `user_service` - The user service instance
    ///
    /// # Returns
    ///
    /// A new `UsersApiImpl` instance.
    pub fn new(user_service: UserService<R>) -> Self {
        Self { user_service }
    }
}

impl<R: UserRepository> UsersApi for UsersApiImpl<R> {
    fn get_usernames(&self) -> Result<Vec<String>, ApiError> {
        self.user_service
            .get_all_usernames()
            .map_err(map_core_error_to_api_error)
    }

    fn get_user_by_username(&self, username: &str) -> Option<UserDto> {
        self.user_service
            .get_user_by_username(username)
            .ok()
            .flatten()
            .map(map_user_to_dto)
    }

    fn create_user(&self, username: String) -> Result<UserDto, ApiError> {
        self.user_service
            .create_user(username)
            .map(map_user_to_dto)
            .map_err(map_core_error_to_api_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::User;
    use crate::repositories::user_repository::UserRepository;

    /// Mock repository for testing
    struct MockUserRepository {
        users: Vec<User>,
        should_fail: bool,
    }

    impl UserRepository for MockUserRepository {
        fn find_all(&self) -> Result<Vec<User>, CoreError> {
            if self.should_fail {
                Err(CoreError::RepositoryError {
                    message: "Mock error".to_string(),
                })
            } else {
                Ok(self.users.clone())
            }
        }

        fn find_by_username(&self, username: &str) -> Result<Option<User>, CoreError> {
            if self.should_fail {
                Err(CoreError::RepositoryError {
                    message: "Mock error".to_string(),
                })
            } else {
                Ok(self.users.iter().find(|u| u.username == username).cloned())
            }
        }

        fn save(&self, user: User) -> Result<User, CoreError> {
            if self.should_fail {
                Err(CoreError::RepositoryError {
                    message: "Mock error".to_string(),
                })
            } else if self.users.iter().any(|u| u.username == user.username) {
                Err(CoreError::ValidationError {
                    message: "User already exists".to_string(),
                })
            } else {
                Ok(user)
            }
        }

        fn delete(&self, _username: &str) -> Result<bool, CoreError> {
            if self.should_fail {
                Err(CoreError::RepositoryError {
                    message: "Mock error".to_string(),
                })
            } else {
                Ok(true)
            }
        }
    }

    fn create_mock_users() -> Vec<User> {
        vec![
            User::new_unchecked("alice".to_string()),
            User::new_unchecked("bob".to_string()),
        ]
    }

    #[test]
    fn test_map_core_error_not_found() {
        let core_error = CoreError::NotFound {
            entity: "User".to_string(),
            id: "test".to_string(),
        };
        let api_error = map_core_error_to_api_error(core_error);

        match api_error {
            ApiError::Simple(code, _) => {
                assert!(matches!(code, lh_api::errors::api_error::ApiErrorCode::NotFound));
            }
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_map_core_error_validation() {
        let core_error = CoreError::ValidationError {
            message: "Invalid input".to_string(),
        };
        let api_error = map_core_error_to_api_error(core_error);

        match api_error {
            ApiError::Simple(code, _) => {
                assert!(matches!(code, lh_api::errors::api_error::ApiErrorCode::ValidationError));
            }
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_map_core_error_repository() {
        let core_error = CoreError::RepositoryError {
            message: "Database error".to_string(),
        };
        let api_error = map_core_error_to_api_error(core_error);

        match api_error {
            ApiError::Simple(code, _) => {
                assert!(matches!(code, lh_api::errors::api_error::ApiErrorCode::InternalError));
            }
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_map_user_to_dto() {
        let user = User::new_unchecked("testuser".to_string());
        let dto = map_user_to_dto(user);

        assert_eq!(dto.username, "testuser");
    }

    #[test]
    fn test_get_usernames_success() {
        let repo = MockUserRepository {
            users: create_mock_users(),
            should_fail: false,
        };
        let service = UserService::new(repo);
        let api = UsersApiImpl::new(service);

        let result = api.get_usernames();

        assert!(result.is_ok());
        let usernames = result.unwrap();
        assert_eq!(usernames.len(), 2);
        assert!(usernames.contains(&"alice".to_string()));
        assert!(usernames.contains(&"bob".to_string()));
    }

    #[test]
    fn test_get_usernames_repository_error() {
        let repo = MockUserRepository {
            users: vec![],
            should_fail: true,
        };
        let service = UserService::new(repo);
        let api = UsersApiImpl::new(service);

        let result = api.get_usernames();

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Simple(code, _) => {
                assert!(matches!(code, lh_api::errors::api_error::ApiErrorCode::InternalError));
            }
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_get_user_by_username_found() {
        let repo = MockUserRepository {
            users: create_mock_users(),
            should_fail: false,
        };
        let service = UserService::new(repo);
        let api = UsersApiImpl::new(service);

        let result = api.get_user_by_username("alice");

        assert!(result.is_some());
        assert_eq!(result.unwrap().username, "alice");
    }

    #[test]
    fn test_get_user_by_username_not_found() {
        let repo = MockUserRepository {
            users: create_mock_users(),
            should_fail: false,
        };
        let service = UserService::new(repo);
        let api = UsersApiImpl::new(service);

        let result = api.get_user_by_username("charlie");

        assert!(result.is_none());
    }

    #[test]
    fn test_create_user_success() {
        let repo = MockUserRepository {
            users: vec![],
            should_fail: false,
        };
        let service = UserService::new(repo);
        let api = UsersApiImpl::new(service);

        let result = api.create_user("newuser".to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().username, "newuser");
    }

    #[test]
    fn test_create_user_validation_error() {
        let repo = MockUserRepository {
            users: create_mock_users(),
            should_fail: false,
        };
        let service = UserService::new(repo);
        let api = UsersApiImpl::new(service);

        let result = api.create_user("alice".to_string());

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Simple(code, _) => {
                assert!(matches!(code, lh_api::errors::api_error::ApiErrorCode::ValidationError));
            }
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_create_user_empty_username() {
        let repo = MockUserRepository {
            users: vec![],
            should_fail: false,
        };
        let service = UserService::new(repo);
        let api = UsersApiImpl::new(service);

        let result = api.create_user("".to_string());

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Simple(code, _) => {
                assert!(matches!(code, lh_api::errors::api_error::ApiErrorCode::ValidationError));
            }
            _ => panic!("Expected Simple variant"),
        }
    }
}
