use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use application::ports::{
    input::{ai_settings::models::AiSettings, local_user::models::UserId},
    output::repository::ai_settings::{AiSettingsRepository, models::AiSettingsRepositoryError},
};
use async_trait::async_trait;
use rusqlite::{Connection, ErrorCode, OptionalExtension, params};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SqliteAiSettingsRepositoryInitError {
    #[error("failed to create database directory {path:?}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to open AI settings database: {0}")]
    Open(#[source] rusqlite::Error),
    #[error("failed to initialize AI settings database: {0}")]
    Initialize(#[source] rusqlite::Error),
}

#[derive(Clone)]
pub struct SqliteAiSettingsRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteAiSettingsRepository {
    pub fn new(
        database_path: impl AsRef<Path>,
    ) -> Result<Self, SqliteAiSettingsRepositoryInitError> {
        let database_path = database_path.as_ref();
        if let Some(parent) = database_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|source| {
                SqliteAiSettingsRepositoryInitError::CreateDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }
        let connection =
            Connection::open(database_path).map_err(SqliteAiSettingsRepositoryInitError::Open)?;
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS ai_settings (
                    user_id TEXT PRIMARY KEY NOT NULL,
                    provider TEXT,
                    api_key TEXT,
                    model_name TEXT,
                    version INTEGER NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES users(username) ON DELETE CASCADE
                );
                ",
            )
            .map_err(SqliteAiSettingsRepositoryInitError::Initialize)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, AiSettingsRepositoryError> {
        self.connection
            .lock()
            .map_err(|_| AiSettingsRepositoryError::Unavailable)
    }

    fn map_error(error: rusqlite::Error) -> AiSettingsRepositoryError {
        match &error {
            rusqlite::Error::SqliteFailure(details, _)
                if matches!(
                    details.code,
                    ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
                ) =>
            {
                AiSettingsRepositoryError::Unavailable
            }
            _ => AiSettingsRepositoryError::Unexpected(error.to_string()),
        }
    }

    fn map_join(error: tokio::task::JoinError) -> AiSettingsRepositoryError {
        AiSettingsRepositoryError::Unexpected(format!(
            "AI settings repository task failed: {error}"
        ))
    }
}

#[async_trait]
impl AiSettingsRepository for SqliteAiSettingsRepository {
    async fn find(
        &self,
        user_id: &UserId,
    ) -> Result<Option<AiSettings>, AiSettingsRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        tokio::task::spawn_blocking(move || {
            repository
                .lock()?
                .query_row(
                    "SELECT provider, api_key, model_name, version
                     FROM ai_settings WHERE user_id = ?1",
                    params![user_id.as_str()],
                    |row| {
                        Ok(AiSettings {
                            owner_id: user_id.clone(),
                            provider: row.get(0)?,
                            api_key: row.get(1)?,
                            model_name: row.get(2)?,
                            version: row.get(3)?,
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
        mut settings: AiSettings,
        expected_version: u64,
    ) -> Result<AiSettings, AiSettingsRepositoryError> {
        let repository = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut connection = repository.lock()?;
            let transaction = connection.transaction().map_err(Self::map_error)?;
            let current_version = transaction
                .query_row(
                    "SELECT version FROM ai_settings WHERE user_id = ?1",
                    params![settings.owner_id.as_str()],
                    |row| row.get::<_, u64>(0),
                )
                .optional()
                .map_err(Self::map_error)?;
            if current_version.unwrap_or(0) != expected_version {
                return Err(AiSettingsRepositoryError::Conflict);
            }
            settings.version = expected_version + 1;
            transaction
                .execute(
                    "INSERT INTO ai_settings (user_id, provider, api_key, model_name, version)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(user_id) DO UPDATE SET
                         provider = excluded.provider,
                         api_key = excluded.api_key,
                         model_name = excluded.model_name,
                         version = excluded.version",
                    params![
                        settings.owner_id.as_str(),
                        settings.provider,
                        settings.api_key,
                        settings.model_name,
                        settings.version,
                    ],
                )
                .map_err(Self::map_error)?;
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
    async fn persists_settings_per_user_and_checks_versions() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("settings.db");
        let users = SqliteUserRepository::new(&path).unwrap();
        users
            .insert(application::ports::input::local_user::models::LocalUser {
                id: UserId::new("alice"),
            })
            .await
            .unwrap();
        let repository = SqliteAiSettingsRepository::new(&path).unwrap();
        let saved = repository
            .upsert(
                AiSettings {
                    owner_id: UserId::new("alice"),
                    provider: Some("openai".to_string()),
                    api_key: Some("secret".to_string()),
                    model_name: Some("gpt".to_string()),
                    version: 0,
                },
                0,
            )
            .await
            .unwrap();
        assert_eq!(saved.version, 1);
        drop(repository);

        let reopened = SqliteAiSettingsRepository::new(&path).unwrap();
        let loaded = reopened.find(&UserId::new("alice")).await.unwrap().unwrap();
        assert_eq!(loaded.provider.as_deref(), Some("openai"));
        assert_eq!(
            reopened.upsert(loaded.clone(), 0).await,
            Err(AiSettingsRepositoryError::Conflict)
        );
        assert!(users.delete(&loaded.owner_id).await.unwrap());
        assert!(reopened.find(&loaded.owner_id).await.unwrap().is_none());
    }
}
