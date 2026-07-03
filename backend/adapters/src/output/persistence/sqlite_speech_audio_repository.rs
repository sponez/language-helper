use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use application::ports::{
    input::{
        card_catalog::models::CardId, card_speech::models::SpeechAudio,
        language_profile::models::ProfileId, local_user::models::UserId,
    },
    output::repository::speech_audio::{SpeechAudioRepository, models::SpeechAudioRepositoryError},
};
use async_trait::async_trait;
use rusqlite::{Connection, ErrorCode, OptionalExtension, params};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SqliteSpeechAudioRepositoryInitError {
    #[error("failed to create database directory {path:?}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to open speech audio database: {0}")]
    Open(#[source] rusqlite::Error),
    #[error("failed to initialize speech audio database: {0}")]
    Initialize(#[source] rusqlite::Error),
}

#[derive(Clone)]
pub struct SqliteSpeechAudioRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteSpeechAudioRepository {
    pub fn new(
        database_path: impl AsRef<Path>,
    ) -> Result<Self, SqliteSpeechAudioRepositoryInitError> {
        let database_path = database_path.as_ref();
        if let Some(parent) = database_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|source| {
                SqliteSpeechAudioRepositoryInitError::CreateDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }

        let connection =
            Connection::open(database_path).map_err(SqliteSpeechAudioRepositoryInitError::Open)?;
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS card_speech_audio (
                    card_id TEXT PRIMARY KEY NOT NULL,
                    fingerprint TEXT NOT NULL,
                    media_type TEXT NOT NULL,
                    audio BLOB NOT NULL,
                    generated_at INTEGER NOT NULL,
                    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
                );
                ",
            )
            .map_err(SqliteSpeechAudioRepositoryInitError::Initialize)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    fn lock_connection(&self) -> Result<MutexGuard<'_, Connection>, SpeechAudioRepositoryError> {
        self.connection
            .lock()
            .map_err(|_| SpeechAudioRepositoryError::Unavailable)
    }

    fn map_sqlite_error(error: rusqlite::Error) -> SpeechAudioRepositoryError {
        match &error {
            rusqlite::Error::SqliteFailure(details, _)
                if details.code == ErrorCode::ConstraintViolation =>
            {
                SpeechAudioRepositoryError::CardNotFound
            }
            rusqlite::Error::SqliteFailure(details, _)
                if matches!(
                    details.code,
                    ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
                ) =>
            {
                SpeechAudioRepositoryError::Unavailable
            }
            _ => SpeechAudioRepositoryError::Unexpected(error.to_string()),
        }
    }

    fn map_join_error(error: tokio::task::JoinError) -> SpeechAudioRepositoryError {
        SpeechAudioRepositoryError::Unexpected(format!(
            "speech audio repository task failed: {error}"
        ))
    }
}

#[async_trait]
impl SpeechAudioRepository for SqliteSpeechAudioRepository {
    async fn find(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        card_id: &CardId,
        fingerprint: &str,
    ) -> Result<Option<SpeechAudio>, SpeechAudioRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        let profile_id = profile_id.clone();
        let card_id = card_id.clone();
        let fingerprint = fingerprint.to_string();
        tokio::task::spawn_blocking(move || {
            repository
                .lock_connection()?
                .query_row(
                    "SELECT speech.media_type, speech.audio
                     FROM card_speech_audio speech
                     JOIN cards card ON card.id = speech.card_id
                     JOIN language_profiles profile ON profile.id = card.profile_id
                     WHERE profile.user_id = ?1
                       AND profile.id = ?2
                       AND card.id = ?3
                       AND speech.fingerprint = ?4",
                    params![
                        user_id.as_str(),
                        profile_id.as_str(),
                        card_id.as_str(),
                        fingerprint,
                    ],
                    |row| {
                        Ok(SpeechAudio {
                            media_type: row.get(0)?,
                            bytes: row.get(1)?,
                        })
                    },
                )
                .optional()
                .map_err(Self::map_sqlite_error)
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn upsert(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        card_id: &CardId,
        fingerprint: String,
        audio: SpeechAudio,
    ) -> Result<SpeechAudio, SpeechAudioRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        let profile_id = profile_id.clone();
        let card_id = card_id.clone();
        let stored = audio.clone();
        tokio::task::spawn_blocking(move || {
            let connection = repository.lock_connection()?;
            let exists = connection
                .query_row(
                    "SELECT EXISTS(
                        SELECT 1
                        FROM cards card
                        JOIN language_profiles profile ON profile.id = card.profile_id
                        WHERE profile.user_id = ?1 AND profile.id = ?2 AND card.id = ?3
                    )",
                    params![user_id.as_str(), profile_id.as_str(), card_id.as_str()],
                    |row| row.get::<_, bool>(0),
                )
                .map_err(Self::map_sqlite_error)?;
            if !exists {
                return Err(SpeechAudioRepositoryError::CardNotFound);
            }

            connection
                .execute(
                    "INSERT INTO card_speech_audio (
                        card_id, fingerprint, media_type, audio, generated_at
                     ) VALUES (?1, ?2, ?3, ?4, CAST(unixepoch('subsec') * 1000 AS INTEGER))
                     ON CONFLICT(card_id) DO UPDATE SET
                        fingerprint = excluded.fingerprint,
                        media_type = excluded.media_type,
                        audio = excluded.audio,
                        generated_at = excluded.generated_at",
                    params![
                        card_id.as_str(),
                        fingerprint,
                        stored.media_type,
                        stored.bytes,
                    ],
                )
                .map_err(Self::map_sqlite_error)?;
            Ok(audio)
        })
        .await
        .map_err(Self::map_join_error)?
    }
}

