use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use application::ports::{
    input::{
        language_profile::models::{
            AiProviderSettings, AnswerMode, LanguageProfile, LearningSettings, ProfileId,
        },
        local_user::models::UserId,
    },
    output::repository::language_profile::{
        LanguageProfileRepository, models::LanguageProfileRepositoryError,
    },
};
use async_trait::async_trait;
use rusqlite::{Connection, ErrorCode, OptionalExtension, params};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SqliteLanguageProfileRepositoryInitError {
    #[error("failed to create database directory {path}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to open language profile database: {0}")]
    Open(#[source] rusqlite::Error),
    #[error("failed to initialize language profile database: {0}")]
    Initialize(#[source] rusqlite::Error),
}

#[derive(Clone)]
pub struct SqliteLanguageProfileRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteLanguageProfileRepository {
    pub fn new(
        database_path: impl AsRef<Path>,
    ) -> Result<Self, SqliteLanguageProfileRepositoryInitError> {
        let database_path = database_path.as_ref();
        if let Some(parent) = database_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|source| {
                SqliteLanguageProfileRepositoryInitError::CreateDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }

        let connection = Connection::open(database_path)
            .map_err(SqliteLanguageProfileRepositoryInitError::Open)?;
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS language_profiles (
                    id TEXT PRIMARY KEY NOT NULL,
                    user_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    source_language TEXT NOT NULL,
                    target_language TEXT NOT NULL,
                    cards_per_set INTEGER NOT NULL,
                    answer_mode TEXT NOT NULL,
                    mastery_threshold INTEGER NOT NULL,
                    check_reading_if_possible INTEGER NOT NULL,
                    ai_provider TEXT,
                    ai_api_key TEXT,
                    ai_model_name TEXT,
                    version INTEGER NOT NULL DEFAULT 0,
                    FOREIGN KEY (user_id) REFERENCES users(username) ON DELETE CASCADE,
                    UNIQUE (user_id, name)
                );

                CREATE INDEX IF NOT EXISTS idx_language_profiles_user_id
                    ON language_profiles(user_id);
                ",
            )
            .map_err(SqliteLanguageProfileRepositoryInitError::Initialize)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    fn lock_connection(
        &self,
    ) -> Result<MutexGuard<'_, Connection>, LanguageProfileRepositoryError> {
        self.connection
            .lock()
            .map_err(|_| LanguageProfileRepositoryError::Unavailable)
    }

    fn map_sqlite_error(error: rusqlite::Error) -> LanguageProfileRepositoryError {
        match &error {
            rusqlite::Error::SqliteFailure(details, _)
                if details.code == ErrorCode::ConstraintViolation =>
            {
                LanguageProfileRepositoryError::AlreadyExists
            }
            rusqlite::Error::SqliteFailure(details, _)
                if matches!(
                    details.code,
                    ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
                ) =>
            {
                LanguageProfileRepositoryError::Unavailable
            }
            _ => LanguageProfileRepositoryError::Unexpected(error.to_string()),
        }
    }

    fn map_join_error(error: tokio::task::JoinError) -> LanguageProfileRepositoryError {
        LanguageProfileRepositoryError::Unexpected(format!(
            "language profile repository task failed: {error}"
        ))
    }

    fn answer_mode_name(mode: &AnswerMode) -> &'static str {
        match mode {
            AnswerMode::Written => "written",
            AnswerMode::SelfReview => "self_review",
        }
    }

    fn parse_answer_mode(value: String) -> Result<AnswerMode, rusqlite::Error> {
        match value.as_str() {
            "written" => Ok(AnswerMode::Written),
            "self_review" => Ok(AnswerMode::SelfReview),
            _ => Err(rusqlite::Error::InvalidQuery),
        }
    }

    fn read_profile(row: &rusqlite::Row<'_>) -> rusqlite::Result<LanguageProfile> {
        let answer_mode = Self::parse_answer_mode(row.get(6)?)?;
        Ok(LanguageProfile {
            id: ProfileId::new(row.get::<_, String>(0)?),
            owner_id: UserId::new(row.get::<_, String>(1)?),
            name: row.get(2)?,
            source_language: row.get(3)?,
            target_language: row.get(4)?,
            settings: LearningSettings {
                cards_per_set: row.get(5)?,
                answer_mode,
                mastery_threshold: row.get(7)?,
                check_reading_if_possible: row.get(8)?,
            },
            ai_settings: AiProviderSettings {
                provider: row.get(9)?,
                api_key: row.get(10)?,
                model_name: row.get(11)?,
            },
            version: row.get(12)?,
        })
    }
}

