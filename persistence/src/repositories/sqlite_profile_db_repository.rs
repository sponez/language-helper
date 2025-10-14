//! SQLite implementation for profile database management.
//!
//! This module provides a SQLite-based repository for creating and managing
//! profile-specific database files. Each profile gets its own database file
//! for storing learning content (vocabulary cards, progress, etc.).

use crate::errors::PersistenceError;
use crate::models::{AssistantSettingsEntity, CardEntity, CardSettingsEntity, CardWithRelations, MeaningEntity};
use lh_core::models::{AssistantSettings, Card, CardSettings};
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

            // Create cards table with word_name as primary key
            conn.execute(
                "CREATE TABLE IF NOT EXISTS cards (
                    word_name TEXT PRIMARY KEY,
                    card_type TEXT NOT NULL CHECK(card_type IN ('straight', 'reverse')),
                    word_readings TEXT NOT NULL DEFAULT '[]',
                    streak INTEGER NOT NULL DEFAULT 0,
                    created_at INTEGER NOT NULL
                )",
                [],
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to create cards table: {}", e))
            })?;

            // Create meanings table with word_name foreign key
            conn.execute(
                "CREATE TABLE IF NOT EXISTS meanings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    word_name TEXT NOT NULL,
                    definition TEXT NOT NULL,
                    translated_definition TEXT NOT NULL,
                    word_translations TEXT NOT NULL DEFAULT '[]',
                    FOREIGN KEY (word_name) REFERENCES cards(word_name) ON DELETE CASCADE
                )",
                [],
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to create meanings table: {}", e))
            })?;

            // Create index on cards for better query performance
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_cards_streak ON cards(streak)",
                [],
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to create index on cards: {}", e))
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

    async fn save_card(&self, db_path: PathBuf, card: Card) -> Result<(), Self::Error> {
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| {
                PersistenceError::database_error(format!(
                    "Failed to open database at {:?}: {}",
                    db_path, e
                ))
            })?;

            // Start transaction
            let tx = conn.unchecked_transaction().map_err(|e| {
                PersistenceError::database_error(format!("Failed to start transaction: {}", e))
            })?;

            // Convert card to entity
            let card_entity = CardEntity::from_domain(&card)?;

            // Upsert card (INSERT OR REPLACE)
            tx.execute(
                "INSERT OR REPLACE INTO cards (word_name, card_type, word_readings, streak, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    card_entity.word_name,
                    card_entity.card_type,
                    card_entity.word_readings,
                    card_entity.streak,
                    card_entity.created_at
                ],
            ).map_err(|e| {
                PersistenceError::database_error(format!("Failed to upsert card: {}", e))
            })?;

            // Delete old meanings (if any)
            tx.execute(
                "DELETE FROM meanings WHERE word_name = ?1",
                [&card_entity.word_name],
            ).map_err(|e| {
                PersistenceError::database_error(format!("Failed to delete old meanings: {}", e))
            })?;

            // Insert new meanings
            for meaning in &card.meanings {
                let meaning_entity = MeaningEntity::from_domain(&card_entity.word_name, meaning)?;
                tx.execute(
                    "INSERT INTO meanings (word_name, definition, translated_definition, word_translations)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        meaning_entity.word_name,
                        meaning_entity.definition,
                        meaning_entity.translated_definition,
                        meaning_entity.word_translations
                    ],
                ).map_err(|e| {
                    PersistenceError::database_error(format!("Failed to insert meaning: {}", e))
                })?;
            }

            // Commit transaction
            tx.commit().map_err(|e| {
                PersistenceError::database_error(format!("Failed to commit transaction: {}", e))
            })?;

            Ok(())
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn get_all_cards(&self, db_path: PathBuf) -> Result<Vec<Card>, Self::Error> {
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| {
                PersistenceError::database_error(format!(
                    "Failed to open database at {:?}: {}",
                    db_path, e
                ))
            })?;

            Self::fetch_cards(&conn, "SELECT word_name, card_type, word_readings, streak, created_at FROM cards", &[])
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn get_cards_by_learned_status(
        &self,
        db_path: PathBuf,
        streak_threshold: i32,
        learned: bool,
    ) -> Result<Vec<Card>, Self::Error> {
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| {
                PersistenceError::database_error(format!(
                    "Failed to open database at {:?}: {}",
                    db_path, e
                ))
            })?;

            let query = if learned {
                "SELECT word_name, card_type, word_readings, streak, created_at FROM cards WHERE streak >= ?1"
            } else {
                "SELECT word_name, card_type, word_readings, streak, created_at FROM cards WHERE streak < ?1"
            };

            Self::fetch_cards(&conn, query, &[&streak_threshold])
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn get_card_by_word_name(&self, db_path: PathBuf, word_name: String) -> Result<Card, Self::Error> {
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| {
                PersistenceError::database_error(format!(
                    "Failed to open database at {:?}: {}",
                    db_path, e
                ))
            })?;

            let cards = Self::fetch_cards(
                &conn,
                "SELECT word_name, card_type, word_readings, streak, created_at FROM cards WHERE word_name = ?1",
                &[&word_name],
            )?;

            cards.into_iter().next().ok_or_else(|| {
                PersistenceError::database_error(format!("Card with word_name '{}' not found", word_name))
            })
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn update_card_streak(&self, db_path: PathBuf, word_name: String, streak: i32) -> Result<(), Self::Error> {
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| {
                PersistenceError::database_error(format!(
                    "Failed to open database at {:?}: {}",
                    db_path, e
                ))
            })?;

            let rows_affected = conn.execute(
                "UPDATE cards SET streak = ?1 WHERE word_name = ?2",
                params![streak, word_name],
            ).map_err(|e| {
                PersistenceError::database_error(format!("Failed to update card streak: {}", e))
            })?;

            if rows_affected == 0 {
                return Err(PersistenceError::database_error(format!("Card with word_name '{}' not found", word_name)));
            }

            Ok(())
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn delete_card(&self, db_path: PathBuf, word_name: String) -> Result<bool, Self::Error> {
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| {
                PersistenceError::database_error(format!(
                    "Failed to open database at {:?}: {}",
                    db_path, e
                ))
            })?;

            let rows_affected = conn.execute(
                "DELETE FROM cards WHERE word_name = ?1",
                params![word_name],
            ).map_err(|e| {
                PersistenceError::database_error(format!("Failed to delete card: {}", e))
            })?;

            Ok(rows_affected > 0)
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }
}