#[cfg(test)]
mod tests {
    use application::ports::{
        input::{
            card_catalog::models::{Card, CardDirection, CardId, Meaning, Word},
            card_speech::models::SpeechAudio,
            language_profile::models::{AiProviderSettings, LanguageProfile, ProfileId},
            local_user::models::{LocalUser, UserId},
        },
        output::repository::{
            CardRepository, LanguageProfileRepository, SpeechAudioRepository, UserRepository,
        },
    };
    use tempfile::TempDir;

    use crate::output::persistence::{
        SqliteCardRepository, SqliteLanguageProfileRepository, SqliteUserRepository,
    };

    use super::*;

    async fn setup() -> (
        TempDir,
        PathBuf,
        SqliteCardRepository,
        SqliteSpeechAudioRepository,
    ) {
        let directory = TempDir::new().unwrap();
        let database_path = directory.path().join("speech.db");
        SqliteUserRepository::new(&database_path)
            .unwrap()
            .insert(LocalUser {
                id: UserId::new("alice"),
            })
            .await
            .unwrap();
        SqliteLanguageProfileRepository::new(&database_path)
            .unwrap()
            .insert(LanguageProfile {
                id: ProfileId::new("profile"),
                owner_id: UserId::new("alice"),
                name: "Japanese".to_string(),
                source_language: "en-US".to_string(),
                target_language: "ja-JP".to_string(),
                ai_settings: AiProviderSettings::default(),
                version: 0,
            })
            .await
            .unwrap();
        let cards = SqliteCardRepository::new(&database_path).unwrap();
        cards
            .insert_batch(
                &UserId::new("alice"),
                &ProfileId::new("profile"),
                vec![Card {
                    id: CardId::new("card"),
                    profile_id: ProfileId::new("profile"),
                    direction: CardDirection::Straight,
                    word: Word {
                        text: "橋".to_string(),
                        readings: vec!["はし".to_string()],
                    },
                    meanings: vec![Meaning {
                        definition: "bridge".to_string(),
                        translated_definition: "мост".to_string(),
                        word_translations: vec!["мост".to_string()],
                        examples: vec![],
                    }],
                    score: 0,
                    created_at: 1,
                    version: 0,
                }],
            )
            .await
            .unwrap();
        let speech = SqliteSpeechAudioRepository::new(&database_path).unwrap();
        (directory, database_path, cards, speech)
    }

    #[tokio::test]
    async fn persists_replaces_and_cascades_cached_audio() {
        let (_directory, database_path, cards, speech) = setup().await;
        let owner = UserId::new("alice");
        let profile = ProfileId::new("profile");
        let card = CardId::new("card");
        let first = SpeechAudio {
            media_type: "audio/wav".to_string(),
            bytes: vec![1, 2, 3],
        };
        speech
            .upsert(&owner, &profile, &card, "first".to_string(), first)
            .await
            .unwrap();

        let reopened = SqliteSpeechAudioRepository::new(&database_path).unwrap();
        assert_eq!(
            reopened
                .find(&owner, &profile, &card, "first")
                .await
                .unwrap(),
            Some(SpeechAudio {
                media_type: "audio/wav".to_string(),
                bytes: vec![1, 2, 3],
            })
        );
        let second = SpeechAudio {
            media_type: "audio/wav".to_string(),
            bytes: vec![4, 5],
        };
        reopened
            .upsert(
                &owner,
                &profile,
                &card,
                "second".to_string(),
                second.clone(),
            )
            .await
            .unwrap();
        assert!(
            reopened
                .find(&owner, &profile, &card, "first")
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            reopened
                .find(&owner, &profile, &card, "second")
                .await
                .unwrap(),
            Some(second)
        );

        cards
            .delete_batch(&owner, &profile, &[card.clone()])
            .await
            .unwrap();
        assert!(
            reopened
                .find(&owner, &profile, &card, "second")
                .await
                .unwrap()
                .is_none()
        );
    }
}
