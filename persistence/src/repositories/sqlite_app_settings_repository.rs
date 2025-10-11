//! SQLite implementation for app settings persistence.
//!
//! This module provides a SQLite-based repository for managing global application
//! settings. The table is designed as a singleton (only one row).

use crate::errors::PersistenceError;
use crate::mappers::app_settings_mapper;
use crate::models::AppSettingsEntity;
use lh_core::models::app_settings::AppSettings;
use lh_core::repositories::adapters::PersistenceAppSettingsRepository;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

/// SQLite-based implementation of AppSettingsRepository.
///
/// This struct manages global application settings using SQLite.
/// The settings table contains only one row (singleton pattern).
///
/// # Database Schema
///
/// The app_settings table contains:
/// - `id` (INTEGER PRIMARY KEY): Always 1 for singleton
/// - `ui_theme` (TEXT): The UI theme preference
/// - `default_ui_language` (TEXT): Default language code for UI
///
/// # Thread Safety
///
/// This implementation uses an `Arc<Mutex<Connection>>` to allow safe
/// concurrent access to the database connection.
pub struct SqliteAppSettingsRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteAppSettingsRepository {
    /// Creates a new SqliteAppSettingsRepository instance.
    ///
    /// This will initialize the app_settings table with default values if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `connection` - Shared database connection
    ///
    /// # Returns
    ///
    /// * `Ok(SqliteAppSettingsRepository)` - A new repository instance
    /// * `Err(PersistenceError)` - If initialization fails
    pub fn new(connection: Arc<Mutex<Connection>>) -> Result<Self, PersistenceError> {
        let repo = Self { connection };
        repo.initialize_schema()?;
        Ok(repo)
    }

    /// Initializes the database schema for app settings.
    fn initialize_schema(&self) -> Result<(), PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        // Create the table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                ui_theme TEXT NOT NULL,
                default_ui_language TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| PersistenceError::database_error(format!("Failed to create schema: {}", e)))?;

        // Insert default settings if no row exists
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_settings", [], |row| row.get(0))
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to check settings: {}", e))
            })?;

        if count == 0 {
            let default_settings = AppSettingsEntity::default();
            conn.execute(
                "INSERT INTO app_settings (id, ui_theme, default_ui_language) VALUES (?1, ?2, ?3)",
                params![
                    default_settings.id,
                    default_settings.ui_theme,
                    default_settings.default_ui_language
                ],
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to insert defaults: {}", e))
            })?;
        }

        Ok(())
    }

    /// Retrieves the global app settings entity.
    ///
    /// # Returns
    ///
    /// * `Ok(AppSettingsEntity)` - The settings entity
    /// * `Err(PersistenceError)` - If the query fails
    pub fn get_entity(&self) -> Result<AppSettingsEntity, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let entity = conn
            .query_row(
                "SELECT id, ui_theme, default_ui_language FROM app_settings WHERE id = 1",
                [],
                |row| {
                    Ok(AppSettingsEntity {
                        id: row.get(0)?,
                        ui_theme: row.get(1)?,
                        default_ui_language: row.get(2)?,
                    })
                },
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to query settings: {}", e))
            })?;

        Ok(entity)
    }

    /// Retrieves the global app settings as a domain model.
    ///
    /// # Returns
    ///
    /// * `Ok(AppSettings)` - The settings
    /// * `Err(PersistenceError)` - If the query fails
    pub fn get(&self) -> Result<AppSettings, PersistenceError> {
        self.get_entity().map(|entity| app_settings_mapper::entity_to_model(&entity))
    }

    /// Updates the global app settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - The settings to save
    ///
    /// # Returns
    ///
    /// * `Ok(AppSettings)` - The saved settings
    /// * `Err(PersistenceError)` - If the update fails
    pub fn update(&self, settings: AppSettings) -> Result<AppSettings, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let entity = app_settings_mapper::model_to_entity(&settings);

        conn.execute(
            "UPDATE app_settings SET ui_theme = ?1, default_ui_language = ?2 WHERE id = 1",
            params![entity.ui_theme, entity.default_ui_language],
        )
        .map_err(|e| {
            PersistenceError::database_error(format!("Failed to update settings: {}", e))
        })?;

        Ok(settings)
    }
}

#[async_trait::async_trait]
impl PersistenceAppSettingsRepository for SqliteAppSettingsRepository {
    type Error = PersistenceError;

    async fn get(&self) -> Result<AppSettings, Self::Error> {
        let conn = self.connection.clone();
        tokio::task::spawn_blocking(move || {
            let connection = conn.lock().map_err(|e| {
                PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
            })?;

            let entity = connection
                .query_row(
                    "SELECT id, ui_theme, default_ui_language FROM app_settings WHERE id = 1",
                    [],
                    |row| {
                        Ok(AppSettingsEntity {
                            id: row.get(0)?,
                            ui_theme: row.get(1)?,
                            default_ui_language: row.get(2)?,
                        })
                    },
                )
                .map_err(|e| {
                    PersistenceError::database_error(format!("Failed to query settings: {}", e))
                })?;

            Ok(app_settings_mapper::entity_to_model(&entity))
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn update(&self, settings: AppSettings) -> Result<AppSettings, Self::Error> {
        let conn = self.connection.clone();
        tokio::task::spawn_blocking(move || {
            let connection = conn.lock().map_err(|e| {
                PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
            })?;

            let entity = app_settings_mapper::model_to_entity(&settings);

            connection
                .execute(
                    "UPDATE app_settings SET ui_theme = ?1, default_ui_language = ?2 WHERE id = 1",
                    params![entity.ui_theme, entity.default_ui_language],
                )
                .map_err(|e| {
                    PersistenceError::database_error(format!("Failed to update settings: {}", e))
                })?;

            Ok(settings)
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (SqliteAppSettingsRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_settings.db");
        let connection = Connection::open(db_path).unwrap();
        let repo = SqliteAppSettingsRepository::new(Arc::new(Mutex::new(connection))).unwrap();
        (repo, temp_dir)
    }

    #[test]
    fn test_initialization_creates_default_settings() {
        let (repo, _temp_dir) = create_test_repo();

        let settings = repo.get().unwrap();
        assert_eq!(settings.ui_theme, "Dark");
        assert_eq!(settings.default_ui_language, "en-US");
    }

    #[test]
    fn test_update_settings() {
        let (repo, _temp_dir) = create_test_repo();

        let new_settings = AppSettings::new("Dark".to_string(), "es".to_string()).unwrap();
        let updated = repo.update(new_settings.clone()).unwrap();

        assert_eq!(updated.ui_theme, "Dark");
        assert_eq!(updated.default_ui_language, "es");

        let retrieved = repo.get().unwrap();
        assert_eq!(retrieved.ui_theme, "Dark");
        assert_eq!(retrieved.default_ui_language, "es");
    }

    #[test]
    fn test_singleton_behavior() {
        let (repo, _temp_dir) = create_test_repo();

        let settings1 = repo.get().unwrap();
        let new_settings = AppSettings::new("Light".to_string(), "fr".to_string()).unwrap();
        repo.update(new_settings).unwrap();
        let settings2 = repo.get().unwrap();

        // Should be only one row, updated
        assert_ne!(settings1.ui_theme, settings2.ui_theme);
        assert_eq!(settings2.ui_theme, "Light");
    }

    #[test]
    fn test_get_entity() {
        let (repo, _temp_dir) = create_test_repo();

        let entity = repo.get_entity().unwrap();
        assert_eq!(entity.id, 1);
        assert!(!entity.ui_theme.is_empty());
        assert!(!entity.default_ui_language.is_empty());
    }
}
