use std::sync::Arc;

use application::ports::input::{
    card_catalog::CardCatalogUsecase, card_normalization::CardNormalizationUsecase,
    language_profile::LanguageProfileUsecase, local_user::LocalUserUsecase,
    study_session::StudySessionUsecase,
};
use lh_bootstrap::BootstrapBridge;

pub struct DesktopState {
    local_users: Arc<dyn LocalUserUsecase>,
    language_profiles: Arc<dyn LanguageProfileUsecase>,
    cards: Arc<dyn CardCatalogUsecase>,
    card_normalization: Arc<dyn CardNormalizationUsecase>,
    study_sessions: Arc<dyn StudySessionUsecase>,
}

impl DesktopState {
    pub fn new(bridge: BootstrapBridge) -> Self {
        Self {
            local_users: bridge.local_users(),
            language_profiles: bridge.language_profiles(),
            cards: bridge.cards(),
            card_normalization: bridge.card_normalization(),
            study_sessions: bridge.study_sessions(),
        }
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
