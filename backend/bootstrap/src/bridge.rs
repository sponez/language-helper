use std::sync::Arc;

use adapters::output::persistence::{
    SqliteAiSettingsRepository, SqliteAiSettingsRepositoryInitError, SqliteCardRepository,
    SqliteCardRepositoryInitError, SqliteLanguageProfileRepository,
    SqliteLanguageProfileRepositoryInitError, SqlitePronunciationSettingsRepository,
    SqlitePronunciationSettingsRepositoryInitError, SqliteSpeechAudioRepository,
    SqliteSpeechAudioRepositoryInitError, SqliteStudySessionRepository,
    SqliteStudySessionRepositoryInitError, SqliteUserRepository, SqliteUserRepositoryInitError,
};
use adapters::output::{AiSpeechSynthesizer, AzurePronunciationAssessor, GenAiCardNormalizer};
use application::{
    ports::input::{
        ai_settings::AiSettingsUsecase, card_catalog::CardCatalogUsecase,
        card_normalization::CardNormalizationUsecase, card_speech::CardSpeechUsecase,
        language_profile::LanguageProfileUsecase, local_user::LocalUserUsecase,
        pronunciation_settings::PronunciationSettingsUsecase, study_session::StudySessionUsecase,
    },
    usecases::{
        AiSettingsService, CardCatalogService, CardNormalizationService, CardSpeechService,
        LanguageProfileService, LocalUserService, PronunciationSettingsService,
        StudySessionService,
    },
};
use thiserror::Error;

