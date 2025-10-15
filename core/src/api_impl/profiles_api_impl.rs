//! ProfilesApi trait implementation.
//!
//! This module provides the concrete implementation of the ProfilesApi trait.

use async_trait::async_trait;
use lh_api::apis::profiles_api::ProfilesApi;
use lh_api::errors::api_error::ApiError;
use lh_api::models::assistant_settings::AssistantSettingsDto;
use lh_api::models::card::{CardDto, CardType as ApiCardType, MeaningDto, WordDto};
use lh_api::models::card_settings::CardSettingsDto;

use crate::errors::CoreError;
use crate::models::{AssistantSettings, Card, CardSettings, CardType, Meaning, Word};
use crate::repositories::profile_repository::ProfileRepository;
use crate::services::profile_service::ProfileService;

/// Helper function to map CoreError to ApiError
fn map_core_error_to_api_error(error: CoreError) -> ApiError {
    match error {
        CoreError::NotFound { entity, id } => {
            ApiError::not_found(format!("{} '{}' not found", entity, id))
        }
        CoreError::ValidationError { message } => ApiError::validation_error(message),
        CoreError::RepositoryError { message } => {
            ApiError::internal_error(format!("Internal error: {}", message))
        }
    }
}

/// Convert domain Card to DTO
fn card_to_dto(card: Card) -> CardDto {
    CardDto {
        card_type: match card.card_type {
            CardType::Straight => ApiCardType::Straight,
            CardType::Reverse => ApiCardType::Reverse,
        },
        word: WordDto {
            name: card.word.name,
            readings: card.word.readings,
        },
        meanings: card
            .meanings
            .into_iter()
            .map(|m| MeaningDto {
                definition: m.definition,
                translated_definition: m.translated_definition,
                word_translations: m.word_translations,
            })
            .collect(),
        streak: card.streak,
        created_at: card.created_at,
    }
}

/// Convert DTO to domain Card
fn dto_to_card(dto: CardDto) -> Result<Card, ApiError> {
    let card_type = match dto.card_type {
        ApiCardType::Straight => CardType::Straight,
        ApiCardType::Reverse => CardType::Reverse,
    };

    let word = Word::new(dto.word.name, dto.word.readings)
        .map_err(|e| ApiError::validation_error(e.to_string()))?;

    let meanings: Result<Vec<Meaning>, ApiError> = dto
        .meanings
        .into_iter()
        .map(|m| {
            Meaning::new(m.definition, m.translated_definition, m.word_translations)
                .map_err(|e| ApiError::validation_error(e.to_string()))
        })
        .collect();

    let meanings = meanings?;

    // If streak and created_at are provided (non-default values), use them
    // Otherwise create a new card with default values
    if dto.streak != 0 || dto.created_at != 0 {
        Ok(Card::new_unchecked(
            card_type,
            word,
            meanings,
            dto.streak,
            dto.created_at,
        ))
    } else {
        Card::new(card_type, word, meanings).map_err(|e| ApiError::validation_error(e.to_string()))
    }
}

/// Implementation of the ProfilesApi trait.
///
/// This struct handles profile database file operations.
pub struct ProfilesApiImpl<R: ProfileRepository> {
    profile_service: ProfileService<R>,
}

impl<R: ProfileRepository> ProfilesApiImpl<R> {
    /// Creates a new ProfilesApiImpl instance.
    ///
    /// # Arguments
    ///
    /// * `profile_service` - The profile service for database file management
    ///
    /// # Returns
    ///
    /// A new `ProfilesApiImpl` instance.
    pub fn new(profile_service: ProfileService<R>) -> Self {
        Self { profile_service }
    }
}

