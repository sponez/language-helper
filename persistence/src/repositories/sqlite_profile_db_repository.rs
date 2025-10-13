//! SQLite implementation for profile database management.
//!
//! This module provides a SQLite-based repository for creating and managing
//! profile-specific database files. Each profile gets its own database file
//! for storing learning content (vocabulary cards, progress, etc.).

use crate::errors::PersistenceError;
use crate::models::{AssistantSettingsEntity, CardSettingsEntity};
use lh_core::models::{AssistantSettings, CardSettings};
use lh_core::repositories::adapters::PersistenceProfileDbRepository;
use rusqlite::{Connection, params};
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
}

impl Default for SqliteProfileDbRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl PersistenceProfileDbRepository for SqliteProfileDbRepository {
    type Error = PersistenceError;

    async fn create_database(&self, db_path: PathBuf) -> Result<(), Self::Error> {
        tokio::task::spawn_blocking(move || {
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

            // Initialize schema
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

            // Create card_settings table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS card_settings (
                    id INTEGER PRIMARY KEY,
                    cards_per_set INTEGER NOT NULL DEFAULT 10,
                    test_answer_method TEXT NOT NULL DEFAULT 'manual',
                    streak_length INTEGER NOT NULL DEFAULT 5
                )",
                [],
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to create card_settings table: {}", e))
            })?;

            // Create assistant_settings table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS assistant_settings (
                    id INTEGER PRIMARY KEY,
                    ai_model TEXT,
                    api_endpoint TEXT,
                    api_key TEXT,
                    api_model_name TEXT
                )",
                [],
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to create assistant_settings table: {}", e))
            })?;

            // Insert initial schema version
            conn.execute(
                "INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (1, ?1)",
                [chrono::Utc::now().timestamp()],
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to set schema version: {}", e))
            })?;

            // Insert default card settings with id=1
            conn.execute(
                "INSERT OR IGNORE INTO card_settings (id, cards_per_set, test_answer_method, streak_length) VALUES (1, 10, 'manual', 5)",
                [],
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to insert default card settings: {}", e))
            })?;

            // Insert default assistant settings with id=1
            conn.execute(
                "INSERT OR IGNORE INTO assistant_settings (id) VALUES (1)",
                [],
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to insert default assistant settings: {}", e))
            })?;

            Ok(())
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn delete_database(&self, db_path: PathBuf) -> Result<bool, Self::Error> {
        tokio::task::spawn_blocking(move || {
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
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn get_card_settings(&self, db_path: PathBuf) -> Result<CardSettings, Self::Error> {
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| {
                PersistenceError::database_error(format!(
                    "Failed to open database at {:?}: {}",
                    db_path, e
                ))
            })?;

            let mut stmt = conn.prepare(
                "SELECT id, cards_per_set, test_answer_method, streak_length
                 FROM card_settings WHERE id = 1"
            ).map_err(|e| {
                PersistenceError::database_error(format!("Failed to prepare query: {}", e))
            })?;

            let entity = stmt.query_row([], |row| {
                Ok(CardSettingsEntity {
                    id: row.get(0)?,
                    cards_per_set: row.get(1)?,
                    test_answer_method: row.get(2)?,
                    streak_length: row.get(3)?,
                })
            }).map_err(|e| {
                PersistenceError::database_error(format!("Failed to fetch card settings: {}", e))
            })?;

            entity.to_domain()
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn update_card_settings(&self, db_path: PathBuf, settings: CardSettings) -> Result<(), Self::Error> {
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| {
                PersistenceError::database_error(format!(
                    "Failed to open database at {:?}: {}",
                    db_path, e
                ))
            })?;

            conn.execute(
                "UPDATE card_settings SET
                    cards_per_set = ?1,
                    test_answer_method = ?2,
                    streak_length = ?3
                 WHERE id = 1",
                params![
                    settings.cards_per_set as i64,
                    settings.test_answer_method,
                    settings.streak_length as i64,
                ],
            ).map_err(|e| {
                PersistenceError::database_error(format!("Failed to update card settings: {}", e))
            })?;

            Ok(())
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn get_assistant_settings(&self, db_path: PathBuf) -> Result<AssistantSettings, Self::Error> {
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| {
                PersistenceError::database_error(format!(
                    "Failed to open database at {:?}: {}",
                    db_path, e
                ))
            })?;

            let mut stmt = conn.prepare(
                "SELECT id, ai_model, api_endpoint, api_key, api_model_name
                 FROM assistant_settings WHERE id = 1"
            ).map_err(|e| {
                PersistenceError::database_error(format!("Failed to prepare query: {}", e))
            })?;

            let entity = stmt.query_row([], |row| {
                Ok(AssistantSettingsEntity {
                    id: row.get(0)?,
                    ai_model: row.get(1)?,
                    api_endpoint: row.get(2)?,
                    api_key: row.get(3)?,
                    api_model_name: row.get(4)?,
                })
            }).map_err(|e| {
                PersistenceError::database_error(format!("Failed to fetch assistant settings: {}", e))
            })?;

            entity.to_domain()
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn update_assistant_settings(&self, db_path: PathBuf, settings: AssistantSettings) -> Result<(), Self::Error> {
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| {
                PersistenceError::database_error(format!(
                    "Failed to open database at {:?}: {}",
                    db_path, e
                ))
            })?;

            conn.execute(
                "UPDATE assistant_settings SET
                    ai_model = ?1,
                    api_endpoint = ?2,
                    api_key = ?3,
                    api_model_name = ?4
                 WHERE id = 1",
                params![
                    settings.ai_model,
                    settings.api_endpoint,
                    settings.api_key,
                    settings.api_model_name,
                ],
            ).map_err(|e| {
                PersistenceError::database_error(format!("Failed to update assistant settings: {}", e))
            })?;

            Ok(())
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn clear_assistant_settings(&self, db_path: PathBuf) -> Result<(), Self::Error> {
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| {
                PersistenceError::database_error(format!(
                    "Failed to open database at {:?}: {}",
                    db_path, e
                ))
            })?;

            conn.execute(
                "UPDATE assistant_settings SET
                    ai_model = NULL,
                    api_endpoint = NULL,
                    api_key = NULL,
                    api_model_name = NULL
                 WHERE id = 1",
                [],
            ).map_err(|e| {
                PersistenceError::database_error(format!("Failed to clear assistant settings: {}", e))
            })?;

            Ok(())
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }
}
