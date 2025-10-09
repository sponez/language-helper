//! User repository adapter for mapping persistence errors to core errors.
//!
//! This adapter wraps a persistence layer repository and implements the
//! `UserRepository` trait. It's generic over the concrete persistence
//! implementation to avoid circular dependencies.

use crate::domain::user::User;
use crate::errors::CoreError;
use crate::repositories::user_repository::UserRepository;

/// Trait representing a persistence-layer user repository.
///
/// This trait defines the interface that persistence implementations must provide.
/// It's separate from the core's `UserRepository` to maintain clean architecture.
pub trait PersistenceUserRepository {
    /// The error type returned by this repository.
    type Error: std::fmt::Display;

    /// Finds a user by username.
    fn find_by_username(&self, username: &str) -> Result<Option<User>, Self::Error>;

    /// Retrieves all users.
    fn find_all(&self) -> Result<Vec<User>, Self::Error>;

    /// Saves a user.
    fn save(&self, user: User) -> Result<User, Self::Error>;

    /// Deletes a user by username.
    fn delete(&self, username: &str) -> Result<bool, Self::Error>;
}

/// Adapter that wraps a persistence repository and maps errors.
///
/// This adapter maintains clean architecture by:
/// - Implementing the core's `UserRepository` trait
/// - Wrapping any persistence layer implementation
/// - Mapping persistence errors to `CoreError`
///
/// # Type Parameters
///
/// * `R` - The concrete persistence repository type
///
/// # Examples
///
/// ```ignore
/// use lh_core::repositories::adapters::UserRepositoryAdapter;
/// use lh_persistence::repositories::sqlite_user_repository::SqliteUserRepository;
///
/// let persistence_repo = SqliteUserRepository::new(":memory:").unwrap();
/// let adapter = UserRepositoryAdapter::new(persistence_repo);
/// ```
pub struct UserRepositoryAdapter<R> {
    repository: R,
}

impl<R> UserRepositoryAdapter<R> {
    /// Creates a new adapter wrapping a persistence repository.
    ///
    /// # Arguments
    ///
    /// * `repository` - The persistence layer repository to wrap
    ///
    /// # Returns
    ///
    /// A new `UserRepositoryAdapter` instance.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: PersistenceUserRepository + Send + Sync> UserRepository for UserRepositoryAdapter<R> {
    fn find_all(&self) -> Result<Vec<User>, CoreError> {
        self.repository
            .find_all()
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    fn find_by_username(&self, username: &str) -> Result<Option<User>, CoreError> {
        self.repository
            .find_by_username(username)
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    fn save(&self, user: User) -> Result<User, CoreError> {
        self.repository
            .save(user)
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    fn delete(&self, username: &str) -> Result<bool, CoreError> {
        self.repository
            .delete(username)
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Mock persistence repository for testing the adapter.
    struct MockPersistenceRepository {
        users: Arc<Mutex<HashMap<String, User>>>,
        should_fail: bool,
    }

    impl MockPersistenceRepository {
        fn new() -> Self {
            Self {
                users: Arc::new(Mutex::new(HashMap::new())),
                should_fail: false,
            }
        }

        fn with_failure() -> Self {
            Self {
                users: Arc::new(Mutex::new(HashMap::new())),
                should_fail: true,
            }
        }
    }

    #[derive(Debug)]
    struct MockError(String);

    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Mock error: {}", self.0)
        }
    }

    impl PersistenceUserRepository for MockPersistenceRepository {
        type Error = MockError;

        fn find_by_username(&self, username: &str) -> Result<Option<User>, Self::Error> {
            if self.should_fail {
                return Err(MockError("Database error".to_string()));
            }
            Ok(self.users.lock().unwrap().get(username).cloned())
        }

        fn find_all(&self) -> Result<Vec<User>, Self::Error> {
            if self.should_fail {
                return Err(MockError("Database error".to_string()));
            }
            Ok(self.users.lock().unwrap().values().cloned().collect())
        }

        fn save(&self, user: User) -> Result<User, Self::Error> {
            if self.should_fail {
                return Err(MockError("Database error".to_string()));
            }
            self.users.lock().unwrap().insert(user.username.clone(), user.clone());
            Ok(user)
        }

        fn delete(&self, username: &str) -> Result<bool, Self::Error> {
            if self.should_fail {
                return Err(MockError("Database error".to_string()));
            }
            Ok(self.users.lock().unwrap().remove(username).is_some())
        }
    }

    #[test]
    fn test_adapter_creation() {
        let repo = MockPersistenceRepository::new();
        let adapter = UserRepositoryAdapter::new(repo);

        // Should be able to use as UserRepository trait
        let result = adapter.find_all();
        assert!(result.is_ok());
    }

    #[test]
    fn test_adapter_find_all() {
        let repo = MockPersistenceRepository::new();
        let adapter = UserRepositoryAdapter::new(repo);

        let users = adapter.find_all().unwrap();
        assert_eq!(users.len(), 0);
    }

    #[test]
    fn test_adapter_save_and_find() {
        let repo = MockPersistenceRepository::new();
        let adapter = UserRepositoryAdapter::new(repo);

        let user = User::new_unchecked("test_user".to_string());
        let saved = adapter.save(user.clone()).unwrap();
        assert_eq!(saved.username, "test_user");

        let found = adapter.find_by_username("test_user").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, "test_user");
    }

    #[test]
    fn test_adapter_delete() {
        let repo = MockPersistenceRepository::new();
        let adapter = UserRepositoryAdapter::new(repo);

        let user = User::new_unchecked("test_user".to_string());
        adapter.save(user).unwrap();

        let deleted = adapter.delete("test_user").unwrap();
        assert!(deleted);

        let found = adapter.find_by_username("test_user").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_adapter_delete_nonexistent() {
        let repo = MockPersistenceRepository::new();
        let adapter = UserRepositoryAdapter::new(repo);

        let deleted = adapter.delete("nonexistent").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_adapter_error_mapping() {
        let repo = MockPersistenceRepository::with_failure();
        let adapter = UserRepositoryAdapter::new(repo);

        // All operations should fail and map to CoreError
        let result = adapter.find_all();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::RepositoryError { .. }));

        let result = adapter.find_by_username("test");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::RepositoryError { .. }));

        let result = adapter.save(User::new_unchecked("test".to_string()));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::RepositoryError { .. }));

        let result = adapter.delete("test");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::RepositoryError { .. }));
    }
}
