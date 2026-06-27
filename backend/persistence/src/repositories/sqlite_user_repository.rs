//! SQLite implementation for user persistence.
//!
//! This module provides a SQLite-based repository for managing user data
//! persistence in a local database file with full timestamp tracking using
//! Unix timestamps.

use crate::errors::PersistenceError;
use crate::mappers::user_mapper;
use crate::models::UserEntity;
use lh_core::models::user::User;
use lh_core::repositories::adapters::PersistenceUserRepository;
use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// SQLite-based implementation of UserRepository.
///
/// This struct manages user data persistence using SQLite. It stores users
/// with their creation and last used timestamps as Unix timestamps (INTEGER)
/// for efficient storage and querying.
///
/// # Database Schema
///
/// The users table contains:
/// - `username` (TEXT PRIMARY KEY): The username
/// - `created_at` (INTEGER): Unix timestamp (seconds since epoch) of creation
/// - `last_used_at` (INTEGER): Unix timestamp (seconds since epoch) of last use
///
/// # Thread Safety
///
/// This implementation uses an `Arc<Mutex<Connection>>` to allow safe
/// concurrent access to the database connection.
///
/// # Examples
///
/// ```no_run
/// use lh_persistence::SqliteUserRepository;
///
/// let repo = SqliteUserRepository::new("data/users.db").unwrap();
/// ```
pub struct SqliteUserRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteUserRepository {
    /// Creates a new SqliteUserRepository instance.
    ///
    /// This method will:
    /// 1. Create the parent directory if it doesn't exist
    /// 2. Open or create the database file at the specified path
    /// 3. Initialize the database schema if needed
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the SQLite database file
    ///
    /// # Returns
    ///
    /// * `Ok(SqliteUserRepository)` - A new repository instance
    /// * `Err(PersistenceError)` - If database initialization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lh_persistence::SqliteUserRepository;
    ///
    /// let repo = SqliteUserRepository::new("data/users.db").unwrap();
    /// ```
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, PersistenceError> {
        // Ensure the parent directory exists
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                PersistenceError::io_error(format!("Failed to create database directory: {}", e))
            })?;
        }

        // Open or create the database connection
        let connection = Connection::open(db_path).map_err(|e| {
            PersistenceError::database_error(format!("Failed to open database: {}", e))
        })?;

        let repo = Self {
            connection: Arc::new(Mutex::new(connection)),
        };

        // Initialize the database schema
        repo.initialize_schema()?;

        Ok(repo)
    }

    /// Initializes the database schema.
    ///
    /// Creates the users table if it doesn't exist, using INTEGER for timestamps.
    fn initialize_schema(&self) -> Result<(), PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                username TEXT PRIMARY KEY NOT NULL,
                created_at INTEGER NOT NULL,
                last_used_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| PersistenceError::database_error(format!("Failed to create schema: {}", e)))?;

        Ok(())
    }

    /// Retrieves a full UserEntity from the database.
    ///
    /// This method returns the complete persistence model including all timestamps.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to query
    ///
    /// # Returns
    ///
    /// * `Ok(Some(UserEntity))` - If the user exists
    /// * `Ok(None)` - If the user doesn't exist
    /// * `Err(PersistenceError)` - If the query fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lh_persistence::SqliteUserRepository;
    ///
    /// let repo = SqliteUserRepository::new("data/users.db").unwrap();
    /// if let Some(entity) = repo.find_entity_by_username("john").unwrap() {
    ///     println!("User created at: {}", entity.created_at);
    /// }
    /// ```
    pub fn find_entity_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserEntity>, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let result: SqliteResult<(String, i64, i64)> = conn.query_row(
            "SELECT username, created_at, last_used_at FROM users WHERE username = ?1",
            params![username],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );

        match result {
            Ok((username, created_at, last_used_at)) => Ok(Some(UserEntity {
                username,
                created_at,
                last_used_at,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(PersistenceError::database_error(format!(
                "Failed to query user: {}",
                e
            ))),
        }
    }

    /// Retrieves all UserEntity records from the database.
    ///
    /// This method returns complete persistence models including all timestamps,
    /// ordered by most recently used first.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<UserEntity>)` - A vector of all user entities
    /// * `Err(PersistenceError)` - If the query fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lh_persistence::SqliteUserRepository;
    ///
    /// let repo = SqliteUserRepository::new("data/users.db").unwrap();
    /// for entity in repo.find_all_entities().unwrap() {
    ///     println!("User: {}, Last used: {}", entity.username, entity.last_used_at);
    /// }
    /// ```
    pub fn find_all_entities(&self) -> Result<Vec<UserEntity>, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let mut stmt = conn
            .prepare(
                "SELECT username, created_at, last_used_at FROM users ORDER BY last_used_at DESC",
            )
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to prepare query: {}", e))
            })?;

        let entities = stmt
            .query_map([], |row| {
                Ok(UserEntity {
                    username: row.get(0)?,
                    created_at: row.get(1)?,
                    last_used_at: row.get(2)?,
                })
            })
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to execute query: {}", e))
            })?
            .collect::<SqliteResult<Vec<UserEntity>>>()
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to collect results: {}", e))
            })?;

        Ok(entities)
    }

    /// Finds a user by username and returns the domain model.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to query
    ///
    /// # Returns
    ///
    /// * `Ok(Some(User))` - If the user exists
    /// * `Ok(None)` - If the user doesn't exist
    /// * `Err(PersistenceError)` - If the query fails
    pub fn find_by_username(&self, username: &str) -> Result<Option<User>, PersistenceError> {
        self.find_entity_by_username(username)
            .map(|opt_entity| opt_entity.map(|entity| user_mapper::entity_to_model(&entity)))
    }

    /// Retrieves all users and returns domain models.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<User>)` - A vector of all users
    /// * `Err(PersistenceError)` - If the query fails
    pub fn find_all(&self) -> Result<Vec<User>, PersistenceError> {
        self.find_all_entities().map(|entities| {
            entities
                .into_iter()
                .map(|entity| user_mapper::entity_to_model(&entity))
                .collect()
        })
    }

    /// Saves a user to the database.
    ///
    /// If the user already exists, updates the last_used_at timestamp.
    ///
    /// # Arguments
    ///
    /// * `user` - The user to save
    ///
    /// # Returns
    ///
    /// * `Ok(User)` - The saved user
    /// * `Err(PersistenceError)` - If the save operation fails
    pub fn save(&self, user: User) -> Result<User, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let entity = user_mapper::model_to_entity(&user);

        // Try to insert; if the user exists, update last_used_at
        conn.execute(
            "INSERT INTO users (username, created_at, last_used_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(username) DO UPDATE SET last_used_at = ?3",
            params![entity.username, entity.created_at, entity.last_used_at],
        )
        .map_err(|e| PersistenceError::database_error(format!("Failed to save user: {}", e)))?;

        Ok(user)
    }

    /// Deletes a user from the database.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to delete
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the user was deleted
    /// * `Ok(false)` - If the user didn't exist
    /// * `Err(PersistenceError)` - If the delete operation fails
    pub fn delete(&self, username: &str) -> Result<bool, PersistenceError> {
        let conn = self.connection.lock().map_err(|e| {
            PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let rows_affected = conn
            .execute("DELETE FROM users WHERE username = ?1", params![username])
            .map_err(|e| {
                PersistenceError::database_error(format!("Failed to delete user: {}", e))
            })?;

        Ok(rows_affected > 0)
    }
}

#[async_trait::async_trait]
impl PersistenceUserRepository for SqliteUserRepository {
    type Error = PersistenceError;

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Self::Error> {
        let username = username.to_string();
        let conn = self.connection.clone();
        tokio::task::spawn_blocking(move || {
            let connection = conn.lock().map_err(|e| {
                PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
            })?;

            let result: SqliteResult<(String, i64, i64)> = connection.query_row(
                "SELECT username, created_at, last_used_at FROM users WHERE username = ?1",
                params![username],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            );

            match result {
                Ok((username, created_at, last_used_at)) => {
                    let entity = UserEntity {
                        username,
                        created_at,
                        last_used_at,
                    };
                    Ok(Some(user_mapper::entity_to_model(&entity)))
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(PersistenceError::database_error(format!(
                    "Failed to query user: {}",
                    e
                ))),
            }
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn find_all(&self) -> Result<Vec<User>, Self::Error> {
        let conn = self.connection.clone();
        tokio::task::spawn_blocking(move || {
            let connection = conn.lock().map_err(|e| {
                PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
            })?;

            let mut stmt = connection
                .prepare(
                    "SELECT username, created_at, last_used_at FROM users ORDER BY last_used_at DESC",
                )
                .map_err(|e| {
                    PersistenceError::database_error(format!("Failed to prepare query: {}", e))
                })?;

            let entities = stmt
                .query_map([], |row| {
                    Ok(UserEntity {
                        username: row.get(0)?,
                        created_at: row.get(1)?,
                        last_used_at: row.get(2)?,
                    })
                })
                .map_err(|e| {
                    PersistenceError::database_error(format!("Failed to execute query: {}", e))
                })?
                .collect::<SqliteResult<Vec<UserEntity>>>()
                .map_err(|e| {
                    PersistenceError::database_error(format!("Failed to collect results: {}", e))
                })?;

            Ok(entities
                .into_iter()
                .map(|entity| user_mapper::entity_to_model(&entity))
                .collect())
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn save(&self, user: User) -> Result<User, Self::Error> {
        let conn = self.connection.clone();
        tokio::task::spawn_blocking(move || {
            let connection = conn.lock().map_err(|e| {
                PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
            })?;

            let entity = user_mapper::model_to_entity(&user);

            connection
                .execute(
                    "INSERT INTO users (username, created_at, last_used_at) VALUES (?1, ?2, ?3)
                     ON CONFLICT(username) DO UPDATE SET last_used_at = ?3",
                    params![entity.username, entity.created_at, entity.last_used_at],
                )
                .map_err(|e| {
                    PersistenceError::database_error(format!("Failed to save user: {}", e))
                })?;

            Ok(user)
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }

    async fn delete(&self, username: &str) -> Result<bool, Self::Error> {
        let username = username.to_string();
        let conn = self.connection.clone();
        tokio::task::spawn_blocking(move || {
            let connection = conn.lock().map_err(|e| {
                PersistenceError::lock_error(format!("Failed to acquire database lock: {}", e))
            })?;

            let rows_affected = connection
                .execute("DELETE FROM users WHERE username = ?1", params![username])
                .map_err(|e| {
                    PersistenceError::database_error(format!("Failed to delete user: {}", e))
                })?;

            Ok(rows_affected > 0)
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (SqliteUserRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_users.db");
        let repo = SqliteUserRepository::new(db_path).unwrap();
        (repo, temp_dir)
    }

    #[test]
    fn test_create_and_find_user() {
        let (repo, _temp_dir) = create_test_repo();

        let user = User::new_unchecked("test_user".to_string());
        let saved = repo.save(user.clone()).unwrap();

        assert_eq!(saved.username, user.username);

        let found = repo.find_by_username("test_user").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, "test_user");
    }

    #[test]
    fn test_find_nonexistent_user() {
        let (repo, _temp_dir) = create_test_repo();

        let result = repo.find_by_username("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_find_all_users() {
        let (repo, _temp_dir) = create_test_repo();

        repo.save(User::new_unchecked("user1".to_string())).unwrap();
        repo.save(User::new_unchecked("user2".to_string())).unwrap();
        repo.save(User::new_unchecked("user3".to_string())).unwrap();

        let users = repo.find_all().unwrap();
        assert_eq!(users.len(), 3);
    }

    #[test]
    fn test_update_existing_user() {
        let (repo, _temp_dir) = create_test_repo();

        let user = User::new_unchecked("update_test".to_string());
        repo.save(user.clone()).unwrap();

        // Save again - should update last_used_at
        std::thread::sleep(std::time::Duration::from_millis(10));
        repo.save(user).unwrap();

        let users = repo.find_all().unwrap();
        assert_eq!(users.len(), 1);
    }

    #[test]
    fn test_delete_user() {
        let (repo, _temp_dir) = create_test_repo();

        let user = User::new_unchecked("delete_test".to_string());
        repo.save(user).unwrap();

        let deleted = repo.delete("delete_test").unwrap();
        assert!(deleted);

        let found = repo.find_by_username("delete_test").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_delete_nonexistent_user() {
        let (repo, _temp_dir) = create_test_repo();

        let deleted = repo.delete("nonexistent").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_timestamps() {
        let (repo, _temp_dir) = create_test_repo();

        let user = User::new_unchecked("timestamp_test".to_string());
        repo.save(user).unwrap();

        let entity = repo.find_entity_by_username("timestamp_test").unwrap();
        assert!(entity.is_some());

        let entity = entity.unwrap();
        assert!(entity.created_at > 0);
        assert!(entity.last_used_at > 0);
        assert_eq!(entity.created_at, entity.last_used_at);
    }

    #[test]
    fn test_update_last_used() {
        let (repo, _temp_dir) = create_test_repo();

        let user = User::new_unchecked("last_used_test".to_string());
        repo.save(user.clone()).unwrap();

        let entity_before = repo
            .find_entity_by_username("last_used_test")
            .unwrap()
            .unwrap();

        // Sleep for at least 1 second since Unix timestamps are in seconds
        std::thread::sleep(std::time::Duration::from_secs(1));
        repo.save(user).unwrap();

        let entity_after = repo
            .find_entity_by_username("last_used_test")
            .unwrap()
            .unwrap();

        assert_eq!(entity_before.created_at, entity_after.created_at);
        assert!(entity_after.last_used_at > entity_before.last_used_at);
    }

    #[test]
    fn test_database_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir
            .path()
            .join("nested")
            .join("directory")
            .join("users.db");

        let repo = SqliteUserRepository::new(&db_path).unwrap();
        assert!(db_path.exists());

        // Verify we can use the repository
        let user = User::new_unchecked("test".to_string());
        repo.save(user).unwrap();
    }

    #[test]
    fn test_find_all_entities() {
        let (repo, _temp_dir) = create_test_repo();

        // Create users with delays to ensure different timestamps
        repo.save(User::new_unchecked("user1".to_string())).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        repo.save(User::new_unchecked("user2".to_string())).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        repo.save(User::new_unchecked("user3".to_string())).unwrap();

        let entities = repo.find_all_entities().unwrap();
        assert_eq!(entities.len(), 3);

        // Should be ordered by last_used_at DESC
        assert_eq!(entities[0].username, "user3");
        assert_eq!(entities[1].username, "user2");
        assert_eq!(entities[2].username, "user1");

        // All should have valid timestamps
        for entity in entities {
            assert!(entity.created_at > 0);
            assert!(entity.last_used_at > 0);
        }
    }

    #[test]
    fn test_integer_timestamp_storage() {
        let (repo, _temp_dir) = create_test_repo();

        let user = User::new_unchecked("timestamp_storage_test".to_string());
        repo.save(user).unwrap();

        // Verify timestamps are integers by querying directly
        let entity = repo
            .find_entity_by_username("timestamp_storage_test")
            .unwrap()
            .unwrap();

        // Unix timestamps should be positive integers
        assert!(entity.created_at > 1_600_000_000); // After 2020
        assert!(entity.last_used_at > 1_600_000_000);
    }
}
