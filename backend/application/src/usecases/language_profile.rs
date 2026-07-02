use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::ports::{
    input::language_profile::{
        LanguageProfileUsecase,
        models::{
            CreateLanguageProfileCommand, DeleteLanguageProfileCommand, GetLanguageProfileQuery,
            LanguageProfile, LanguageProfileChanges, LanguageProfileError, LanguageProfileSummary,
            ListLanguageProfilesQuery, ProfileId, UpdateLanguageProfileCommand,
        },
    },
    output::repository::language_profile::{
        LanguageProfileRepository, models::LanguageProfileRepositoryError,
    },
};

const MAX_PROFILE_NAME_LENGTH: usize = 50;
const SUPPORTED_LANGUAGES: [&str; 3] = ["en-US", "ru-RU", "ja-JP"];
const SUPPORTED_AI_PROVIDERS: [&str; 2] = ["openai", "gemini"];

pub struct LanguageProfileService {
    repository: Arc<dyn LanguageProfileRepository>,
}

impl LanguageProfileService {
    pub fn new(repository: Arc<dyn LanguageProfileRepository>) -> Self {
        Self { repository }
    }

    fn normalize_name(name: &str) -> Result<String, LanguageProfileError> {
        let name = name.trim();
        if name.is_empty()
            || name.chars().count() > MAX_PROFILE_NAME_LENGTH
            || name.chars().any(char::is_control)
        {
            return Err(LanguageProfileError::InvalidProfile);
        }

        Ok(name.to_string())
    }

    fn validate_languages(source: &str, target: &str) -> Result<(), LanguageProfileError> {
        if source == target
            || !SUPPORTED_LANGUAGES.contains(&source)
            || !SUPPORTED_LANGUAGES.contains(&target)
        {
            return Err(LanguageProfileError::InvalidProfile);
        }

        Ok(())
    }

    fn map_repository_error(error: LanguageProfileRepositoryError) -> LanguageProfileError {
        match error {
            LanguageProfileRepositoryError::AlreadyExists => LanguageProfileError::AlreadyExists,
            LanguageProfileRepositoryError::Conflict => LanguageProfileError::Conflict,
            LanguageProfileRepositoryError::Unavailable => LanguageProfileError::Unexpected(
                "language profile repository is unavailable".to_string(),
            ),
            LanguageProfileRepositoryError::Unexpected(message) => {
                LanguageProfileError::Unexpected(message)
            }
        }
    }

    fn apply_changes(
        mut profile: LanguageProfile,
        changes: LanguageProfileChanges,
    ) -> Result<LanguageProfile, LanguageProfileError> {
        if let Some(name) = changes.name {
            profile.name = Self::normalize_name(&name)?;
        }
        if let Some(source_language) = changes.source_language {
            profile.source_language = source_language;
        }
        if let Some(target_language) = changes.target_language {
            profile.target_language = target_language;
        }
        if let Some(settings) = changes.settings {
            if settings.cards_per_set == 0
                || settings.cards_per_set > 100
                || settings.mastery_threshold == 0
                || settings.mastery_threshold > 50
            {
                return Err(LanguageProfileError::InvalidProfile);
            }
            profile.settings = settings;
        }
        if let Some(mut ai_settings) = changes.ai_settings {
            if ai_settings
                .provider
                .as_deref()
                .is_some_and(|provider| !SUPPORTED_AI_PROVIDERS.contains(&provider))
            {
                return Err(LanguageProfileError::InvalidProfile);
            }
            ai_settings.model_name = ai_settings
                .model_name
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            ai_settings.api_key = ai_settings.api_key.filter(|value| !value.is_empty());
            profile.ai_settings = ai_settings;
        }

        Self::validate_languages(&profile.source_language, &profile.target_language)?;
        Ok(profile)
    }
}

#[async_trait]
impl LanguageProfileUsecase for LanguageProfileService {
    async fn create_profile(
        &self,
        command: CreateLanguageProfileCommand,
    ) -> Result<LanguageProfile, LanguageProfileError> {
        let name = Self::normalize_name(&command.name)?;
        Self::validate_languages(&command.source_language, &command.target_language)?;

        if self
            .repository
            .list(&command.user_id)
            .await
            .map_err(Self::map_repository_error)?
            .iter()
            .any(|profile| profile.name == name)
        {
            return Err(LanguageProfileError::AlreadyExists);
        }

        self.repository
            .insert(LanguageProfile {
                id: ProfileId::new(Uuid::new_v4().to_string()),
                owner_id: command.user_id,
                name,
                source_language: command.source_language,
                target_language: command.target_language,
                settings: command.settings,
                ai_settings: Default::default(),
                version: 0,
            })
            .await
            .map_err(Self::map_repository_error)
    }

    async fn get_profile(
        &self,
        query: GetLanguageProfileQuery,
    ) -> Result<LanguageProfile, LanguageProfileError> {
        self.repository
            .find(&query.user_id, &query.profile_id)
            .await
            .map_err(Self::map_repository_error)?
            .ok_or(LanguageProfileError::NotFound)
    }

