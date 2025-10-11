//! Adapter for ProfileRepository that maps persistence errors to core errors.
//!
//! This module provides an adapter that wraps a persistence-layer profile database
//! repository and converts its errors to core domain errors.

use async_trait::async_trait;
use crate::errors::CoreError;
use crate::repositories::profile_repository::ProfileRepository;
use std::fmt::Display;
use std::path::PathBuf;

/// Trait for persistence-layer profile database repositories.
///
/// This trait must be implemented by the persistence layer to handle
/// profile database file operations with persistence-specific error types.
#[async_trait]
pub trait PersistenceProfileDbRepository: Send + Sync {
    /// The error type from the persistence layer
    type Error: Display;

    /// Creates a new profile database file.
    async fn create_database(&self, db_path: PathBuf) -> Result<(), Self::Error>;

    /// Deletes a profile database file.
    async fn delete_database(&self, db_path: PathBuf) -> Result<bool, Self::Error>;
}

/// Adapter that wraps a persistence repository and converts errors to CoreError.
///
/// This struct implements the core ProfileRepository trait by delegating to
/// a persistence-layer implementation and mapping errors.
pub struct ProfileDbRepositoryAdapter<R: PersistenceProfileDbRepository> {
    repository: R,
}

impl<R: PersistenceProfileDbRepository> ProfileDbRepositoryAdapter<R> {
    /// Creates a new ProfileDbRepositoryAdapter.
    ///
    /// # Arguments
    ///
    /// * `repository` - The persistence-layer repository to wrap
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: PersistenceProfileDbRepository> ProfileRepository for ProfileDbRepositoryAdapter<R> {
    async fn create_database(&self, db_path: PathBuf) -> Result<(), CoreError> {
        self.repository
            .create_database(db_path)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn delete_database(&self, db_path: PathBuf) -> Result<bool, CoreError> {
        self.repository
            .delete_database(db_path)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }
}
