//! SQLite implementation for profile persistence.
//!
//! This module provides a SQLite-based repository for managing learning profiles
//! with a one-to-many relationship to users.

use crate::errors::PersistenceError;
use crate::models::ProfileEntity;
use lh_core::domain::profile::Profile;
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
                profile_id TEXT PRIMARY KEY NOT NULL,
                username TEXT NOT NULL,
                target_language TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_activity_at INTEGER NOT NULL,
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
    pub fn find_entity_by_id(
        &self,
        profile_id: &str,
    ) -> Result<Option<ProfileEntity>, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let result = conn.query_row(
            "SELECT profile_id, username, target_language, created_at, last_activity_at
             FROM profiles WHERE profile_id = ?1",
            params![profile_id],
            |row| {
                Ok(ProfileEntity {
                    profile_id: row.get(0)?,
                    username: row.get(1)?,
                    target_language: row.get(2)?,
                    created_at: row.get(3)?,
                    last_activity_at: row.get(4)?,
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
                "SELECT profile_id, username, target_language, created_at, last_activity_at
                 FROM profiles WHERE username = ?1 ORDER BY last_activity_at DESC",
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to prepare query: {}", e))
            })?;

        let entities = stmt
            .query_map(params![username], |row| {
                Ok(ProfileEntity {
                    profile_id: row.get(0)?,
                    username: row.get(1)?,
                    target_language: row.get(2)?,
                    created_at: row.get(3)?,
                    last_activity_at: row.get(4)?,
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
                "SELECT profile_id, username, target_language, created_at, last_activity_at
                 FROM profiles ORDER BY last_activity_at DESC",
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to prepare query: {}", e))
            })?;

        let entities = stmt
            .query_map([], |row| {
                Ok(ProfileEntity {
                    profile_id: row.get(0)?,
                    username: row.get(1)?,
                    target_language: row.get(2)?,
                    created_at: row.get(3)?,
                    last_activity_at: row.get(4)?,
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
    pub fn find_by_id(&self, profile_id: &str) -> Result<Option<Profile>, PersistenceError> {
        self.find_entity_by_id(profile_id)
            .map(|opt| opt.map(|entity| entity.to_domain()))
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
        self.find_entities_by_username(username)
            .map(|entities| entities.into_iter().map(|e| e.to_domain()).collect())
    }

    /// Retrieves all profiles as domain models.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Profile>)` - A vector of all profiles
    /// * `Err(PersistenceError)` - If the query fails
    pub fn find_all(&self) -> Result<Vec<Profile>, PersistenceError> {
        self.find_all_entities()
            .map(|entities| entities.into_iter().map(|e| e.to_domain()).collect())
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
    pub fn save(&self, profile: Profile) -> Result<Profile, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let entity = ProfileEntity::from_domain(profile.clone());

        conn.execute(
            "INSERT INTO profiles (profile_id, username, target_language, created_at, last_activity_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(profile_id) DO UPDATE SET
                target_language = ?3, last_activity_at = ?5",
            params![
                entity.profile_id,
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
    pub fn delete(&self, profile_id: &str) -> Result<bool, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let rows_affected = conn
            .execute(
                "DELETE FROM profiles WHERE profile_id = ?1",
                params![profile_id],
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to delete profile: {}", e))
            })?;

        Ok(rows_affected > 0)
    }
}

impl PersistenceProfileRepository for SqliteProfileRepository {
    type Error = PersistenceError;

    fn find_by_id(&self, profile_id: &str) -> Result<Option<Profile>, Self::Error> {
        self.find_by_id(profile_id)
    }

    fn find_by_username(&self, username: &str) -> Result<Vec<Profile>, Self::Error> {
        self.find_by_username(username)
    }

    fn find_all(&self) -> Result<Vec<Profile>, Self::Error> {
        self.find_all()
    }

    fn save(&self, profile: Profile) -> Result<Profile, Self::Error> {
        self.save(profile)
    }

    fn delete(&self, profile_id: &str) -> Result<bool, Self::Error> {
        self.delete(profile_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (SqliteProfileRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_profiles.db");
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

        let repo = SqliteProfileRepository::new(Arc::new(Mutex::new(connection))).unwrap();
        (repo, temp_dir)
    }

    fn insert_test_user(repo: &SqliteProfileRepository, username: &str) {
        let conn = repo.connection.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO users (username, created_at, last_used_at) VALUES (?1, ?2, ?3)",
            params![username, now, now],
        )
        .unwrap();
    }

    #[test]
    fn test_create_and_find_profile() {
        let (repo, _temp_dir) = create_test_repo();
        insert_test_user(&repo, "test_user");

        let profile = Profile::new("test_user".to_string(), "spanish".to_string()).unwrap();
        let saved = repo.save(profile.clone()).unwrap();

        assert_eq!(saved.profile_id, profile.profile_id);

        let found = repo.find_by_id(&profile.profile_id).unwrap();
        assert!(found.is_some());
        let found_profile = found.unwrap();
        assert_eq!(found_profile.username, "test_user");
        assert_eq!(found_profile.target_language, "spanish");
    }

    #[test]
    fn test_find_nonexistent_profile() {
        let (repo, _temp_dir) = create_test_repo();

        let result = repo.find_by_id("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_find_by_username() {
        let (repo, _temp_dir) = create_test_repo();
        insert_test_user(&repo, "multi_user");

        let profile1 = Profile::new("multi_user".to_string(), "spanish".to_string()).unwrap();
        let profile2 = Profile::new("multi_user".to_string(), "french".to_string()).unwrap();
        repo.save(profile1).unwrap();
        repo.save(profile2).unwrap();

        let profiles = repo.find_by_username("multi_user").unwrap();
        assert_eq!(profiles.len(), 2);
    }

    #[test]
    fn test_update_existing_profile() {
        let (repo, _temp_dir) = create_test_repo();
        insert_test_user(&repo, "update_test");

        let mut profile = Profile::new("update_test".to_string(), "german".to_string()).unwrap();
        repo.save(profile.clone()).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(1));
        profile.update_last_activity();
        profile.target_language = "italian".to_string();
        repo.save(profile.clone()).unwrap();

        let found = repo.find_by_id(&profile.profile_id).unwrap().unwrap();
        assert_eq!(found.target_language, "italian");
        assert!(found.last_activity_at > found.created_at);
    }

    #[test]
    fn test_delete_profile() {
        let (repo, _temp_dir) = create_test_repo();
        insert_test_user(&repo, "delete_test");

        let profile = Profile::new("delete_test".to_string(), "portuguese".to_string()).unwrap();
        let profile_id = profile.profile_id.clone();
        repo.save(profile).unwrap();

        let deleted = repo.delete(&profile_id).unwrap();
        assert!(deleted);

        let found = repo.find_by_id(&profile_id).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_delete_nonexistent_profile() {
        let (repo, _temp_dir) = create_test_repo();

        let deleted = repo.delete("nonexistent").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_find_all() {
        let (repo, _temp_dir) = create_test_repo();
        insert_test_user(&repo, "user1");
        insert_test_user(&repo, "user2");

        let profile1 = Profile::new("user1".to_string(), "spanish".to_string()).unwrap();
        let profile2 = Profile::new("user2".to_string(), "french".to_string()).unwrap();
        repo.save(profile1).unwrap();
        repo.save(profile2).unwrap();

        let profiles = repo.find_all().unwrap();
        assert_eq!(profiles.len(), 2);
    }
}
