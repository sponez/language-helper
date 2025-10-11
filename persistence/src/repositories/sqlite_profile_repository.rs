//! SQLite implementation for profile persistence.
//!
//! This module provides a SQLite-based repository for managing learning profiles
//! with a one-to-many relationship to users.

use crate::errors::PersistenceError;
use crate::mappers::profile_mapper;
use crate::models::ProfileEntity;
use lh_core::models::profile::Profile;
use lh_core::repositories::adapters::PersistenceProfileRepository;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

/// SQLite-based implementation of ProfileRepository.
///
/// This struct manages learning profiles using SQLite with a one-to-many
/// relationship to the users table (one user can have multiple profiles).
///
/// # Database Schema
///
/// The profiles table contains:
/// - `profile_id` (TEXT PRIMARY KEY): Unique profile identifier
/// - `username` (TEXT): Foreign key to users table
/// - `target_language` (TEXT): The language being learned
/// - `created_at` (INTEGER): Unix timestamp of creation
/// - `last_activity_at` (INTEGER): Unix timestamp of last activity
///
/// # Thread Safety
///
/// This implementation uses an `Arc<Mutex<Connection>>` to allow safe
/// concurrent access to the database connection.
pub struct SqliteProfileRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteProfileRepository {
    /// Creates a new SqliteProfileRepository instance.
    ///
    /// This will initialize the profiles table if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `connection` - Shared database connection
    ///
    /// # Returns
    ///
    /// * `Ok(SqliteProfileRepository)` - A new repository instance
    /// * `Err(PersistenceError)` - If initialization fails
    pub fn new(connection: Arc<Mutex<Connection>>) -> Result<Self, PersistenceError> {
        let repo = Self { connection };
        repo.initialize_schema()?;
        Ok(repo)
    }

    /// Initializes the database schema for profiles.
    fn initialize_schema(&self) -> Result<(), PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS profiles (
                username TEXT NOT NULL,
                target_language TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_activity_at INTEGER NOT NULL,
                PRIMARY KEY (username, target_language),
                FOREIGN KEY (username) REFERENCES users(username) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| PersistenceError::database_error(format!("Failed to create schema: {}", e)))?;

