//! SQLite implementation for user settings persistence.
//!
//! This module provides a SQLite-based repository for managing user-specific settings
//! with a one-to-one relationship to users.

use crate::errors::PersistenceError;
use crate::mappers::user_settings_mapper;
use crate::models::UserSettingsEntity;
use lh_core::models::user_settings::UserSettings;
use lh_core::repositories::adapters::PersistenceUserSettingsRepository;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

/// SQLite-based implementation of UserSettingsRepository.
///
/// This struct manages user-specific settings using SQLite with a one-to-one
/// relationship to the users table.
///
/// # Database Schema
///
/// The user_settings table contains:
/// - `username` (TEXT PRIMARY KEY): Foreign key to users table
/// - `ui_theme` (TEXT): User's UI theme preference
/// - `ui_language` (TEXT): User's UI language code
///
/// # Thread Safety
///
/// This implementation uses an `Arc<Mutex<Connection>>` to allow safe
/// concurrent access to the database connection.
pub struct SqliteUserSettingsRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteUserSettingsRepository {
    /// Creates a new SqliteUserSettingsRepository instance.
    ///
    /// This will initialize the user_settings table if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `connection` - Shared database connection
    ///
    /// # Returns
    ///
    /// * `Ok(SqliteUserSettingsRepository)` - A new repository instance
    /// * `Err(PersistenceError)` - If initialization fails
    pub fn new(connection: Arc<Mutex<Connection>>) -> Result<Self, PersistenceError> {
        let repo = Self { connection };
        repo.initialize_schema()?;
        Ok(repo)
    }

    /// Initializes the database schema for user settings.
    fn initialize_schema(&self) -> Result<(), PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_settings (
                username TEXT PRIMARY KEY NOT NULL,
                ui_theme TEXT NOT NULL,
                ui_language TEXT NOT NULL,
                FOREIGN KEY (username) REFERENCES users(username) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| PersistenceError::database_error(format!("Failed to create schema: {}", e)))?;

        Ok(())
    }

    /// Retrieves user settings by username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to query
    ///
    /// # Returns
    ///
    /// * `Ok(Some(UserSettingsEntity))` - If settings exist
    /// * `Ok(None)` - If no settings exist for the user
    /// * `Err(PersistenceError)` - If the query fails
    pub fn find_entity_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserSettingsEntity>, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let result = conn.query_row(
            "SELECT username, ui_theme, ui_language FROM user_settings WHERE username = ?1",
            params![username],
            |row| {
                Ok(UserSettingsEntity {
                    username: row.get(0)?,
                    ui_theme: row.get(1)?,
                    ui_language: row.get(2)?,
                })
            },
        );

        match result {
            Ok(entity) => Ok(Some(entity)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(PersistenceError::database_error(format!(
                "Failed to query user settings: {}",
                e
            ))),
        }
    }

    /// Retrieves user settings by username as a domain model.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to query
    ///
    /// # Returns
    ///
    /// * `Ok(Some(UserSettings))` - If settings exist
    /// * `Ok(None)` - If no settings exist
    /// * `Err(PersistenceError)` - If the query fails
    pub fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserSettings>, PersistenceError> {
        self.find_entity_by_username(username)
            .map(|opt| opt.map(|entity| user_settings_mapper::entity_to_model(&entity)))
    }

    /// Saves user settings to the database.
    ///
    /// If settings already exist for the user, they will be updated.
    ///
    /// # Arguments
    ///
    /// * `settings` - The settings to save
    ///
    /// # Returns
    ///
    /// * `Ok(UserSettings)` - The saved settings
    /// * `Err(PersistenceError)` - If the save operation fails
    pub fn save(&self, username: &str, settings: UserSettings) -> Result<UserSettings, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let entity = user_settings_mapper::model_to_entity(username, &settings);

        conn.execute(
            "INSERT INTO user_settings (username, ui_theme, ui_language) VALUES (?1, ?2, ?3)
             ON CONFLICT(username) DO UPDATE SET ui_theme = ?2, ui_language = ?3",
            params![entity.username, entity.ui_theme, entity.ui_language],
        )
        .map_err(|e| {
            PersistenceError::database_error(format!("Failed to save user settings: {}", e))
        })?;

        Ok(settings)
    }

    /// Deletes user settings by username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username whose settings to delete
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If settings were deleted
    /// * `Ok(false)` - If no settings existed
    /// * `Err(PersistenceError)` - If the delete operation fails
    pub fn delete(&self, username: &str) -> Result<bool, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let rows_affected = conn
            .execute(
                "DELETE FROM user_settings WHERE username = ?1",
                params![username],
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to delete user settings: {}", e))
            })?;

        Ok(rows_affected > 0)
    }
}

impl PersistenceUserSettingsRepository for SqliteUserSettingsRepository {
    type Error = PersistenceError;

    fn find_by_username(&self, username: &str) -> Result<Option<UserSettings>, Self::Error> {
        self.find_by_username(username)
    }

    fn save(&self, username: &str, settings: UserSettings) -> Result<UserSettings, Self::Error> {
        self.save(username, settings)
    }

    fn delete(&self, username: &str) -> Result<bool, Self::Error> {
        self.delete(username)
    }
}
