//! SQLite implementation of the UserRepository trait.
//!
//! This module provides a SQLite-based implementation of the UserRepository
//! trait, managing user persistence in a local database file.

use chrono::{DateTime, Utc};
use lh_core::domain::user::User;
use lh_core::errors::CoreError;
use lh_core::repositories::user_repository::UserRepository;
use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// SQLite-based implementation of UserRepository.
///
/// This struct manages user data persistence using SQLite. It stores users
/// with their creation and last used timestamps in a local database file.
///
/// # Database Schema
///
/// The users table contains:
/// - `nickname` (TEXT PRIMARY KEY): The username
/// - `created_at` (TEXT): ISO 8601 timestamp of user creation
/// - `last_used_at` (TEXT): ISO 8601 timestamp of last usage
///
/// # Thread Safety
///
/// This implementation uses an `Arc<Mutex<Connection>>` to allow safe
/// concurrent access to the database connection.
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
    /// * `Err(CoreError)` - If database initialization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lh_persistence::sqlite_user_repository::SqliteUserRepository;
    ///
    /// let repo = SqliteUserRepository::new("data/users.db").unwrap();
    /// ```
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, CoreError> {
        // Ensure the parent directory exists
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CoreError::repository_error(format!("Failed to create database directory: {}", e))
            })?;
        }

        // Open or create the database connection
        let connection = Connection::open(db_path).map_err(|e| {
            CoreError::repository_error(format!("Failed to open database: {}", e))
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
    /// Creates the users table if it doesn't exist.
    fn initialize_schema(&self) -> Result<(), CoreError> {
        let conn = self.connection.lock().map_err(|e| {
            CoreError::repository_error(format!("Failed to acquire database lock: {}", e))
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                nickname TEXT PRIMARY KEY NOT NULL,
                created_at TEXT NOT NULL,
                last_used_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| CoreError::repository_error(format!("Failed to create schema: {}", e)))?;

        Ok(())
    }

    /// Updates the last_used_at timestamp for a user.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to update
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update was successful
    /// * `Err(CoreError)` - If the update fails
    pub fn update_last_used(&self, username: &str) -> Result<(), CoreError> {
        let conn = self.connection.lock().map_err(|e| {
            CoreError::repository_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let now = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE users SET last_used_at = ?1 WHERE nickname = ?2",
            params![now, username],
        )
        .map_err(|e| {
            CoreError::repository_error(format!("Failed to update last_used_at: {}", e))
        })?;

        Ok(())
    }

    /// Retrieves the creation and last used timestamps for a user.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to query
    ///
    /// # Returns
    ///
    /// * `Ok(Some((created_at, last_used_at)))` - If the user exists
    /// * `Ok(None)` - If the user doesn't exist
    /// * `Err(CoreError)` - If the query fails
    pub fn get_timestamps(
        &self,
        username: &str,
    ) -> Result<Option<(DateTime<Utc>, DateTime<Utc>)>, CoreError> {
        let conn = self.connection.lock().map_err(|e| {
            CoreError::repository_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let result: SqliteResult<(String, String)> = conn.query_row(
            "SELECT created_at, last_used_at FROM users WHERE nickname = ?1",
            params![username],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );

        match result {
            Ok((created_str, last_used_str)) => {
                let created_at = DateTime::parse_from_rfc3339(&created_str)
                    .map_err(|e| {
                        CoreError::repository_error(format!("Failed to parse created_at: {}", e))
                    })?
                    .with_timezone(&Utc);

                let last_used_at = DateTime::parse_from_rfc3339(&last_used_str)
                    .map_err(|e| {
                        CoreError::repository_error(format!("Failed to parse last_used_at: {}", e))
                    })?
                    .with_timezone(&Utc);

                Ok(Some((created_at, last_used_at)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(CoreError::repository_error(format!(
                "Failed to query timestamps: {}",
                e
            ))),
        }
    }
}

impl UserRepository for SqliteUserRepository {
    fn find_by_username(&self, username: &str) -> Result<Option<User>, CoreError> {
        let conn = self.connection.lock().map_err(|e| {
            CoreError::repository_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let result: SqliteResult<String> = conn.query_row(
            "SELECT nickname FROM users WHERE nickname = ?1",
            params![username],
            |row| row.get(0),
        );

        match result {
            Ok(nickname) => Ok(Some(User::new(nickname))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(CoreError::repository_error(format!(
                "Failed to query user: {}",
                e
            ))),
        }
    }

    fn find_all(&self) -> Result<Vec<User>, CoreError> {
        let conn = self.connection.lock().map_err(|e| {
            CoreError::repository_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let mut stmt = conn
            .prepare("SELECT nickname FROM users ORDER BY last_used_at DESC")
            .map_err(|e| CoreError::repository_error(format!("Failed to prepare query: {}", e)))?;

        let users = stmt
            .query_map([], |row| {
                let nickname: String = row.get(0)?;
                Ok(User::new(nickname))
            })
            .map_err(|e| CoreError::repository_error(format!("Failed to execute query: {}", e)))?
            .collect::<SqliteResult<Vec<User>>>()
            .map_err(|e| CoreError::repository_error(format!("Failed to collect results: {}", e)))?;

        Ok(users)
    }

    fn save(&self, user: User) -> Result<User, CoreError> {
        let conn = self.connection.lock().map_err(|e| {
            CoreError::repository_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let now = Utc::now().to_rfc3339();

        // Try to insert; if the user exists, update last_used_at
        let result = conn.execute(
            "INSERT INTO users (nickname, created_at, last_used_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(nickname) DO UPDATE SET last_used_at = ?3",
            params![user.username, now, now],
        );

        match result {
            Ok(_) => Ok(user),
            Err(e) => Err(CoreError::repository_error(format!(
                "Failed to save user: {}",
                e
            ))),
        }
    }

    fn delete(&self, username: &str) -> Result<bool, CoreError> {
        let conn = self.connection.lock().map_err(|e| {
            CoreError::repository_error(format!("Failed to acquire database lock: {}", e))
        })?;

        let rows_affected = conn
            .execute("DELETE FROM users WHERE nickname = ?1", params![username])
            .map_err(|e| CoreError::repository_error(format!("Failed to delete user: {}", e)))?;

        Ok(rows_affected > 0)
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

        let user = User::new("test_user".to_string());
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

        repo.save(User::new("user1".to_string())).unwrap();
        repo.save(User::new("user2".to_string())).unwrap();
        repo.save(User::new("user3".to_string())).unwrap();

        let users = repo.find_all().unwrap();
        assert_eq!(users.len(), 3);
    }

    #[test]
    fn test_update_existing_user() {
        let (repo, _temp_dir) = create_test_repo();

        let user = User::new("update_test".to_string());
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

        let user = User::new("delete_test".to_string());
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

        let user = User::new("timestamp_test".to_string());
        repo.save(user).unwrap();

        let timestamps = repo.get_timestamps("timestamp_test").unwrap();
        assert!(timestamps.is_some());

        let (created_at, last_used_at) = timestamps.unwrap();
        assert_eq!(created_at, last_used_at);
    }

    #[test]
    fn test_update_last_used() {
        let (repo, _temp_dir) = create_test_repo();

        let user = User::new("last_used_test".to_string());
        repo.save(user).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));
        repo.update_last_used("last_used_test").unwrap();

        let timestamps = repo.get_timestamps("last_used_test").unwrap();
        assert!(timestamps.is_some());

        let (created_at, last_used_at) = timestamps.unwrap();
        assert!(last_used_at > created_at);
    }

    #[test]
    fn test_database_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("nested").join("directory").join("users.db");

        let repo = SqliteUserRepository::new(&db_path).unwrap();
        assert!(db_path.exists());

        // Verify we can use the repository
        let user = User::new("test".to_string());
        repo.save(user).unwrap();
    }
}
