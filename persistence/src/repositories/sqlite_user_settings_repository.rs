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
    pub fn save(&self, username: String, settings: UserSettings) -> Result<UserSettings, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let entity = user_settings_mapper::model_to_entity(username.clone(), &settings);

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

    fn save(&self, username: String, settings: UserSettings) -> Result<UserSettings, Self::Error> {
        self.save(username, settings)
    }

    fn delete(&self, username: &str) -> Result<bool, Self::Error> {
        self.delete(username)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn create_test_repository() -> SqliteUserSettingsRepository {
        let connection = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));

        // Create users table first (required for foreign key constraint)
        {
            let conn = connection.lock().unwrap();
            conn.execute(
                "CREATE TABLE users (
                    username TEXT PRIMARY KEY NOT NULL
                )",
                [],
            )
            .unwrap();
        }

        SqliteUserSettingsRepository::new(connection).unwrap()
    }

    fn insert_test_user(repo: &SqliteUserSettingsRepository, username: &str) {
        let conn = repo.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO users (username) VALUES (?1)",
            params![username],
        )
        .unwrap();
    }

    #[test]
    fn test_initialization_creates_schema() {
        let connection = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));

        // Create users table first (required for foreign key)
        {
            let conn = connection.lock().unwrap();
            conn.execute(
                "CREATE TABLE users (
                    username TEXT PRIMARY KEY NOT NULL
                )",
                [],
            )
            .unwrap();
        }

        let _repo = SqliteUserSettingsRepository::new(connection.clone()).unwrap();

        // Verify table exists by querying it
        let conn = connection.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM user_settings").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_save_and_find_user_settings() {
        let repo = create_test_repository();
        insert_test_user(&repo, "alice");

        let settings = UserSettings::new_unchecked("Dark".to_string(), "en".to_string());

        // Save settings
        let saved = repo.save("alice".to_string(), settings.clone()).unwrap();
        assert_eq!(saved.ui_theme, "Dark");
        assert_eq!(saved.ui_language, "en");

        // Find settings
        let found = repo.find_by_username("alice").unwrap();
        assert!(found.is_some());
        let found_settings = found.unwrap();
        assert_eq!(found_settings.ui_theme, "Dark");
        assert_eq!(found_settings.ui_language, "en");
    }

    #[test]
    fn test_find_nonexistent_settings() {
        let repo = create_test_repository();

        let result = repo.find_by_username("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_existing_settings() {
        let repo = create_test_repository();
        insert_test_user(&repo, "bob");

        // Create initial settings
        let initial = UserSettings::new_unchecked("Dark".to_string(), "en".to_string());
        repo.save("bob".to_string(), initial).unwrap();

        // Update settings
        let updated = UserSettings::new_unchecked("Light".to_string(), "es".to_string());
        repo.save("bob".to_string(), updated).unwrap();

        // Verify update
        let found = repo.find_by_username("bob").unwrap().unwrap();
        assert_eq!(found.ui_theme, "Light");
        assert_eq!(found.ui_language, "es");
    }

    #[test]
    fn test_delete_existing_settings() {
        let repo = create_test_repository();
        insert_test_user(&repo, "charlie");

        // Create settings
        let settings = UserSettings::new_unchecked("Dark".to_string(), "en".to_string());
        repo.save("charlie".to_string(), settings).unwrap();

        // Delete settings
        let deleted = repo.delete("charlie").unwrap();
        assert!(deleted);

        // Verify deletion
        let found = repo.find_by_username("charlie").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_delete_nonexistent_settings() {
        let repo = create_test_repository();

        let deleted = repo.delete("nonexistent").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_find_entity_by_username() {
        let repo = create_test_repository();
        insert_test_user(&repo, "dave");

        let settings = UserSettings::new_unchecked("Nord".to_string(), "fr".to_string());
        repo.save("dave".to_string(), settings).unwrap();

        let entity = repo.find_entity_by_username("dave").unwrap().unwrap();
        assert_eq!(entity.username, "dave");
        assert_eq!(entity.ui_theme, "Nord");
        assert_eq!(entity.ui_language, "fr");
    }

    #[test]
    fn test_multiple_users_settings() {
        let repo = create_test_repository();
        insert_test_user(&repo, "eve");
        insert_test_user(&repo, "frank");

        // Save settings for multiple users
        let eve_settings = UserSettings::new_unchecked("Dark".to_string(), "en".to_string());
        let frank_settings = UserSettings::new_unchecked("Light".to_string(), "es".to_string());

        repo.save("eve".to_string(), eve_settings).unwrap();
        repo.save("frank".to_string(), frank_settings).unwrap();

        // Verify both exist independently
        let eve_found = repo.find_by_username("eve").unwrap().unwrap();
        assert_eq!(eve_found.ui_theme, "Dark");

        let frank_found = repo.find_by_username("frank").unwrap().unwrap();
        assert_eq!(frank_found.ui_theme, "Light");
    }

    #[test]
    fn test_cascade_delete_with_user() {
        let repo = create_test_repository();
        insert_test_user(&repo, "grace");

        // Save settings
        let settings = UserSettings::new_unchecked("Dark".to_string(), "en".to_string());
        repo.save("grace".to_string(), settings).unwrap();

        // Delete user (should cascade to settings)
        {
            let conn = repo.connection.lock().unwrap();
            conn.execute("DELETE FROM users WHERE username = ?1", params!["grace"])
                .unwrap();
        }

        // Verify settings were also deleted
        let found = repo.find_by_username("grace").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_save_validates_foreign_key() {
        let repo = create_test_repository();

        // Try to save settings for non-existent user
        let settings = UserSettings::new_unchecked("Dark".to_string(), "en".to_string());
        let result = repo.save("nonexistent".to_string(), settings);

        // Should fail due to foreign key constraint
        assert!(result.is_err());
    }
}