    async fn list_profiles(
        &self,
        query: ListLanguageProfilesQuery,
    ) -> Result<Vec<LanguageProfileSummary>, LanguageProfileError> {
        self.repository
            .list(&query.user_id)
            .await
            .map(|profiles| {
                profiles
                    .into_iter()
                    .map(|profile| LanguageProfileSummary {
                        id: profile.id,
                        name: profile.name,
                        source_language: profile.source_language,
                        target_language: profile.target_language,
                    })
                    .collect()
            })
            .map_err(Self::map_repository_error)
    }

    async fn update_profile(
        &self,
        command: UpdateLanguageProfileCommand,
    ) -> Result<LanguageProfile, LanguageProfileError> {
        let profile = self
            .repository
            .find(&command.user_id, &command.profile_id)
            .await
            .map_err(Self::map_repository_error)?
            .ok_or(LanguageProfileError::NotFound)?;
        let profile = Self::apply_changes(profile, command.changes)?;

        self.repository
            .update(profile, command.expected_version)
            .await
            .map_err(Self::map_repository_error)
    }

    async fn delete_profile(
        &self,
        command: DeleteLanguageProfileCommand,
    ) -> Result<bool, LanguageProfileError> {
        self.repository
            .delete(&command.user_id, &command.profile_id)
            .await
            .map_err(Self::map_repository_error)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use crate::ports::{
        input::{
            language_profile::models::{LearningSettings, ProfileId},
            local_user::models::UserId,
        },
        output::repository::language_profile::models::LanguageProfileRepositoryError,
    };

    use super::*;

    #[derive(Default)]
    struct InMemoryRepository {
        profiles: Mutex<HashMap<ProfileId, LanguageProfile>>,
    }

    #[async_trait]
    impl LanguageProfileRepository for InMemoryRepository {
        async fn insert(
            &self,
            profile: LanguageProfile,
        ) -> Result<LanguageProfile, LanguageProfileRepositoryError> {
            self.profiles
                .lock()
                .unwrap()
                .insert(profile.id.clone(), profile.clone());
            Ok(profile)
        }

        async fn find(
            &self,
            user_id: &UserId,
            profile_id: &ProfileId,
        ) -> Result<Option<LanguageProfile>, LanguageProfileRepositoryError> {
            Ok(self
                .profiles
                .lock()
                .unwrap()
                .get(profile_id)
                .filter(|profile| &profile.owner_id == user_id)
                .cloned())
        }

        async fn list(
            &self,
            user_id: &UserId,
        ) -> Result<Vec<LanguageProfile>, LanguageProfileRepositoryError> {
            Ok(self
                .profiles
                .lock()
                .unwrap()
                .values()
                .filter(|profile| &profile.owner_id == user_id)
                .cloned()
                .collect())
        }

        async fn update(
            &self,
            profile: LanguageProfile,
            _expected_version: u64,
        ) -> Result<LanguageProfile, LanguageProfileRepositoryError> {
            self.profiles
                .lock()
                .unwrap()
                .insert(profile.id.clone(), profile.clone());
            Ok(profile)
        }

        async fn delete(
            &self,
            user_id: &UserId,
            profile_id: &ProfileId,
        ) -> Result<bool, LanguageProfileRepositoryError> {
            let mut profiles = self.profiles.lock().unwrap();
            if profiles
                .get(profile_id)
                .is_some_and(|profile| &profile.owner_id == user_id)
            {
                Ok(profiles.remove(profile_id).is_some())
            } else {
                Ok(false)
            }
        }
    }

    fn command(user: &str, name: &str, source: &str, target: &str) -> CreateLanguageProfileCommand {
        CreateLanguageProfileCommand {
            user_id: UserId::new(user),
            name: name.to_string(),
            source_language: source.to_string(),
            target_language: target.to_string(),
            settings: LearningSettings::default(),
        }
    }

    #[tokio::test]
    async fn creates_a_normalized_profile_with_a_uuid() {
        let service = LanguageProfileService::new(Arc::new(InMemoryRepository::default()));
        let profile = service
            .create_profile(command("alice", " Japanese ", "en-US", "ja-JP"))
            .await
            .unwrap();

        assert_eq!(profile.name, "Japanese");
        assert!(Uuid::parse_str(profile.id.as_str()).is_ok());
    }

    #[tokio::test]
    async fn validates_language_pairs_and_names() {
        let service = LanguageProfileService::new(Arc::new(InMemoryRepository::default()));

        for invalid in [
            command("alice", "", "en-US", "ja-JP"),
            command("alice", "Same", "en-US", "en-US"),
            command("alice", "Unsupported", "de-DE", "en-US"),
        ] {
            assert_eq!(
                service.create_profile(invalid).await,
                Err(LanguageProfileError::InvalidProfile)
            );
        }
    }

    #[tokio::test]
    async fn names_are_unique_only_within_one_user() {
        let service = LanguageProfileService::new(Arc::new(InMemoryRepository::default()));
        service
            .create_profile(command("alice", "Japanese", "en-US", "ja-JP"))
            .await
            .unwrap();

        assert_eq!(
            service
                .create_profile(command("alice", "Japanese", "ru-RU", "ja-JP"))
                .await,
            Err(LanguageProfileError::AlreadyExists)
        );
        assert!(
            service
                .create_profile(command("bob", "Japanese", "ru-RU", "ja-JP"))
                .await
                .is_ok()
        );
    }
}
