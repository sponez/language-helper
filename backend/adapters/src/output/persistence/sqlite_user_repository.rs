use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
    time::{SystemTime, UNIX_EPOCH},
};

use application::ports::{
    input::local_user::models::{LocalUser, UserId},
    output::repository::user::{UserRepository, models::UserRepositoryError},
};
use async_trait::async_trait;
use rusqlite::{Connection, ErrorCode, OptionalExtension, params};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SqliteUserRepositoryInitError {
    #[error("failed to create database directory {path}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to open user database: {0}")]
    Open(#[source] rusqlite::Error),
    #[error("failed to initialize user database: {0}")]
    Initialize(#[source] rusqlite::Error),
}

/// SQLite implementation of the local user repository.
///
/// The schema intentionally matches the legacy `users` table so existing local
/// databases can be opened by the new application.
#[derive(Clone)]
pub struct SqliteUserRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteUserRepository {
    pub fn new(database_path: impl AsRef<Path>) -> Result<Self, SqliteUserRepositoryInitError> {
        let database_path = database_path.as_ref();
        if let Some(parent) = database_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|source| {
                SqliteUserRepositoryInitError::CreateDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }

        let connection =
            Connection::open(database_path).map_err(SqliteUserRepositoryInitError::Open)?;
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS users (
                    username TEXT PRIMARY KEY NOT NULL,
                    created_at INTEGER NOT NULL,
                    last_used_at INTEGER NOT NULL
                );
                ",
            )
            .map_err(SqliteUserRepositoryInitError::Initialize)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    fn lock_connection(&self) -> Result<MutexGuard<'_, Connection>, UserRepositoryError> {
        self.connection
            .lock()
            .map_err(|_| UserRepositoryError::Unavailable)
    }

    fn current_timestamp() -> Result<i64, UserRepositoryError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .map_err(|error| UserRepositoryError::Unexpected(error.to_string()))
    }

    fn map_sqlite_error(error: rusqlite::Error) -> UserRepositoryError {
        match &error {
            rusqlite::Error::SqliteFailure(details, _)
                if details.code == ErrorCode::ConstraintViolation =>
            {
                UserRepositoryError::AlreadyExists
            }
            rusqlite::Error::SqliteFailure(details, _)
                if matches!(
                    details.code,
                    ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
                ) =>
            {
                UserRepositoryError::Unavailable
            }
            _ => UserRepositoryError::Unexpected(error.to_string()),
        }
    }

    fn map_join_error(error: tokio::task::JoinError) -> UserRepositoryError {
        UserRepositoryError::Unexpected(format!("user repository task failed: {error}"))
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn insert(&self, user: LocalUser) -> Result<LocalUser, UserRepositoryError> {
        let repository = self.clone();
        tokio::task::spawn_blocking(move || {
            let timestamp = Self::current_timestamp()?;
            repository
                .lock_connection()?
                .execute(
                    "INSERT INTO users (username, created_at, last_used_at)
                     VALUES (?1, ?2, ?3)",
                    params![user.id.as_str(), timestamp, timestamp],
                )
                .map_err(Self::map_sqlite_error)?;

            Ok(user)
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn find(&self, user_id: &UserId) -> Result<Option<LocalUser>, UserRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        tokio::task::spawn_blocking(move || {
            repository
                .lock_connection()?
                .query_row(
                    "SELECT username FROM users WHERE username = ?1",
                    params![user_id.as_str()],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map(|username| {
                    username.map(|username| LocalUser {
                        id: UserId::new(username),
                    })
                })
                .map_err(Self::map_sqlite_error)
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn list(&self) -> Result<Vec<LocalUser>, UserRepositoryError> {
        let repository = self.clone();
        tokio::task::spawn_blocking(move || {
            let connection = repository.lock_connection()?;
            let mut statement = connection
                .prepare(
                    "SELECT username
                     FROM users
                     ORDER BY last_used_at DESC, username ASC",
                )
                .map_err(Self::map_sqlite_error)?;

            let users = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(Self::map_sqlite_error)?
                .map(|username| {
                    username
                        .map(|username| LocalUser {
                            id: UserId::new(username),
                        })
                        .map_err(Self::map_sqlite_error)
                })
                .collect();

            users
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn delete(&self, user_id: &UserId) -> Result<bool, UserRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        tokio::task::spawn_blocking(move || {
            repository
                .lock_connection()?
                .execute(
                    "DELETE FROM users WHERE username = ?1",
                    params![user_id.as_str()],
                )
                .map(|affected_rows| affected_rows > 0)
                .map_err(Self::map_sqlite_error)
        })
        .await
        .map_err(Self::map_join_error)?
    }
}

#[cfg(test)]
mod tests {
    use application::ports::output::repository::user::models::UserRepositoryError;
    use tempfile::TempDir;

    use super::*;

    fn repository() -> (SqliteUserRepository, TempDir, PathBuf) {
        let directory = TempDir::new().unwrap();
        let database_path = directory.path().join("users.db");
        let repository = SqliteUserRepository::new(&database_path).unwrap();
        (repository, directory, database_path)
    }

    #[tokio::test]
    async fn persists_a_user_in_the_legacy_compatible_schema() {
        let (repository, _directory, database_path) = repository();
        let user = LocalUser {
            id: UserId::new("persisted_user"),
        };

        repository.insert(user.clone()).await.unwrap();

        let reopened = SqliteUserRepository::new(database_path).unwrap();
        assert_eq!(reopened.find(&user.id).await.unwrap(), Some(user));
    }

    #[tokio::test]
    async fn reports_duplicate_users() {
        let (repository, _directory, _database_path) = repository();
        let user = LocalUser {
            id: UserId::new("duplicate_user"),
        };

        repository.insert(user.clone()).await.unwrap();

        assert_eq!(
            repository.insert(user).await,
            Err(UserRepositoryError::AlreadyExists)
        );
    }
}
