use std::sync::Arc;

use adapters::output::persistence::{
    SqliteLanguageProfileRepository, SqliteLanguageProfileRepositoryInitError,
    SqliteUserRepository, SqliteUserRepositoryInitError,
};
use application::{
    ports::input::{language_profile::LanguageProfileUsecase, local_user::LocalUserUsecase},
    usecases::{LanguageProfileService, LocalUserService},
};
use thiserror::Error;

use crate::config::BootstrapConfig;

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("failed to initialize the local user repository: {0}")]
    UserRepository(#[from] SqliteUserRepositoryInitError),
    #[error("failed to initialize the language profile repository: {0}")]
    LanguageProfileRepository(#[from] SqliteLanguageProfileRepositoryInitError),
}

/// Ready-to-use application ports shared by inbound adapters.
#[derive(Clone)]
pub struct BootstrapBridge {
    local_users: Arc<dyn LocalUserUsecase>,
    language_profiles: Arc<dyn LanguageProfileUsecase>,
}

impl BootstrapBridge {
    pub fn create(config: BootstrapConfig) -> Result<Self, BootstrapError> {
        let user_repository = Arc::new(SqliteUserRepository::new(&config.database_path)?);
        let language_profile_repository =
            Arc::new(SqliteLanguageProfileRepository::new(&config.database_path)?);
        let local_users = Arc::new(LocalUserService::new(user_repository));
        let language_profiles = Arc::new(LanguageProfileService::new(language_profile_repository));

        Ok(Self {
            local_users,
            language_profiles,
        })
    }

    pub fn local_users(&self) -> Arc<dyn LocalUserUsecase> {
        Arc::clone(&self.local_users)
    }

    pub fn language_profiles(&self) -> Arc<dyn LanguageProfileUsecase> {
        Arc::clone(&self.language_profiles)
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
