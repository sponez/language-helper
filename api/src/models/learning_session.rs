//! Learning session data transfer objects.
//!
//! This module defines DTOs for managing learning sessions.

use crate::models::card::CardDto;
use crate::models::test_result::TestResultDto;
use serde::{Deserialize, Serialize};

/// Phase of the learning session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LearningPhase {
    /// Studying cards (showing full information).
    Study,
    /// Testing cards (asking for answers).
    Test,
}

/// Data transfer object for a learning session.
///
/// Tracks the state of a learning session including which cards
/// are being studied, the current phase, and test results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningSessionDto {
    /// All unlearned cards available for this session (sorted by created_at).
    pub all_cards: Vec<CardDto>,
    /// Index of the first card in the current set (within all_cards).
    pub current_set_start_index: usize,
    /// Number of cards per set.
    pub cards_per_set: usize,
    /// Current phase (study or test).
    pub phase: LearningPhase,
    /// Index of current card within the current set (0 to cards_per_set-1).
    pub current_card_in_set: usize,
    /// Test method ("manual" for written, "self_review" for self-evaluation).
    pub test_method: String,
    /// Test results for the current set.
    pub test_results: Vec<TestResultDto>,
    /// Answers already provided for current card (for multi-answer testing).
    pub current_card_provided_answers: Vec<String>,
    /// Whether the current card has been answered incorrectly.
    pub current_card_failed: bool,
}

impl LearningSessionDto {
    /// Creates a new learning session DTO.
    pub fn new(
        all_cards: Vec<CardDto>,
        start_index: usize,
        cards_per_set: usize,
        test_method: String,
    ) -> Self {
        Self {
            all_cards,
            current_set_start_index: start_index,
            cards_per_set,
            phase: LearningPhase::Study,
            current_card_in_set: 0,
            test_method,
            test_results: Vec::new(),
            current_card_provided_answers: Vec::new(),
            current_card_failed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::card::{CardType, MeaningDto, WordDto};

    #[test]
    fn test_learning_phase_serialization() {
        let study = LearningPhase::Study;
        let json = serde_json::to_string(&study).unwrap();
        assert_eq!(json, r#""study""#);

        let test = LearningPhase::Test;
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r#""test""#);
    }

    #[test]
    fn test_learning_session_creation() {
        let card = CardDto {
            card_type: CardType::Straight,
            word: WordDto {
                name: "hello".to_string(),
                readings: vec![],
            },
            meanings: vec![MeaningDto {
                definition: "greeting".to_string(),
                translated_definition: "saludo".to_string(),
                word_translations: vec!["hola".to_string()],
            }],
            streak: 0,
            created_at: 1000,
        };

        let session = LearningSessionDto::new(vec![card], 0, 5, "manual".to_string());

        assert_eq!(session.all_cards.len(), 1);
        assert_eq!(session.current_set_start_index, 0);
        assert_eq!(session.cards_per_set, 5);
        assert_eq!(session.phase, LearningPhase::Study);
        assert_eq!(session.test_method, "manual");
    }
}