        // Create index for username lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_profiles_username ON profiles(username)",
            [],
        )
        .map_err(|e| PersistenceError::database_error(format!("Failed to create index: {}", e)))?;

        Ok(())
    }

    /// Retrieves a profile by its ID.
    ///
    /// # Arguments
    ///
    /// * `profile_id` - The profile ID to query
    ///
    /// # Returns
    ///
    /// * `Ok(Some(ProfileEntity))` - If the profile exists
    /// * `Ok(None)` - If the profile doesn't exist
    /// * `Err(PersistenceError)` - If the query fails
    pub fn find_entity_by_username_and_target_language(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<Option<ProfileEntity>, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let result = conn.query_row(
            "SELECT username, target_language, created_at, last_activity_at
             FROM profiles WHERE username = ?1 and target_language = ?2",
            params![username, target_language],
            |row| {
                Ok(ProfileEntity {
                    username: row.get(0)?,
                    target_language: row.get(1)?,
                    created_at: row.get(2)?,
                    last_activity_at: row.get(3)?,
                })
            },
        );

        match result {
            Ok(entity) => Ok(Some(entity)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(PersistenceError::database_error(format!(
                "Failed to query profile: {}",
                e
            ))),
        }
    }

    /// Retrieves all profiles for a specific user.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to query
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ProfileEntity>)` - A vector of profile entities
    /// * `Err(PersistenceError)` - If the query fails
    pub fn find_entities_by_username(
        &self,
        username: &str,
    ) -> Result<Vec<ProfileEntity>, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let mut stmt = conn
            .prepare(
                "SELECT username, target_language, created_at, last_activity_at
                 FROM profiles WHERE username = ?1 ORDER BY last_activity_at DESC",
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to prepare query: {}", e))
            })?;

        let entities = stmt
            .query_map(params![username], |row| {
                Ok(ProfileEntity {
                    username: row.get(0)?,
                    target_language: row.get(1)?,
                    created_at: row.get(2)?,
                    last_activity_at: row.get(3)?,
                })
            })
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to execute query: {}", e))
            })?
            .collect::<rusqlite::Result<Vec<ProfileEntity>>>()
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to collect results: {}", e))
            })?;

        Ok(entities)
    }

    /// Retrieves all profiles.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ProfileEntity>)` - A vector of all profile entities
    /// * `Err(PersistenceError)` - If the query fails
    pub fn find_all_entities(&self) -> Result<Vec<ProfileEntity>, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let mut stmt = conn
            .prepare(
                "SELECT username, target_language, created_at, last_activity_at
                 FROM profiles ORDER BY last_activity_at DESC",
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to prepare query: {}", e))
            })?;

        let entities = stmt
            .query_map([], |row| {
                Ok(ProfileEntity {
                    username: row.get(0)?,
                    target_language: row.get(1)?,
                    created_at: row.get(2)?,
                    last_activity_at: row.get(3)?,
                })
            })
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to execute query: {}", e))
            })?
            .collect::<rusqlite::Result<Vec<ProfileEntity>>>()
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to collect results: {}", e))
            })?;

        Ok(entities)
    }

    /// Retrieves a profile by ID as a domain model.
    ///
    /// # Arguments
    ///
    /// * `profile_id` - The profile ID to query
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Profile))` - If the profile exists
    /// * `Ok(None)` - If the profile doesn't exist
    /// * `Err(PersistenceError)` - If the query fails
    pub fn find_by_username_and_target_language(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<Option<Profile>, PersistenceError> {
        self.find_entity_by_username_and_target_language(username, target_language)
            .map(|opt| opt.map(|entity| profile_mapper::entity_to_model(&entity)))
    }

    /// Retrieves all profiles for a user as domain models.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to query
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Profile>)` - A vector of profiles
    /// * `Err(PersistenceError)` - If the query fails
    pub fn find_by_username(&self, username: &str) -> Result<Vec<Profile>, PersistenceError> {
        self.find_entities_by_username(username).map(|entities| {
            entities
                .into_iter()
                .map(|e| profile_mapper::entity_to_model(&e))
                .collect()
        })
    }

    /// Retrieves all profiles as domain models.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Profile>)` - A vector of all profiles
    /// * `Err(PersistenceError)` - If the query fails
    pub fn find_all(&self) -> Result<Vec<Profile>, PersistenceError> {
        self.find_all_entities().map(|entities| {
            entities
                .into_iter()
                .map(|e| profile_mapper::entity_to_model(&e))
                .collect()
        })
    }

    /// Saves a profile to the database.
    ///
    /// If a profile with the same ID exists, it will be updated.
    ///
    /// # Arguments
    ///
    /// * `profile` - The profile to save
    ///
    /// # Returns
    ///
    /// * `Ok(Profile)` - The saved profile
    /// * `Err(PersistenceError)` - If the save operation fails
    pub fn save(&self, username: &str, profile: Profile) -> Result<Profile, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let entity = profile_mapper::model_to_entity(username, &profile);

        conn.execute(
            "INSERT INTO profiles (profile_id, username, target_language, created_at, last_activity_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(profile_id) DO UPDATE SET
                target_language = ?3, last_activity_at = ?5",
            params![
                entity.username,
                entity.target_language,
                entity.created_at,
                entity.last_activity_at
            ],
        )
        .map_err(|e| PersistenceError::database_error(format!("Failed to save profile: {}", e)))?;

        Ok(profile)
    }

    /// Deletes a profile by ID.
    ///
    /// # Arguments
    ///
    /// * `profile_id` - The profile ID to delete
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the profile was deleted
    /// * `Ok(false)` - If the profile didn't exist
    /// * `Err(PersistenceError)` - If the delete operation fails
    pub fn delete(
        &self,
        username: &str,
        target_language: &str
    ) -> Result<bool, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let rows_affected = conn
            .execute(
                "DELETE FROM profiles WHERE username = ?1 and target_language = ?2",
                params![username, target_language],
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to delete profile: {}", e))
            })?;

        Ok(rows_affected > 0)
    }
}

impl PersistenceProfileRepository for SqliteProfileRepository {
    type Error = PersistenceError;

    fn find_by_username_and_target_language(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<Option<Profile>, Self::Error> {
        self.find_by_username_and_target_language(username, target_language)
    }

    fn find_by_username(&self, username: &str) -> Result<Vec<Profile>, Self::Error> {
        self.find_by_username(username)
    }

    fn find_all(&self) -> Result<Vec<Profile>, Self::Error> {
        self.find_all()
    }

    fn save(&self, username: &str, profile: Profile) -> Result<Profile, Self::Error> {
        self.save(username, profile)
    }

    fn delete(
        &self,
        username: &str,
        target_language: &str
    ) -> Result<bool, Self::Error> {
        self.delete(username, target_language)
    }
}
