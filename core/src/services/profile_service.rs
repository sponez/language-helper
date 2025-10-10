//! Profile service implementation.
//!
//! This module provides the business logic for learning profile operations.
//! It uses the ProfileRepository trait for persistence operations.

use crate::domain::profile::Profile;
use crate::errors::CoreError;
use crate::repositories::profile_repository::ProfileRepository;
use crate::repositories::user_repository::UserRepository;

/// Service for profile business logic.
///
/// This struct implements the core business logic for learning profiles,
/// delegating persistence operations to a ProfileRepository implementation.
///
/// # Type Parameters
///
/// * `PR` - The profile repository implementation type
/// * `UR` - The user repository implementation type
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::profile_service::ProfileService;
/// use lh_core::repositories::profile_repository::ProfileRepository;
/// use lh_core::repositories::user_repository::UserRepository;
///
/// fn example(profile_repo: impl ProfileRepository, user_repo: impl UserRepository) {
///     let service = ProfileService::new(profile_repo, user_repo);
///
///     match service.get_profiles_for_user("john_doe") {
///         Ok(profiles) => println!("Found {} profiles", profiles.len()),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub struct ProfileService<PR: ProfileRepository, UR: UserRepository> {
    profile_repository: PR,
    user_repository: UR,
}

impl<PR: ProfileRepository, UR: UserRepository> ProfileService<PR, UR> {
    /// Creates a new ProfileService instance.
    ///
    /// # Arguments
    ///
    /// * `profile_repository` - The profile repository implementation
    /// * `user_repository` - The user repository implementation
    ///
    /// # Returns
    ///
    /// A new `ProfileService` instance.
    pub fn new(profile_repository: PR, user_repository: UR) -> Self {
        Self {
            profile_repository,
            user_repository,
        }
    }

    /// Retrieves all profiles for a specific user.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Profile>)` - A vector of profiles
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
    /// # use lh_core::services::profile_service::ProfileService;
    /// # use lh_core::repositories::profile_repository::ProfileRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # fn example(service: &ProfileService<impl ProfileRepository, impl UserRepository>) {
    /// match service.get_profiles_for_user("john_doe") {
    ///     Ok(profiles) => {
    ///         for profile in profiles {
    ///             println!("Profile: {} - {}", profile.profile_id, profile.target_language);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Failed to get profiles: {}", e),
    /// }
    /// # }
    /// ```
    pub fn get_profiles_for_user(&self, username: &str) -> Result<Vec<Profile>, CoreError> {
        self.profile_repository.find_by_username(username)
    }

    /// Retrieves a profile by its ID.
    ///
    /// # Arguments
    ///
    /// * `profile_id` - The profile ID to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Profile)` - The profile if found
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the profile doesn't exist,
    /// or `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::profile_service::ProfileService;
    /// # use lh_core::repositories::profile_repository::ProfileRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # fn example(service: &ProfileService<impl ProfileRepository, impl UserRepository>) {
    /// match service.get_profile_by_id("profile_123") {
    ///     Ok(profile) => println!("Found profile: {:?}", profile),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// # }
    /// ```
    pub fn get_profile_by_id(&self, profile_id: &str) -> Result<Profile, CoreError> {
        self.profile_repository
            .find_by_id(profile_id)?
            .ok_or_else(|| CoreError::not_found("Profile", profile_id))
    }

    /// Creates a new learning profile for a user.
    ///
    /// This method validates that the user exists before creating the profile.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the profile
    /// * `target_language` - The target language being learned
    ///
    /// # Returns
    ///
    /// * `Ok(Profile)` - The newly created profile
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the user doesn't exist,
    /// `CoreError::ValidationError` if the profile data is invalid,
    /// or `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::profile_service::ProfileService;
    /// # use lh_core::repositories::profile_repository::ProfileRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # fn example(service: &ProfileService<impl ProfileRepository, impl UserRepository>) {
    /// match service.create_profile("jane_doe".to_string(), "spanish".to_string()) {
    ///     Ok(profile) => println!("Created profile: {:?}", profile),
    ///     Err(e) => eprintln!("Failed to create profile: {}", e),
    /// }
    /// # }
    /// ```
    pub fn create_profile(
        &self,
        username: String,
        target_language: String,
    ) -> Result<Profile, CoreError> {
        // Business logic: ensure user exists
        if self.user_repository.find_by_username(&username)?.is_none() {
            return Err(CoreError::not_found("User", &username));
        }

        // Domain validation happens in Profile::new()
        let profile = Profile::new(username, target_language)?;

        // Note: profile_id uniqueness is handled by UUID generation in Profile::new()
        // The repository will handle any database-level uniqueness constraints
        self.profile_repository.save(profile)
    }

