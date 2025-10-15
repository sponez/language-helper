//! User service implementation.
//!
//! This module provides the business logic for user operations.
//! It uses the UserRepository trait for persistence operations.

use crate::errors::CoreError;
use crate::models::user::User;
use crate::repositories::user_repository::UserRepository;

/// Service for user business logic.
///
/// This struct implements the core business logic for user operations,
/// delegating persistence operations to a UserRepository implementation.
///
/// # Type Parameters
///
/// * `R` - The repository implementation type
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::user_service::UserService;
/// use lh_core::repositories::user_repository::UserRepository;
///
/// async fn example(repository: impl UserRepository) {
///     let service = UserService::new(repository);
///
///     match service.get_all_usernames().await {
///         Ok(usernames) => println!("Found {} users", usernames.len()),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub struct UserService<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> UserService<R> {
    /// Creates a new UserService instance.
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository implementation for persistence operations
    ///
    /// # Returns
    ///
    /// A new `UserService` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lh_core::services::user_service::UserService;
    /// use lh_core::repositories::user_repository::UserRepository;
    ///
    /// fn create_service(repo: impl UserRepository) {
    ///     let service = UserService::new(repo);
    /// }
    /// ```
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Retrieves all usernames from the system.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - A vector containing all usernames
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::user_service::UserService;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserService<impl UserRepository>) {
    /// match service.get_all_usernames().await {
    ///     Ok(usernames) => {
    ///         for username in usernames {
    ///             println!("User: {}", username);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Failed to get usernames: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn get_all_usernames(&self) -> Result<Vec<String>, CoreError> {
        let users = self.repository.find_all().await?;
        Ok(users.into_iter().map(|user| user.username).collect())
    }

    /// Retrieves a user by their username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(User))` - The user if found
    /// * `Ok(None)` - If no user with the given username exists
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::user_service::UserService;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserService<impl UserRepository>) {
    /// match service.get_user_by_username("john_doe").await {
    ///     Ok(Some(user)) => println!("Found user: {:?}", user),
    ///     Ok(None) => println!("User not found"),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, CoreError> {
        self.repository.find_by_username(username).await
    }

    /// Creates a new user.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the new user
    ///
    /// # Returns
    ///
    /// * `Ok(User)` - The newly created user
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::ValidationError` if the user data is invalid,
    /// or `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::user_service::UserService;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserService<impl UserRepository>) {
    /// match service.create_user("jane_doe").await {
    ///     Ok(user) => println!("Created user: {:?}", user),
    ///     Err(e) => eprintln!("Failed to create user: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn create_user(&self, username: &str) -> Result<User, CoreError> {
        // Domain validation happens in User::new()
        let user = User::new(username)?;

        // Business logic: check if user already exists
        if self
            .repository
            .find_by_username(&user.username)
            .await?
            .is_some()
        {
            return Err(CoreError::validation_error(format!(
                "User with username '{}' already exists",
                user.username
            )));
        }

        self.repository.save(user).await
    }

    /// Updates an existing user.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the user to update
    ///
    /// # Returns
    ///
    /// * `Ok(User)` - The updated user
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the user doesn't exist,
    /// `CoreError::ValidationError` if the user data is invalid,
    /// or `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::user_service::UserService;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserService<impl UserRepository>) {
    /// match service.update_user("john_doe").await {
    ///     Ok(user) => println!("Updated user: {:?}", user),
    ///     Err(e) => eprintln!("Failed to update user: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn update_user(&self, username: &str) -> Result<User, CoreError> {
        // Business logic: ensure user exists
        if self.repository.find_by_username(username).await?.is_none() {
            return Err(CoreError::not_found("User", username));
        }

        // Domain validation happens in User::new()
        let user = User::new(username)?;
        self.repository.save(user).await
    }

    /// Deletes a user by username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the user to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the user was successfully deleted
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the user doesn't exist,
    /// or `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::user_service::UserService;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserService<impl UserRepository>) {
    /// match service.delete_user("john_doe").await {
    ///     Ok(()) => println!("User deleted successfully"),
    ///     Err(e) => eprintln!("Failed to delete user: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn delete_user(&self, username: &str) -> Result<(), CoreError> {
        let deleted = self.repository.delete(username).await?;
        if !deleted {
            return Err(CoreError::not_found("User", username));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock repository for testing
    struct MockUserRepository {
        users: std::sync::Mutex<Vec<User>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn with_users(users: Vec<User>) -> Self {
            Self {
                users: std::sync::Mutex::new(users),
            }
        }
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_username(&self, username: &str) -> Result<Option<User>, CoreError> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().find(|u| u.username == username).cloned())
        }

        async fn find_all(&self) -> Result<Vec<User>, CoreError> {
            Ok(self.users.lock().unwrap().clone())
        }

        async fn save(&self, user: User) -> Result<User, CoreError> {
            let mut users = self.users.lock().unwrap();
            if let Some(pos) = users.iter().position(|u| u.username == user.username) {
                users[pos] = user.clone();
            } else {
                users.push(user.clone());
            }
            Ok(user)
        }

        async fn delete(&self, username: &str) -> Result<bool, CoreError> {
            let mut users = self.users.lock().unwrap();
            if let Some(pos) = users.iter().position(|u| u.username == username) {
                users.remove(pos);
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    #[tokio::test]
    async fn test_get_all_usernames() {
        let users = vec![
            User::new_unchecked("user1".to_string()),
            User::new_unchecked("user2".to_string()),
        ];
        let repo = MockUserRepository::with_users(users);
        let service = UserService::new(repo);

        let usernames = service.get_all_usernames().await.unwrap();
        assert_eq!(usernames.len(), 2);
        assert!(usernames.contains(&"user1".to_string()));
        assert!(usernames.contains(&"user2".to_string()));
    }

    #[tokio::test]
    async fn test_get_user_by_username_found() {
        let user = User::new_unchecked("john_doe".to_string());
        let repo = MockUserRepository::with_users(vec![user.clone()]);
        let service = UserService::new(repo);

        let result = service.get_user_by_username("john_doe").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().username, "john_doe");
    }

    #[tokio::test]
    async fn test_get_user_by_username_not_found() {
        let repo = MockUserRepository::new();
        let service = UserService::new(repo);

        let result = service.get_user_by_username("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let repo = MockUserRepository::new();
        let service = UserService::new(repo);

        let result = service.create_user("new_user").await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "new_user");
    }

    #[tokio::test]
    async fn test_create_user_empty_username() {
        let repo = MockUserRepository::new();
        let service = UserService::new(repo);

        let result = service.create_user("").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CoreError::ValidationError { message } => {
                assert!(message.contains("Username cannot be empty"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_create_user_duplicate_username() {
        let existing_user = User::new_unchecked("existing".to_string());
        let repo = MockUserRepository::with_users(vec![existing_user]);
        let service = UserService::new(repo);

        let result = service.create_user("existing").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CoreError::ValidationError { message } => {
                assert!(message.contains("already exists"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        let repo = MockUserRepository::new();
        let service = UserService::new(repo);

        let result = service.update_user("nonexistent").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CoreError::NotFound { entity, id } => {
                assert_eq!(entity, "User");
                assert_eq!(id, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_delete_user_success() {
        let user = User::new_unchecked("delete_me".to_string());
        let repo = MockUserRepository::with_users(vec![user]);
        let service = UserService::new(repo);

        let result = service.delete_user("delete_me").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let repo = MockUserRepository::new();
        let service = UserService::new(repo);

        let result = service.delete_user("nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CoreError::NotFound { entity, id } => {
                assert_eq!(entity, "User");
                assert_eq!(id, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }
}