#[async_trait]
impl LanguageProfileRepository for SqliteLanguageProfileRepository {
    async fn insert(
        &self,
        profile: LanguageProfile,
    ) -> Result<LanguageProfile, LanguageProfileRepositoryError> {
        let repository = self.clone();
        let stored = profile.clone();
        tokio::task::spawn_blocking(move || {
            repository
                .lock_connection()?
                .execute(
                    "INSERT INTO language_profiles (
                        id, user_id, name, source_language, target_language,
                        cards_per_set, answer_mode, mastery_threshold,
                        check_reading_if_possible, ai_provider, ai_api_key,
                        ai_model_name, version
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                    params![
                        stored.id.as_str(),
                        stored.owner_id.as_str(),
                        stored.name,
                        stored.source_language,
                        stored.target_language,
                        stored.settings.cards_per_set,
                        Self::answer_mode_name(&stored.settings.answer_mode),
                        stored.settings.mastery_threshold,
                        stored.settings.check_reading_if_possible,
                        stored.ai_settings.provider,
                        stored.ai_settings.api_key,
                        stored.ai_settings.model_name,
                        stored.version,
                    ],
                )
                .map_err(Self::map_sqlite_error)?;
            Ok(profile)
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn find(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
    ) -> Result<Option<LanguageProfile>, LanguageProfileRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        let profile_id = profile_id.clone();
        tokio::task::spawn_blocking(move || {
            repository
                .lock_connection()?
                .query_row(
                    "SELECT id, user_id, name, source_language, target_language,
                            cards_per_set, answer_mode, mastery_threshold,
                            check_reading_if_possible, ai_provider, ai_api_key,
                            ai_model_name, version
                     FROM language_profiles
                     WHERE user_id = ?1 AND id = ?2",
                    params![user_id.as_str(), profile_id.as_str()],
                    Self::read_profile,
                )
                .optional()
                .map_err(Self::map_sqlite_error)
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn list(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<LanguageProfile>, LanguageProfileRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        tokio::task::spawn_blocking(move || {
            let connection = repository.lock_connection()?;
            let mut statement = connection
                .prepare(
                    "SELECT id, user_id, name, source_language, target_language,
                            cards_per_set, answer_mode, mastery_threshold,
                            check_reading_if_possible, ai_provider, ai_api_key,
                            ai_model_name, version
                     FROM language_profiles
                     WHERE user_id = ?1
                     ORDER BY name ASC",
                )
                .map_err(Self::map_sqlite_error)?;
            let profiles = statement
                .query_map(params![user_id.as_str()], Self::read_profile)
                .map_err(Self::map_sqlite_error)?
                .map(|profile| profile.map_err(Self::map_sqlite_error))
                .collect();

            profiles
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn update(
        &self,
        mut profile: LanguageProfile,
        expected_version: u64,
    ) -> Result<LanguageProfile, LanguageProfileRepositoryError> {
        let repository = self.clone();
        tokio::task::spawn_blocking(move || {
            let affected_rows = repository
                .lock_connection()?
                .execute(
                    "UPDATE language_profiles
                     SET name = ?1, source_language = ?2, target_language = ?3,
                         cards_per_set = ?4, answer_mode = ?5, mastery_threshold = ?6,
                         check_reading_if_possible = ?7,
                         ai_provider = ?8, ai_api_key = ?9, ai_model_name = ?10,
                         version = version + 1
                     WHERE user_id = ?11 AND id = ?12 AND version = ?13",
                    params![
                        profile.name,
                        profile.source_language,
                        profile.target_language,
                        profile.settings.cards_per_set,
                        Self::answer_mode_name(&profile.settings.answer_mode),
                        profile.settings.mastery_threshold,
                        profile.settings.check_reading_if_possible,
                        profile.ai_settings.provider,
                        profile.ai_settings.api_key,
                        profile.ai_settings.model_name,
                        profile.owner_id.as_str(),
                        profile.id.as_str(),
                        expected_version,
                    ],
                )
                .map_err(Self::map_sqlite_error)?;
            if affected_rows == 0 {
                return Err(LanguageProfileRepositoryError::Conflict);
            }

            profile.version = expected_version + 1;
            Ok(profile)
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn delete(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
    ) -> Result<bool, LanguageProfileRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        let profile_id = profile_id.clone();
        tokio::task::spawn_blocking(move || {
            repository
                .lock_connection()?
                .execute(
                    "DELETE FROM language_profiles WHERE user_id = ?1 AND id = ?2",
                    params![user_id.as_str(), profile_id.as_str()],
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
    use application::ports::{
        input::{language_profile::models::LearningSettings, local_user::models::LocalUser},
        output::repository::user::UserRepository,
    };
    use tempfile::TempDir;

    use crate::output::persistence::SqliteUserRepository;

    use super::*;

    #[tokio::test]
    async fn separates_users_and_persists_profiles_after_reopening() {
        let directory = TempDir::new().unwrap();
        let database_path = directory.path().join("users.db");
        let users = SqliteUserRepository::new(&database_path).unwrap();
        for username in ["alice", "bob"] {
            users
                .insert(LocalUser {
                    id: UserId::new(username),
                })
                .await
                .unwrap();
        }

        let repository = SqliteLanguageProfileRepository::new(&database_path).unwrap();
        let profile = LanguageProfile {
            id: ProfileId::new("profile-id"),
            owner_id: UserId::new("alice"),
            name: "Japanese".to_string(),
            source_language: "en-US".to_string(),
            target_language: "ja-JP".to_string(),
            settings: LearningSettings::default(),
            ai_settings: AiProviderSettings::default(),
            version: 0,
        };
        repository.insert(profile.clone()).await.unwrap();
        drop(repository);

        let reopened = SqliteLanguageProfileRepository::new(database_path).unwrap();
        assert_eq!(
            reopened.list(&UserId::new("alice")).await.unwrap(),
            vec![profile]
        );
        assert!(reopened.list(&UserId::new("bob")).await.unwrap().is_empty());
    }
}
