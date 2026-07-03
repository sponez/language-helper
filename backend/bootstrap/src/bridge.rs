use std::sync::Arc;

use adapters::output::GenAiCardNormalizer;
use adapters::output::persistence::{
    SqliteCardRepository, SqliteCardRepositoryInitError, SqliteLanguageProfileRepository,
    SqliteLanguageProfileRepositoryInitError, SqliteStudySessionRepository,
    SqliteStudySessionRepositoryInitError, SqliteUserRepository, SqliteUserRepositoryInitError,
};
use application::{
    ports::input::{
        card_catalog::CardCatalogUsecase, card_normalization::CardNormalizationUsecase,
        language_profile::LanguageProfileUsecase, local_user::LocalUserUsecase,
        study_session::StudySessionUsecase,
    },
    usecases::{
        CardCatalogService, CardNormalizationService, LanguageProfileService, LocalUserService,
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
}

/// Ready-to-use application ports shared by inbound adapters.
#[derive(Clone)]
pub struct BootstrapBridge {
    local_users: Arc<dyn LocalUserUsecase>,
    language_profiles: Arc<dyn LanguageProfileUsecase>,
    cards: Arc<dyn CardCatalogUsecase>,
    card_normalization: Arc<dyn CardNormalizationUsecase>,
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
        let local_users = Arc::new(LocalUserService::new(user_repository));
        let language_profiles = Arc::new(LanguageProfileService::new(Arc::clone(
            &language_profile_repository,
        )
            as Arc<dyn application::ports::output::repository::LanguageProfileRepository>));
        let cards = Arc::new(CardCatalogService::new(Arc::clone(&card_repository)
            as Arc<dyn application::ports::output::repository::CardRepository>));
        let study_sessions = Arc::new(StudySessionService::new(
            card_repository,
            study_session_repository,
        ));
        let card_normalization = Arc::new(CardNormalizationService::new(
            language_profile_repository,
            Arc::new(GenAiCardNormalizer),
        ));

        Ok(Self {
            local_users,
            language_profiles,
            cards,
            card_normalization,
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
