//! ProfilesApi trait implementation.
//!
//! This module provides the concrete implementation of the ProfilesApi trait.

use async_trait::async_trait;
use lh_api::apis::profiles_api::ProfilesApi;
use lh_api::errors::api_error::ApiError;
use lh_api::models::assistant_settings::AssistantSettingsDto;
use lh_api::models::card::{CardDto, CardType as ApiCardType, MeaningDto, WordDto};
use lh_api::models::card_settings::CardSettingsDto;
use lh_api::models::learning_session::{LearningPhase as ApiLearningPhase, LearningSessionDto};
use lh_api::models::test_result::TestResultDto;

use crate::errors::CoreError;
use crate::models::{
    AssistantSettings, Card, CardSettings, CardType, LearningPhase, LearningSession, Meaning,
    TestResult, Word,
};
use crate::repositories::profile_repository::ProfileRepository;
use crate::services::learning_service::LearningService;
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

/// Convert domain LearningSession to DTO
fn learning_session_to_dto(session: LearningSession) -> LearningSessionDto {
    LearningSessionDto {
        all_cards: session.all_cards.into_iter().map(card_to_dto).collect(),
        current_set_start_index: session.current_set_start_index,
        cards_per_set: session.cards_per_set,
        phase: match session.phase {
            LearningPhase::Study => ApiLearningPhase::Study,
            LearningPhase::Test => ApiLearningPhase::Test,
        },
        current_card_in_set: session.current_card_in_set,
        test_method: session.test_method,
        test_results: session
            .test_results
            .into_iter()
            .map(test_result_to_dto)
            .collect(),
        current_card_provided_answers: session.current_card_provided_answers,
        current_card_failed: session.current_card_failed,
    }
}

/// Convert DTO to domain LearningSession
fn dto_to_learning_session(dto: LearningSessionDto) -> Result<LearningSession, ApiError> {
    let all_cards: Result<Vec<Card>, ApiError> =
        dto.all_cards.into_iter().map(dto_to_card).collect();

    Ok(LearningSession {
        all_cards: all_cards?,
        current_set_start_index: dto.current_set_start_index,
        cards_per_set: dto.cards_per_set,
        phase: match dto.phase {
            ApiLearningPhase::Study => LearningPhase::Study,
            ApiLearningPhase::Test => LearningPhase::Test,
        },
        current_card_in_set: dto.current_card_in_set,
        test_method: dto.test_method,
        test_results: dto
            .test_results
            .into_iter()
            .map(dto_to_test_result)
            .collect(),
        current_card_provided_answers: dto.current_card_provided_answers,
        current_card_failed: dto.current_card_failed,
    })
}

/// Convert domain TestResult to DTO
fn test_result_to_dto(result: TestResult) -> TestResultDto {
    TestResultDto {
        word_name: result.word_name,
        is_correct: result.is_correct,
        user_answer: result.user_answer,
        expected_answer: result.expected_answer,
    }
}

/// Convert DTO to domain TestResult
fn dto_to_test_result(dto: TestResultDto) -> TestResult {
    TestResult {
        word_name: dto.word_name,
        is_correct: dto.is_correct,
        user_answer: dto.user_answer,
        expected_answer: dto.expected_answer,
    }
}

/// Implementation of the ProfilesApi trait.
///
/// This struct handles profile database file operations.
pub struct ProfilesApiImpl<R: ProfileRepository> {
    profile_service: ProfileService<R>,
    learning_service: LearningService<R>,
}

impl<R: ProfileRepository> ProfilesApiImpl<R> {
    /// Creates a new ProfilesApiImpl instance.
    ///
    /// # Arguments
    ///
    /// * `profile_service` - The profile service for database file management
    /// * `learning_service` - The learning service for session management
    ///
    /// # Returns
    ///
    /// A new `ProfilesApiImpl` instance.
    pub fn new(profile_service: ProfileService<R>, learning_service: LearningService<R>) -> Self {
        Self {
            profile_service,
            learning_service,
        }
    }
}

