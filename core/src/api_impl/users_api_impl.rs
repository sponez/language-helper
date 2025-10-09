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
