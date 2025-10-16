//! Card settings view model for GUI presentation.

use lh_api::models::card_settings::CardSettingsDto;

/// View model for displaying card settings in the GUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSettingsView {
    /// Number of cards shown per training set
    pub cards_per_set: u32,
    /// Method for testing: "manual" or "self_review"
    pub test_answer_method: String,
    /// Number of correct answers in a row to mark card as learned
    pub streak_length: u32,
}

impl CardSettingsView {
    /// Creates a new CardSettingsView.
    pub fn new(cards_per_set: u32, test_answer_method: String, streak_length: u32) -> Self {
        Self {
            cards_per_set,
            test_answer_method,
            streak_length,
        }
    }

    /// Creates a CardSettingsView from a DTO.
    pub fn from_dto(dto: CardSettingsDto) -> Self {
        Self {
            cards_per_set: dto.cards_per_set,
            test_answer_method: dto.test_answer_method,
            streak_length: dto.streak_length,
        }
    }

    /// Converts this view model to a DTO.
    pub fn to_dto(&self) -> CardSettingsDto {
        CardSettingsDto::new(
            self.cards_per_set,
            self.test_answer_method.clone(),
            self.streak_length,
        )
    }
}

impl Default for CardSettingsView {
    fn default() -> Self {
        Self {
            cards_per_set: 10,
            test_answer_method: "manual".to_string(),
            streak_length: 5,
        }
    }
}
