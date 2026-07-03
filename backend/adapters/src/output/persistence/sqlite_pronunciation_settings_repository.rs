use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use application::ports::{
    input::{local_user::models::UserId, pronunciation_settings::models::PronunciationSettings},
    output::repository::pronunciation_settings::{
        PronunciationSettingsRepository, models::PronunciationSettingsRepositoryError,
    },
};
use async_trait::async_trait;
use rusqlite::{Connection, ErrorCode, OptionalExtension, params};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SqlitePronunciationSettingsRepositoryInitError {
    #[error("failed to create database directory {path:?}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to open pronunciation settings database: {0}")]
    Open(#[source] rusqlite::Error),
    #[error("failed to initialize pronunciation settings database: {0}")]
    Initialize(#[source] rusqlite::Error),
}

#[derive(Clone)]
pub struct SqlitePronunciationSettingsRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqlitePronunciationSettingsRepository {
    pub fn new(
        database_path: impl AsRef<Path>,
    ) -> Result<Self, SqlitePronunciationSettingsRepositoryInitError> {
        let database_path = database_path.as_ref();
        if let Some(parent) = database_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|source| {
                SqlitePronunciationSettingsRepositoryInitError::CreateDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }
        let connection = Connection::open(database_path)
            .map_err(SqlitePronunciationSettingsRepositoryInitError::Open)?;
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS pronunciation_settings (
                    user_id TEXT PRIMARY KEY NOT NULL,
                    azure_endpoint TEXT,
                    azure_subscription_key TEXT,
                    version INTEGER NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES users(username) ON DELETE CASCADE
                );
                ",
            )
            .map_err(SqlitePronunciationSettingsRepositoryInitError::Initialize)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, PronunciationSettingsRepositoryError> {
        self.connection
            .lock()
            .map_err(|_| PronunciationSettingsRepositoryError::Unavailable)
    }

    fn map_error(error: rusqlite::Error) -> PronunciationSettingsRepositoryError {
        match &error {
            rusqlite::Error::SqliteFailure(details, _)
                if matches!(
                    details.code,
                    ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
                ) =>
            {
                PronunciationSettingsRepositoryError::Unavailable
            }
            _ => PronunciationSettingsRepositoryError::Unexpected(error.to_string()),
        }
    }

    fn map_join(error: tokio::task::JoinError) -> PronunciationSettingsRepositoryError {
        PronunciationSettingsRepositoryError::Unexpected(format!(
            "pronunciation settings repository task failed: {error}"
        ))
    }
}

#[async_trait]
impl PronunciationSettingsRepository for SqlitePronunciationSettingsRepository {
    async fn find(
        &self,
        user_id: &UserId,
    ) -> Result<Option<PronunciationSettings>, PronunciationSettingsRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        tokio::task::spawn_blocking(move || {
            repository
                .lock()?
                .query_row(
                    "SELECT azure_endpoint, azure_subscription_key, version
                     FROM pronunciation_settings WHERE user_id = ?1",
                    params![user_id.as_str()],
                    |row| {
                        Ok(PronunciationSettings {
                            owner_id: user_id.clone(),
                            endpoint: row.get(0)?,
                            subscription_key: row.get(1)?,
                            version: row.get(2)?,
                        })
                    },
                )
                .optional()
                .map_err(Self::map_error)
        })
        .await
        .map_err(Self::map_join)?
    }

    async fn upsert(
        &self,
        mut settings: PronunciationSettings,
        expected_version: u64,
    ) -> Result<PronunciationSettings, PronunciationSettingsRepositoryError> {
        let repository = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut connection = repository.lock()?;
            let transaction = connection.transaction().map_err(Self::map_error)?;
            let current_version = transaction
                .query_row(
                    "SELECT version FROM pronunciation_settings WHERE user_id = ?1",
                    params![settings.owner_id.as_str()],
                    |row| row.get::<_, u64>(0),
                )
                .optional()
                .map_err(Self::map_error)?;
            if current_version.unwrap_or(0) != expected_version {
                return Err(PronunciationSettingsRepositoryError::Conflict);
            }
            settings.version = expected_version + 1;
            if current_version.is_some() {
                transaction
                    .execute(
                        "UPDATE pronunciation_settings
                         SET azure_endpoint = ?1, azure_subscription_key = ?2, version = ?3
                         WHERE user_id = ?4 AND version = ?5",
                        params![
                            settings.endpoint,
                            settings.subscription_key,
                            settings.version,
                            settings.owner_id.as_str(),
                            expected_version,
                        ],
                    )
                    .map_err(Self::map_error)?;
            } else {
                transaction
                    .execute(
                        "INSERT INTO pronunciation_settings
                         (user_id, azure_endpoint, azure_subscription_key, version)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![
                            settings.owner_id.as_str(),
                            settings.endpoint,
                            settings.subscription_key,
                            settings.version,
                        ],
                    )
                    .map_err(Self::map_error)?;
            }
            transaction.commit().map_err(Self::map_error)?;
            Ok(settings)
        })
        .await
        .map_err(Self::map_join)?
    }
}

#[cfg(test)]
mod tests {
    use application::ports::output::repository::UserRepository;
    use tempfile::TempDir;

    use super::*;
    use crate::output::persistence::SqliteUserRepository;

    #[tokio::test]
    async fn persists_user_scoped_settings_after_reopening() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("settings.db");
        SqliteUserRepository::new(&path)
            .unwrap()
            .insert(application::ports::input::local_user::models::LocalUser {
                id: UserId::new("alice"),
            })
            .await
            .unwrap();
        let repository = SqlitePronunciationSettingsRepository::new(&path).unwrap();
        let saved = repository
            .upsert(
                PronunciationSettings {
                    owner_id: UserId::new("alice"),
                    endpoint: Some("https://speech.example".to_string()),
                    subscription_key: Some("secret".to_string()),
                    version: 0,
                },
                0,
            )
            .await
            .unwrap();
        assert_eq!(saved.version, 1);
        drop(repository);

        let reopened = SqlitePronunciationSettingsRepository::new(&path).unwrap();
        let loaded = reopened.find(&UserId::new("alice")).await.unwrap().unwrap();
        assert_eq!(loaded.endpoint.as_deref(), Some("https://speech.example"));
        assert_eq!(loaded.subscription_key.as_deref(), Some("secret"));
        assert_eq!(
            reopened.upsert(loaded, 0).await,
            Err(PronunciationSettingsRepositoryError::Conflict)
        );
    }
}