#[async_trait]
impl<R: ProfileRepository> ProfilesApi for ProfilesApiImpl<R> {
    async fn create_profile_database(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<(), ApiError> {
        self.profile_service
            .create_profile_database(username, target_language)
            .await
            .map(|_| ())
            .map_err(map_core_error_to_api_error)
    }

    async fn delete_profile_database(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<bool, ApiError> {
        self.profile_service
            .delete_profile_database(username, target_language)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn delete_user_folder(&self, username: &str) -> Result<bool, ApiError> {
        self.profile_service
            .delete_user_folder(username)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn get_card_settings(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<CardSettingsDto, ApiError> {
        let settings = self
            .profile_service
            .get_card_settings(username, target_language)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Convert domain model to DTO
        Ok(CardSettingsDto::new(
            settings.cards_per_set,
            settings.test_answer_method,
            settings.streak_length,
        ))
    }

    async fn update_card_settings(
        &self,
        username: &str,
        target_language: &str,
        settings: CardSettingsDto,
    ) -> Result<(), ApiError> {
        // Convert DTO to domain model
        let domain_settings = CardSettings::new(
            settings.cards_per_set,
            settings.test_answer_method,
            settings.streak_length,
        );

        self.profile_service
            .update_card_settings(username, target_language, domain_settings)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn get_assistant_settings(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<AssistantSettingsDto, ApiError> {
        let settings = self
            .profile_service
            .get_assistant_settings(username, target_language)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Convert domain model to DTO
        Ok(AssistantSettingsDto::new(
            settings.ai_model,
            settings.api_endpoint,
            settings.api_key,
            settings.api_model_name,
        ))
    }

    async fn update_assistant_settings(
        &self,
        username: &str,
        target_language: &str,
        settings: AssistantSettingsDto,
    ) -> Result<(), ApiError> {
        // Convert DTO to domain model
        let domain_settings = AssistantSettings::new(
            settings.ai_model,
            settings.api_endpoint,
            settings.api_key,
            settings.api_model_name,
        );

        self.profile_service
            .update_assistant_settings(username, target_language, domain_settings)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn clear_assistant_settings(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<(), ApiError> {
        self.profile_service
            .clear_assistant_settings(username, target_language)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn save_card(
        &self,
        username: &str,
        target_language: &str,
        card: CardDto,
    ) -> Result<(), ApiError> {
        let domain_card = dto_to_card(card)?;
        self.profile_service
            .save_card(username, target_language, domain_card)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn get_all_cards(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<Vec<CardDto>, ApiError> {
        let cards = self
            .profile_service
            .get_all_cards(username, target_language)
            .await
            .map_err(map_core_error_to_api_error)?;

        Ok(cards.into_iter().map(card_to_dto).collect())
    }

    async fn get_unlearned_cards(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<Vec<CardDto>, ApiError> {
        let cards = self
            .profile_service
            .get_unlearned_cards(username, target_language)
            .await
            .map_err(map_core_error_to_api_error)?;

        Ok(cards.into_iter().map(card_to_dto).collect())
    }

    async fn get_learned_cards(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<Vec<CardDto>, ApiError> {
        let cards = self
            .profile_service
            .get_learned_cards(username, target_language)
            .await
            .map_err(map_core_error_to_api_error)?;

        Ok(cards.into_iter().map(card_to_dto).collect())
    }

    async fn get_card_by_word_name(
        &self,
        username: &str,
        target_language: &str,
        word_name: &str,
    ) -> Result<CardDto, ApiError> {
        let card = self
            .profile_service
            .get_card_by_word_name(username, target_language, word_name)
            .await
            .map_err(map_core_error_to_api_error)?;

        Ok(card_to_dto(card))
    }

    async fn update_card_streak(
        &self,
        username: &str,
        target_language: &str,
        word_name: &str,
        streak: i32,
    ) -> Result<(), ApiError> {
        self.profile_service
            .update_card_streak(username, target_language, word_name, streak)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn delete_card(
        &self,
        username: &str,
        target_language: &str,
        word_name: &str,
    ) -> Result<bool, ApiError> {
        self.profile_service
            .delete_card(username, target_language, word_name)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn get_inverted_cards(
        &self,
        username: &str,
        target_language: &str,
        card: CardDto,
    ) -> Result<Vec<CardDto>, ApiError> {
        // Convert DTO to domain Card
        let domain_card = dto_to_card(card)?;

        // Get inverted cards from service
        let inverted_cards = self
            .profile_service
            .get_inverted_cards(username, target_language, &domain_card)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Convert domain Cards to DTOs
        Ok(inverted_cards.into_iter().map(card_to_dto).collect())
    }
}