#[async_trait]
impl<R: ProfileRepository> ProfilesApi for ProfilesApiImpl<R> {
    async fn create_profile_database(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<(), ApiError> {
        self.profile_service
            .create_profile_database(username, profile_name)
            .await
            .map(|_| ())
            .map_err(map_core_error_to_api_error)
    }

    async fn delete_profile_database(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<bool, ApiError> {
        self.profile_service
            .delete_profile_database(username, profile_name)
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
        profile_name: &str,
    ) -> Result<CardSettingsDto, ApiError> {
        let settings = self
            .profile_service
            .get_card_settings(username, profile_name)
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
        profile_name: &str,
        settings: CardSettingsDto,
    ) -> Result<(), ApiError> {
        // Convert DTO to domain model
        let domain_settings = CardSettings::new(
            settings.cards_per_set,
            settings.test_answer_method,
            settings.streak_length,
        );

        self.profile_service
            .update_card_settings(username, profile_name, domain_settings)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn get_assistant_settings(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<AssistantSettingsDto, ApiError> {
        let settings = self
            .profile_service
            .get_assistant_settings(username, profile_name)
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
        profile_name: &str,
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
            .update_assistant_settings(username, profile_name, domain_settings)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn clear_assistant_settings(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<(), ApiError> {
        self.profile_service
            .clear_assistant_settings(username, profile_name)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn save_card(
        &self,
        username: &str,
        profile_name: &str,
        card: CardDto,
    ) -> Result<(), ApiError> {
        let domain_card = dto_to_card(card)?;
        self.profile_service
            .save_card(username, profile_name, domain_card)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn get_all_cards(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<Vec<CardDto>, ApiError> {
        let cards = self
            .profile_service
            .get_all_cards(username, profile_name)
            .await
            .map_err(map_core_error_to_api_error)?;

        Ok(cards.into_iter().map(card_to_dto).collect())
    }

    async fn get_unlearned_cards(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<Vec<CardDto>, ApiError> {
        let cards = self
            .profile_service
            .get_unlearned_cards(username, profile_name)
            .await
            .map_err(map_core_error_to_api_error)?;

        Ok(cards.into_iter().map(card_to_dto).collect())
    }

    async fn get_learned_cards(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<Vec<CardDto>, ApiError> {
        let cards = self
            .profile_service
            .get_learned_cards(username, profile_name)
            .await
            .map_err(map_core_error_to_api_error)?;

        Ok(cards.into_iter().map(card_to_dto).collect())
    }

    async fn get_card_by_word_name(
        &self,
        username: &str,
        profile_name: &str,
        word_name: &str,
    ) -> Result<CardDto, ApiError> {
        let card = self
            .profile_service
            .get_card_by_word_name(username, profile_name, word_name)
            .await
            .map_err(map_core_error_to_api_error)?;

        Ok(card_to_dto(card))
    }

    async fn update_card_streak(
        &self,
        username: &str,
        profile_name: &str,
        word_name: &str,
        streak: i32,
    ) -> Result<(), ApiError> {
        self.profile_service
            .update_card_streak(username, profile_name, word_name, streak)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn delete_card(
        &self,
        username: &str,
        profile_name: &str,
        word_name: &str,
    ) -> Result<bool, ApiError> {
        self.profile_service
            .delete_card(username, profile_name, word_name)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn get_inverted_cards(
        &self,
        username: &str,
        profile_name: &str,
        card: CardDto,
    ) -> Result<Vec<CardDto>, ApiError> {
        // Convert DTO to domain Card
        let domain_card = dto_to_card(card)?;

        // Get inverted cards from service
        let inverted_cards = self
            .profile_service
            .get_inverted_cards(username, profile_name, &domain_card)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Convert domain Cards to DTOs
        Ok(inverted_cards.into_iter().map(card_to_dto).collect())
    }

    async fn create_learning_session(
        &self,
        username: &str,
        profile_name: &str,
        start_card_number: usize,
    ) -> Result<LearningSessionDto, ApiError> {
        // Get card settings to determine cards_per_set and test_method
        let card_settings = self
            .profile_service
            .get_card_settings(username, profile_name)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Get all unlearned cards
        let unlearned_cards = self
            .profile_service
            .get_unlearned_cards(username, profile_name)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Create session using learning service
        let session = LearningService::<R>::create_session_from_cards(
            unlearned_cards,
            start_card_number,
            card_settings.cards_per_set as usize,
            card_settings.test_answer_method,
        )
        .map_err(map_core_error_to_api_error)?;

        Ok(learning_session_to_dto(session))
    }

    async fn check_answer(
        &self,
        session: &LearningSessionDto,
        user_input: &str,
    ) -> Result<(bool, String), ApiError> {
        // Convert DTO to domain session
        let domain_session = dto_to_learning_session(session.clone())?;

        // Check answer using learning service
        let result = self
            .learning_service
            .check_answer_for_session(&domain_session, user_input);

        Ok(result)
    }

    async fn process_self_review(
        &self,
        username: &str,
        profile_name: &str,
        word_name: &str,
        is_correct: bool,
    ) -> Result<TestResultDto, ApiError> {
        // Get the card
        let card = self
            .profile_service
            .get_card_by_word_name(username, profile_name, word_name)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Process self-review
        let result = self.learning_service.process_self_review(&card, is_correct);

        Ok(test_result_to_dto(result))
    }

    async fn create_test_session(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<LearningSessionDto, ApiError> {
        // Get card settings for test_method
        let card_settings = self
            .profile_service
            .get_card_settings(username, profile_name)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Get all unlearned cards
        let unlearned_cards = self
            .profile_service
            .get_unlearned_cards(username, profile_name)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Create test session (shuffled, all cards)
        let session = LearningService::<R>::create_test_session(
            unlearned_cards,
            card_settings.test_answer_method,
        )
        .map_err(map_core_error_to_api_error)?;

        Ok(learning_session_to_dto(session))
    }

    async fn create_repeat_session(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<LearningSessionDto, ApiError> {
        // Get card settings for test_method
        let card_settings = self
            .profile_service
            .get_card_settings(username, profile_name)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Get all learned cards
        let learned_cards = self
            .profile_service
            .get_learned_cards(username, profile_name)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Create test session (shuffled, all cards)
        let session = LearningService::<R>::create_test_session(
            learned_cards,
            card_settings.test_answer_method,
        )
        .map_err(map_core_error_to_api_error)?;

        Ok(learning_session_to_dto(session))
    }

    async fn update_test_streaks(
        &self,
        username: &str,
        profile_name: &str,
        results: Vec<TestResultDto>,
    ) -> Result<(), ApiError> {
        // Convert DTOs to domain TestResults
        let domain_results: Vec<TestResult> = results.into_iter().map(dto_to_test_result).collect();

        // Process results with is_repeat_mode=false
        self.profile_service
            .process_test_results(username, profile_name, domain_results, false)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn update_repeat_streaks(
        &self,
        username: &str,
        profile_name: &str,
        results: Vec<TestResultDto>,
    ) -> Result<(), ApiError> {
        // Convert DTOs to domain TestResults
        let domain_results: Vec<TestResult> = results.into_iter().map(dto_to_test_result).collect();

        // Process results with is_repeat_mode=true
        self.profile_service
            .process_test_results(username, profile_name, domain_results, true)
            .await
            .map_err(map_core_error_to_api_error)
    }
}
