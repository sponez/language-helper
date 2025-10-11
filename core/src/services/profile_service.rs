//! Profile service for managing individual profile databases.
//!
//! This module provides the business logic for managing profile-specific databases
//! where learning content (vocabulary cards, progress, etc.) is stored.
//! Each profile gets its own database file at `data/{username}/{target_language}_profile.db`.

use crate::errors::CoreError;
use crate::repositories::profile_repository::ProfileRepository;
use std::path::PathBuf;

/// Service for profile database management.
///
/// This struct implements the business logic for creating and managing profile-specific
/// databases. Each profile has its own database file for storing learning content.
///
/// # Type Parameters
///
/// * `R` - The profile repository implementation type
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::profile_service::ProfileService;
/// use lh_core::repositories::profile_repository::ProfileRepository;
///
/// fn example(repo: impl ProfileRepository) {
///     let service = ProfileService::new(repo, "data");
///
///     match service.create_profile_database("john_doe", "spanish") {
///         Ok(path) => println!("Created database at: {:?}", path),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub struct ProfileService<R: ProfileRepository> {
    repository: R,
    data_dir: String,
}

impl<R: ProfileRepository> ProfileService<R> {
    /// Creates a new ProfileService instance.
    ///
    /// # Arguments
    ///
    /// * `repository` - The profile repository implementation
    /// * `data_dir` - The base data directory path (e.g., "data")
    ///
    /// # Returns
    ///
    /// A new `ProfileService` instance.
    pub fn new(repository: R, data_dir: impl Into<String>) -> Self {
        Self {
            repository,
            data_dir: data_dir.into(),
        }
    }

    /// Creates a new profile database.
    ///
    /// This creates a database file at `data/{username}/{target_language}_profile.db`
    /// and initializes it with the necessary schema (currently empty, for future use).
    ///
    /// # Arguments
    ///
    /// * `username` - The username this profile belongs to
    /// * `target_language` - The target language being learned
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The path to the created database file
    /// * `Err(CoreError)` - If an error occurs during creation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::ValidationError` if parameters are invalid,
    /// or `CoreError::RepositoryError` if there's a problem creating the database.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::profile_service::ProfileService;
    /// # use lh_core::repositories::profile_repository::ProfileRepository;
    /// # fn example(service: &ProfileService<impl ProfileRepository>) {
    /// match service.create_profile_database("jane_doe", "french") {
    ///     Ok(path) => println!("Database created at: {:?}", path),
    ///     Err(e) => eprintln!("Failed to create database: {}", e),
    /// }
    /// # }
    /// ```
    pub fn create_profile_database(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<PathBuf, CoreError> {
        // Validate inputs
        if username.is_empty() {
            return Err(CoreError::validation_error("Username cannot be empty"));
        }
        if target_language.is_empty() {
            return Err(CoreError::validation_error("Target language cannot be empty"));
        }

        // Build the path: data/{username}/{target_language}_profile.db
        let user_dir = PathBuf::from(&self.data_dir).join(username);
        let db_filename = format!("{}_profile.db", target_language);
        let db_path = user_dir.join(db_filename);

        // Create the database using the repository
        self.repository.create_database(db_path.clone())?;

        Ok(db_path)
    }

    /// Checks if a profile database exists.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - true if the database exists, false otherwise
    /// * `Err(CoreError)` - If an error occurs during the check
    pub fn profile_database_exists(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<bool, CoreError> {
        let user_dir = PathBuf::from(&self.data_dir).join(username);
        let db_filename = format!("{}_profile.db", target_language);
        let db_path = user_dir.join(db_filename);

        Ok(db_path.exists())
    }

    /// Deletes a profile database.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - true if the database was deleted, false if it didn't exist
    /// * `Err(CoreError)` - If an error occurs during deletion
    pub fn delete_profile_database(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<bool, CoreError> {
        let user_dir = PathBuf::from(&self.data_dir).join(username);
        let db_filename = format!("{}_profile.db", target_language);
        let db_path = user_dir.join(db_filename);

        self.repository.delete_database(db_path)
    }
}
