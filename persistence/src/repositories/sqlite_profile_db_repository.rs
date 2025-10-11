//! SQLite implementation for profile database management.
//!
//! This module provides a SQLite-based repository for creating and managing
//! profile-specific database files. Each profile gets its own database file
//! for storing learning content (vocabulary cards, progress, etc.).

use crate::errors::PersistenceError;
use lh_core::repositories::adapters::PersistenceProfileDbRepository;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

/// SQLite-based implementation of ProfileRepository.
///
/// This struct manages profile-specific database files. Each profile has its own
/// database file at `data/{username}/{target_language}_profile.db`.
pub struct SqliteProfileDbRepository;

impl SqliteProfileDbRepository {
    /// Creates a new SqliteProfileDbRepository instance.
    pub fn new() -> Self {
        Self
    }

    /// Creates a new profile database file with initial schema.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path where the database should be created
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the database was successfully created
    /// * `Err(PersistenceError)` - If an error occurs during creation
    fn create_database_internal(&self, db_path: PathBuf) -> Result<(), PersistenceError> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                PersistenceError::database_error(format!(
                    "Failed to create directory {:?}: {}",
                    parent, e
                ))
            })?;
        }

        // Create the database file and initialize schema
        let conn = Connection::open(&db_path).map_err(|e| {
            PersistenceError::database_error(format!(
                "Failed to create database at {:?}: {}",
                db_path, e
            ))
        })?;

        // Initialize schema (empty for now, ready for future tables)
        // In the future, this will include tables for:
        // - vocabulary cards
        // - learning progress
        // - user-generated content
        // For now, we just ensure the database file is created with proper initialization
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| {
            PersistenceError::database_error(format!("Failed to initialize schema: {}", e))
        })?;

        // Insert initial schema version
        conn.execute(
            "INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (1, ?1)",
            [chrono::Utc::now().timestamp()],
        )
        .map_err(|e| {
            PersistenceError::database_error(format!("Failed to set schema version: {}", e))
        })?;

        Ok(())
    }

    /// Deletes a profile database file.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the database file
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the database was deleted
    /// * `Ok(false)` - If the database didn't exist
    /// * `Err(PersistenceError)` - If an error occurs during deletion
    fn delete_database_internal(&self, db_path: PathBuf) -> Result<bool, PersistenceError> {
        if !db_path.exists() {
            return Ok(false);
        }

        fs::remove_file(&db_path).map_err(|e| {
            PersistenceError::database_error(format!(
                "Failed to delete database at {:?}: {}",
                db_path, e
            ))
        })?;

        Ok(true)
    }
}

impl Default for SqliteProfileDbRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PersistenceProfileDbRepository for SqliteProfileDbRepository {
    type Error = PersistenceError;

    fn create_database(&self, db_path: PathBuf) -> Result<(), Self::Error> {
        self.create_database_internal(db_path)
    }

    fn delete_database(&self, db_path: PathBuf) -> Result<bool, Self::Error> {
        self.delete_database_internal(db_path)
    }
}