impl SqliteProfileDbRepository {
    /// Helper function to fetch cards with their related data.
    fn fetch_cards(
        conn: &Connection,
        query: &str,
        params: &[&dyn rusqlite::ToSql],
    ) -> Result<Vec<Card>, PersistenceError> {
        let mut stmt = conn.prepare(query).map_err(|e| {
            PersistenceError::database_error(format!("Failed to prepare query: {}", e))
        })?;

        let card_entities: Vec<CardEntity> = stmt
            .query_map(params, |row| {
                Ok(CardEntity {
                    word_name: row.get(0)?,
                    card_type: row.get(1)?,
                    word_readings: row.get(2)?,
                    streak: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to query cards: {}", e))
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to collect cards: {}", e))
            })?;

        let mut cards = Vec::new();

        for card_entity in card_entities {
            // Fetch meanings
            let mut meaning_stmt = conn
                .prepare("SELECT id, word_name, definition, translated_definition, word_translations FROM meanings WHERE word_name = ?1")
                .map_err(|e| {
                    PersistenceError::database_error(format!("Failed to prepare meanings query: {}", e))
                })?;

            let meaning_entities: Vec<MeaningEntity> = meaning_stmt
                .query_map([&card_entity.word_name], |row| {
                    Ok(MeaningEntity {
                        id: row.get(0)?,
                        word_name: row.get(1)?,
                        definition: row.get(2)?,
                        translated_definition: row.get(3)?,
                        word_translations: row.get(4)?,
                    })
                })
                .map_err(|e| {
                    PersistenceError::database_error(format!("Failed to query meanings: {}", e))
                })?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    PersistenceError::database_error(format!("Failed to collect meanings: {}", e))
                })?;

            let card_with_relations = CardWithRelations {
                card: card_entity,
                meanings: meaning_entities,
            };

            cards.push(card_with_relations.to_domain()?);
        }

        Ok(cards)
    }
}
