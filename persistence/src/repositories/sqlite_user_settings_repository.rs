//! SQLite implementation for user settings persistence.
//!
//! This module provides a SQLite-based repository for managing user-specific settings
//! with a one-to-one relationship to users.

use crate::errors::PersistenceError;
use crate::models::UserSettingsEntity;
use lh_core::domain::user_settings::UserSettings;
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
    pub fn find_by_username(&self, username: &str) -> Result<Option<UserSettings>, PersistenceError> {
        self.find_entity_by_username(username)
            .map(|opt| opt.map(|entity| entity.to_domain()))
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
    pub fn save(&self, settings: UserSettings) -> Result<UserSettings, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let entity = UserSettingsEntity::from_domain(settings.clone());

        conn.execute(
            "INSERT INTO user_settings (username, ui_theme, ui_language) VALUES (?1, ?2, ?3)
             ON CONFLICT(username) DO UPDATE SET ui_theme = ?2, ui_language = ?3",
            params![entity.username, entity.ui_theme, entity.ui_language],
        )
        .map_err(|e| PersistenceError::database_error(format!("Failed to save user settings: {}", e)))?;

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
            .execute("DELETE FROM user_settings WHERE username = ?1", params![username])
            .map_err(|e| PersistenceError::database_error(format!("Failed to delete user settings: {}", e)))?;

        Ok(rows_affected > 0)
    }
}

impl PersistenceUserSettingsRepository for SqliteUserSettingsRepository {
    type Error = PersistenceError;

    fn find_by_username(&self, username: &str) -> Result<Option<UserSettings>, Self::Error> {
        self.find_by_username(username)
    }

    fn save(&self, settings: UserSettings) -> Result<UserSettings, Self::Error> {
        self.save(settings)
    }

    fn delete(&self, username: &str) -> Result<bool, Self::Error> {
        self.delete(username)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (SqliteUserSettingsRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_user_settings.db");
        let connection = Connection::open(db_path).unwrap();

        // Create users table first (for foreign key constraint)
        connection
            .execute(
                "CREATE TABLE users (
                    username TEXT PRIMARY KEY NOT NULL,
                    created_at INTEGER NOT NULL,
                    last_used_at INTEGER NOT NULL
                )",
                [],
            )
            .unwrap();

        let repo = SqliteUserSettingsRepository::new(Arc::new(Mutex::new(connection))).unwrap();
        (repo, temp_dir)
    }

    fn insert_test_user(repo: &SqliteUserSettingsRepository, username: &str) {
        let conn = repo.connection.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO users (username, created_at, last_used_at) VALUES (?1, ?2, ?3)",
            params![username, now, now],
        )
        .unwrap();
    }

    #[test]
    fn test_create_and_find_settings() {
        let (repo, _temp_dir) = create_test_repo();
        insert_test_user(&repo, "test_user");

        let settings = UserSettings::new(
            "test_user".to_string(),
            "Dark".to_string(),
            "en".to_string(),
        )
        .unwrap();
        let saved = repo.save(settings.clone()).unwrap();

        assert_eq!(saved.username, settings.username);

        let found = repo.find_by_username("test_user").unwrap();
        assert!(found.is_some());
        let found_settings = found.unwrap();
        assert_eq!(found_settings.username, "test_user");
        assert_eq!(found_settings.ui_theme, "Dark");
        assert_eq!(found_settings.ui_language, "en");
    }

    #[test]
    fn test_find_nonexistent_settings() {
        let (repo, _temp_dir) = create_test_repo();

        let result = repo.find_by_username("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_existing_settings() {
        let (repo, _temp_dir) = create_test_repo();
        insert_test_user(&repo, "update_test");

        let settings1 = UserSettings::new(
            "update_test".to_string(),
            "Dark".to_string(),
            "en".to_string(),
        )
        .unwrap();
        repo.save(settings1).unwrap();

        let settings2 = UserSettings::new(
            "update_test".to_string(),
            "Light".to_string(),
            "es".to_string(),
        )
        .unwrap();
        repo.save(settings2).unwrap();

        let found = repo.find_by_username("update_test").unwrap().unwrap();
        assert_eq!(found.ui_theme, "Light");
        assert_eq!(found.ui_language, "es");
    }

    #[test]
    fn test_delete_settings() {
        let (repo, _temp_dir) = create_test_repo();
        insert_test_user(&repo, "delete_test");

        let settings = UserSettings::new(
            "delete_test".to_string(),
            "Dark".to_string(),
            "en".to_string(),
        )
        .unwrap();
        repo.save(settings).unwrap();

        let deleted = repo.delete("delete_test").unwrap();
        assert!(deleted);

        let found = repo.find_by_username("delete_test").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_delete_nonexistent_settings() {
        let (repo, _temp_dir) = create_test_repo();

        let deleted = repo.delete("nonexistent").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_find_entity_by_username() {
        let (repo, _temp_dir) = create_test_repo();
        insert_test_user(&repo, "entity_test");

        let settings = UserSettings::new(
            "entity_test".to_string(),
            "System".to_string(),
            "fr".to_string(),
        )
        .unwrap();
        repo.save(settings).unwrap();

        let entity = repo.find_entity_by_username("entity_test").unwrap();
        assert!(entity.is_some());
        let entity = entity.unwrap();
        assert_eq!(entity.username, "entity_test");
        assert_eq!(entity.ui_theme, "System");
        assert_eq!(entity.ui_language, "fr");
    }
}
