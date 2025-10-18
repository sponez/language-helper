//! User profiles service implementation.
//!
//! This module provides the business logic for learning profile metadata operations.
//! It uses the UserProfilesRepository trait for persistence operations.

use crate::errors::CoreError;
use crate::models::profile::Profile;
use crate::repositories::user_profiles_repository::UserProfilesRepository;
use crate::repositories::user_repository::UserRepository;

/// Service for user profile metadata business logic.
///
/// This struct implements the core business logic for learning profile metadata,
/// delegating persistence operations to a UserProfilesRepository implementation.
///
/// # Type Parameters
///
/// * `PR` - The user profiles repository implementation type
/// * `UR` - The user repository implementation type
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::user_profiles_service::UserProfilesService;
/// use lh_core::repositories::user_profiles_repository::UserProfilesRepository;
/// use lh_core::repositories::user_repository::UserRepository;
///
/// async fn example(profile_repo: impl UserProfilesRepository, user_repo: impl UserRepository) {
///     let service = UserProfilesService::new(profile_repo, user_repo);
///
///     match service.get_profiles_for_user("john_doe").await {
///         Ok(profiles) => println!("Found {} profiles", profiles.len()),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub struct UserProfilesService<PR: UserProfilesRepository, UR: UserRepository> {
    profile_repository: PR,
    user_repository: UR,
}

impl<PR: UserProfilesRepository, UR: UserRepository> UserProfilesService<PR, UR> {
    /// Creates a new UserProfilesService instance.
    ///
    /// # Arguments
    ///
    /// * `profile_repository` - The user profiles repository implementation
    /// * `user_repository` - The user repository implementation
    ///
    /// # Returns
    ///
    /// A new `UserProfilesService` instance.
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
    /// # use lh_core::services::user_profiles_service::UserProfilesService;
    /// # use lh_core::repositories::user_profiles_repository::UserProfilesRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserProfilesService<impl UserProfilesRepository, impl UserRepository>) {
    /// match service.get_profiles_for_user("john_doe").await {
    ///     Ok(profiles) => {
    ///         for profile in profiles {
    ///             println!("Profile: {}", profile.target_language);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Failed to get profiles: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn get_profiles_for_user(&self, username: &str) -> Result<Vec<Profile>, CoreError> {
        self.profile_repository.find_by_username(username).await
    }

    /// Retrieves a profile by username and profile name (composite key).
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `profile_name` - The profile name
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
    /// # use lh_core::services::user_profiles_service::UserProfilesService;
    /// # use lh_core::repositories::user_profiles_repository::UserProfilesRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserProfilesService<impl UserProfilesRepository, impl UserRepository>) {
    /// match service.get_profile_by_id("john_doe", "My Spanish").await {
    ///     Ok(profile) => println!("Found profile: {:?}", profile),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn get_profile_by_id(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<Profile, CoreError> {
        self.profile_repository
            .find_by_username_and_profile_name(username, profile_name)
            .await?
            .ok_or_else(|| CoreError::not_found("Profile", profile_name))
    }

    /// Creates a new learning profile for a user.
    ///
    /// This method validates that the user exists before creating the profile.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the profile
    /// * `profile_name` - The name of the profile (user-defined)
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
    /// # use lh_core::services::user_profiles_service::UserProfilesService;
    /// # use lh_core::repositories::user_profiles_repository::UserProfilesRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserProfilesService<impl UserProfilesRepository, impl UserRepository>) {
    /// match service.create_profile("jane_doe", "My Spanish", "spanish").await {
    ///     Ok(profile) => println!("Created profile: {:?}", profile),
    ///     Err(e) => eprintln!("Failed to create profile: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn create_profile(
        &self,
        username: &str,
        profile_name: &str,
        target_language: &str,
    ) -> Result<Profile, CoreError> {
        // Business logic: ensure user exists
        if self
            .user_repository
            .find_by_username(username)
            .await?
            .is_none()
        {
            return Err(CoreError::not_found("User", username));
        }

        // Check if profile with this name already exists
        if let Some(_) = self
            .profile_repository
            .find_by_username_and_profile_name(username, profile_name)
            .await?
        {
            return Err(CoreError::validation_error(format!(
                "Profile '{}' already exists",
                profile_name
            )));
        }

        // Domain validation happens in Profile::new()
        let profile = Profile::new(profile_name, target_language)?;

        self.profile_repository.save(username, profile).await
    }

    /// Updates the last activity timestamp for a profile.
    ///
    /// This method should be called whenever a user interacts with a profile.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `profile_name` - The profile name
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
    /// # use lh_core::services::user_profiles_service::UserProfilesService;
    /// # use lh_core::repositories::user_profiles_repository::UserProfilesRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserProfilesService<impl UserProfilesRepository, impl UserRepository>) {
    /// match service.update_profile_activity("john_doe", "My Spanish").await {
    ///     Ok(profile) => println!("Updated activity: {:?}", profile),
    ///     Err(e) => eprintln!("Failed to update activity: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn update_profile_activity(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<Profile, CoreError> {
        let mut profile = self
            .profile_repository
            .find_by_username_and_profile_name(username, profile_name)
            .await?
            .ok_or_else(|| CoreError::not_found("Profile", profile_name))?;

        profile.update_last_activity();
        self.profile_repository.save(username, profile).await
    }

    /// Deletes a profile by username and profile name.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `profile_name` - The profile name
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
    /// # use lh_core::services::user_profiles_service::UserProfilesService;
    /// # use lh_core::repositories::user_profiles_repository::UserProfilesRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserProfilesService<impl UserProfilesRepository, impl UserRepository>) {
    /// match service.delete_profile("john_doe", "My Spanish").await {
    ///     Ok(()) => println!("Profile deleted successfully"),
    ///     Err(e) => eprintln!("Failed to delete profile: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn delete_profile(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<(), CoreError> {
        let deleted = self
            .profile_repository
            .delete(username, profile_name)
            .await?;
        if !deleted {
            return Err(CoreError::not_found("Profile", profile_name));
        }
        Ok(())
    }
}