    /// Updates the last activity timestamp for a profile.
    ///
    /// This method should be called whenever a user interacts with a profile.
    ///
    /// # Arguments
    ///
    /// * `profile_id` - The profile ID to update
    ///
    /// # Returns
    ///
    /// * `Ok(Profile)` - The updated profile
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the profile doesn't exist,
    /// or `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::profile_service::ProfileService;
    /// # use lh_core::repositories::profile_repository::ProfileRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # fn example(service: &ProfileService<impl ProfileRepository, impl UserRepository>) {
    /// match service.update_profile_activity("profile_123") {
    ///     Ok(profile) => println!("Updated activity: {:?}", profile),
    ///     Err(e) => eprintln!("Failed to update activity: {}", e),
    /// }
    /// # }
    /// ```
    pub fn update_profile_activity(&self, profile_id: &str) -> Result<Profile, CoreError> {
        let mut profile = self
            .profile_repository
            .find_by_id(profile_id)?
            .ok_or_else(|| CoreError::not_found("Profile", profile_id))?;

        profile.update_last_activity();
        self.profile_repository.save(profile)
    }

    /// Deletes a profile by ID.
    ///
    /// # Arguments
    ///
    /// * `profile_id` - The profile ID to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the profile was successfully deleted
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the profile doesn't exist,
    /// or `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::profile_service::ProfileService;
    /// # use lh_core::repositories::profile_repository::ProfileRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # fn example(service: &ProfileService<impl ProfileRepository, impl UserRepository>) {
    /// match service.delete_profile("profile_123") {
    ///     Ok(()) => println!("Profile deleted successfully"),
    ///     Err(e) => eprintln!("Failed to delete profile: {}", e),
    /// }
    /// # }
    /// ```
    pub fn delete_profile(&self, profile_id: &str) -> Result<(), CoreError> {
        let deleted = self.profile_repository.delete(profile_id)?;
        if !deleted {
            return Err(CoreError::not_found("Profile", profile_id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::User;

    // Mock repositories for testing
    struct MockProfileRepository {
        profiles: std::sync::Mutex<Vec<Profile>>,
    }

    impl MockProfileRepository {
        fn new() -> Self {
            Self {
                profiles: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn with_profiles(profiles: Vec<Profile>) -> Self {
            Self {
                profiles: std::sync::Mutex::new(profiles),
            }
        }
    }

    impl ProfileRepository for MockProfileRepository {
        fn find_by_id(&self, profile_id: &str) -> Result<Option<Profile>, CoreError> {
            let profiles = self.profiles.lock().unwrap();
            Ok(profiles
                .iter()
                .find(|p| p.profile_id == profile_id)
                .cloned())
        }

        fn find_by_username(&self, username: &str) -> Result<Vec<Profile>, CoreError> {
            let profiles = self.profiles.lock().unwrap();
            Ok(profiles
                .iter()
                .filter(|p| p.username == username)
                .cloned()
                .collect())
        }

        fn find_all(&self) -> Result<Vec<Profile>, CoreError> {
            Ok(self.profiles.lock().unwrap().clone())
        }

        fn save(&self, profile: Profile) -> Result<Profile, CoreError> {
            let mut profiles = self.profiles.lock().unwrap();
            if let Some(pos) = profiles
                .iter()
                .position(|p| p.profile_id == profile.profile_id)
            {
                profiles[pos] = profile.clone();
            } else {
                profiles.push(profile.clone());
            }
            Ok(profile)
        }

        fn delete(&self, profile_id: &str) -> Result<bool, CoreError> {
            let mut profiles = self.profiles.lock().unwrap();
            if let Some(pos) = profiles.iter().position(|p| p.profile_id == profile_id) {
                profiles.remove(pos);
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    struct MockUserRepository {
        users: std::sync::Mutex<Vec<User>>,
    }

    impl MockUserRepository {
        fn with_users(users: Vec<User>) -> Self {
            Self {
                users: std::sync::Mutex::new(users),
            }
        }
    }

    impl UserRepository for MockUserRepository {
        fn find_by_username(&self, username: &str) -> Result<Option<User>, CoreError> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().find(|u| u.username == username).cloned())
        }

        fn find_all(&self) -> Result<Vec<User>, CoreError> {
            Ok(self.users.lock().unwrap().clone())
        }

        fn save(&self, user: User) -> Result<User, CoreError> {
            let mut users = self.users.lock().unwrap();
            if let Some(pos) = users.iter().position(|u| u.username == user.username) {
                users[pos] = user.clone();
            } else {
                users.push(user.clone());
            }
            Ok(user)
        }

        fn delete(&self, username: &str) -> Result<bool, CoreError> {
            let mut users = self.users.lock().unwrap();
            if let Some(pos) = users.iter().position(|u| u.username == username) {
                users.remove(pos);
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    #[test]
    fn test_get_profiles_for_user() {
        let profile1 = Profile::new_unchecked(
            "profile_1".to_string(),
            "user1".to_string(),
            "spanish".to_string(),
            1000,
            1000,
        );
        let profile2 = Profile::new_unchecked(
            "profile_2".to_string(),
            "user1".to_string(),
            "french".to_string(),
            1000,
            1000,
        );
        let profile_repo = MockProfileRepository::with_profiles(vec![profile1, profile2]);
        let user_repo =
            MockUserRepository::with_users(vec![User::new_unchecked("user1".to_string())]);
        let service = ProfileService::new(profile_repo, user_repo);

        let result = service.get_profiles_for_user("user1").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_get_profile_by_id_found() {
        let profile = Profile::new_unchecked(
            "profile_123".to_string(),
            "user1".to_string(),
            "spanish".to_string(),
            1000,
            1000,
        );
        let profile_repo = MockProfileRepository::with_profiles(vec![profile]);
        let user_repo = MockUserRepository::with_users(vec![]);
        let service = ProfileService::new(profile_repo, user_repo);

        let result = service.get_profile_by_id("profile_123").unwrap();
        assert_eq!(result.profile_id, "profile_123");
        assert_eq!(result.target_language, "spanish");
    }

    #[test]
    fn test_get_profile_by_id_not_found() {
        let profile_repo = MockProfileRepository::new();
        let user_repo = MockUserRepository::with_users(vec![]);
        let service = ProfileService::new(profile_repo, user_repo);

        let result = service.get_profile_by_id("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::NotFound { .. }));
    }

    #[test]
    fn test_create_profile_success() {
        let profile_repo = MockProfileRepository::new();
        let user_repo =
            MockUserRepository::with_users(vec![User::new_unchecked("test_user".to_string())]);
        let service = ProfileService::new(profile_repo, user_repo);

        let result = service.create_profile("test_user".to_string(), "italian".to_string());
        assert!(result.is_ok());
        let profile = result.unwrap();
        assert_eq!(profile.username, "test_user");
        assert_eq!(profile.target_language, "italian");
        assert!(profile.profile_id.contains("test_user"));
    }

    #[test]
    fn test_create_profile_user_not_found() {
        let profile_repo = MockProfileRepository::new();
        let user_repo = MockUserRepository::with_users(vec![]);
        let service = ProfileService::new(profile_repo, user_repo);

        let result = service.create_profile("nonexistent".to_string(), "spanish".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::NotFound { .. }));
    }

    #[test]
    fn test_create_profile_invalid_language() {
        let profile_repo = MockProfileRepository::new();
        let user_repo =
            MockUserRepository::with_users(vec![User::new_unchecked("test_user".to_string())]);
        let service = ProfileService::new(profile_repo, user_repo);

        let result = service.create_profile("test_user".to_string(), "".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_update_profile_activity() {
        let profile = Profile::new_unchecked(
            "profile_123".to_string(),
            "user1".to_string(),
            "spanish".to_string(),
            1000,
            1000,
        );
        let profile_repo = MockProfileRepository::with_profiles(vec![profile.clone()]);
        let user_repo = MockUserRepository::with_users(vec![]);
        let service = ProfileService::new(profile_repo, user_repo);

        let result = service.update_profile_activity("profile_123");
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert!(updated.last_activity_at > profile.last_activity_at);
    }

    #[test]
    fn test_update_profile_activity_not_found() {
        let profile_repo = MockProfileRepository::new();
        let user_repo = MockUserRepository::with_users(vec![]);
        let service = ProfileService::new(profile_repo, user_repo);

        let result = service.update_profile_activity("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::NotFound { .. }));
    }

    #[test]
    fn test_delete_profile_success() {
        let profile = Profile::new_unchecked(
            "profile_123".to_string(),
            "user1".to_string(),
            "spanish".to_string(),
            1000,
            1000,
        );
        let profile_repo = MockProfileRepository::with_profiles(vec![profile]);
        let user_repo = MockUserRepository::with_users(vec![]);
        let service = ProfileService::new(profile_repo, user_repo);

        let result = service.delete_profile("profile_123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_profile_not_found() {
        let profile_repo = MockProfileRepository::new();
        let user_repo = MockUserRepository::with_users(vec![]);
        let service = ProfileService::new(profile_repo, user_repo);

        let result = service.delete_profile("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::NotFound { .. }));
    }
}