use crate::config::BootstrapConfig;

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("failed to initialize the local user repository: {0}")]
    UserRepository(#[from] SqliteUserRepositoryInitError),
    #[error("failed to initialize the language profile repository: {0}")]
    LanguageProfileRepository(#[from] SqliteLanguageProfileRepositoryInitError),
    #[error("failed to initialize the card repository: {0}")]
    CardRepository(#[from] SqliteCardRepositoryInitError),
    #[error("failed to initialize the study session repository: {0}")]
    StudySessionRepository(#[from] SqliteStudySessionRepositoryInitError),
    #[error("failed to initialize the speech audio repository: {0}")]
    SpeechAudioRepository(#[from] SqliteSpeechAudioRepositoryInitError),
    #[error("failed to initialize the pronunciation settings repository: {0}")]
    PronunciationSettingsRepository(#[from] SqlitePronunciationSettingsRepositoryInitError),
    #[error("failed to initialize the AI settings repository: {0}")]
    AiSettingsRepository(#[from] SqliteAiSettingsRepositoryInitError),
}

/// Ready-to-use application ports shared by inbound adapters.
#[derive(Clone)]
pub struct BootstrapBridge {
    local_users: Arc<dyn LocalUserUsecase>,
    language_profiles: Arc<dyn LanguageProfileUsecase>,
    cards: Arc<dyn CardCatalogUsecase>,
    card_normalization: Arc<dyn CardNormalizationUsecase>,
    card_speech: Arc<dyn CardSpeechUsecase>,
    ai_settings: Arc<dyn AiSettingsUsecase>,
    pronunciation_settings: Arc<dyn PronunciationSettingsUsecase>,
    study_sessions: Arc<dyn StudySessionUsecase>,
}

impl BootstrapBridge {
    pub fn create(config: BootstrapConfig) -> Result<Self, BootstrapError> {
        let user_repository = Arc::new(SqliteUserRepository::new(&config.database_path)?);
        let language_profile_repository =
            Arc::new(SqliteLanguageProfileRepository::new(&config.database_path)?);
        let card_repository = Arc::new(SqliteCardRepository::new(&config.database_path)?);
        let study_session_repository =
            Arc::new(SqliteStudySessionRepository::new(&config.database_path)?);
        let speech_audio_repository =
            Arc::new(SqliteSpeechAudioRepository::new(&config.database_path)?);
        let pronunciation_settings_repository = Arc::new(
            SqlitePronunciationSettingsRepository::new(&config.database_path)?,
        );
        let ai_settings_repository =
            Arc::new(SqliteAiSettingsRepository::new(&config.database_path)?);
        let local_users = Arc::new(LocalUserService::new(user_repository));
        let language_profiles = Arc::new(LanguageProfileService::new(Arc::clone(
            &language_profile_repository,
        )
            as Arc<dyn application::ports::output::repository::LanguageProfileRepository>));
        let cards = Arc::new(CardCatalogService::new(Arc::clone(&card_repository)
            as Arc<dyn application::ports::output::repository::CardRepository>));
        let study_sessions = Arc::new(StudySessionService::new(
            Arc::clone(&card_repository)
                as Arc<dyn application::ports::output::repository::CardRepository>,
            study_session_repository,
            Arc::clone(&language_profile_repository)
                as Arc<dyn application::ports::output::repository::LanguageProfileRepository>,
            Arc::clone(&pronunciation_settings_repository)
                as Arc<dyn application::ports::output::repository::PronunciationSettingsRepository>,
            Arc::new(AzurePronunciationAssessor::default()),
        ));
        let pronunciation_settings = Arc::new(PronunciationSettingsService::new(
            pronunciation_settings_repository,
        ));
        let ai_settings = Arc::new(AiSettingsService::new(Arc::clone(&ai_settings_repository)
            as Arc<dyn application::ports::output::repository::AiSettingsRepository>));
        let card_normalization = Arc::new(CardNormalizationService::new(
            Arc::clone(&language_profile_repository)
                as Arc<dyn application::ports::output::repository::LanguageProfileRepository>,
            Arc::clone(&ai_settings_repository)
                as Arc<dyn application::ports::output::repository::AiSettingsRepository>,
            Arc::new(GenAiCardNormalizer),
        ));
        let card_speech = Arc::new(CardSpeechService::new(
            language_profile_repository,
            ai_settings_repository,
            card_repository,
            speech_audio_repository,
            Arc::new(AiSpeechSynthesizer::default()),
        ));

        Ok(Self {
            local_users,
            language_profiles,
            cards,
            card_normalization,
            card_speech,
            ai_settings,
            pronunciation_settings,
            study_sessions,
        })
    }

    pub fn local_users(&self) -> Arc<dyn LocalUserUsecase> {
        Arc::clone(&self.local_users)
    }

    pub fn language_profiles(&self) -> Arc<dyn LanguageProfileUsecase> {
        Arc::clone(&self.language_profiles)
    }

    pub fn cards(&self) -> Arc<dyn CardCatalogUsecase> {
        Arc::clone(&self.cards)
    }

    pub fn card_normalization(&self) -> Arc<dyn CardNormalizationUsecase> {
        Arc::clone(&self.card_normalization)
    }

    pub fn card_speech(&self) -> Arc<dyn CardSpeechUsecase> {
        Arc::clone(&self.card_speech)
    }

    pub fn ai_settings(&self) -> Arc<dyn AiSettingsUsecase> {
        Arc::clone(&self.ai_settings)
    }

    pub fn pronunciation_settings(&self) -> Arc<dyn PronunciationSettingsUsecase> {
        Arc::clone(&self.pronunciation_settings)
    }

    pub fn study_sessions(&self) -> Arc<dyn StudySessionUsecase> {
        Arc::clone(&self.study_sessions)
    }
}

#[cfg(test)]
mod tests {
    use application::ports::input::local_user::models::CreateLocalUserCommand;
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn creates_a_user_through_the_bootstrapped_application() {
        let directory = TempDir::new().unwrap();
        let bridge =
            BootstrapBridge::create(BootstrapConfig::new(directory.path().join("users.db")))
                .unwrap();

        let user = bridge
            .local_users()
            .create_user(CreateLocalUserCommand {
                username: "bootstrap_user".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(user.id.as_str(), "bootstrap_user");
        assert_eq!(bridge.local_users().list_users().await.unwrap().len(), 1);
    }
}
