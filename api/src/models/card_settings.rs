//! Card settings data transfer object.
//!
//! This module defines the DTO for card learning settings.

use serde::{Deserialize, Serialize};

/// Card settings data transfer object.
///
/// Used for transferring card settings between the API and consumers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardSettingsDto {
    /// Number of cards to show per learning set (1-100)
    pub cards_per_set: u32,
    /// Method for testing answers ("manual" or "self_review")
    pub test_answer_method: String,
    /// Number of consecutive correct answers needed to mark as "remembered" (1-50)
    pub streak_length: u32,
}

impl CardSettingsDto {
    /// Creates a new CardSettingsDto.
    pub fn new(
        cards_per_set: u32,
        test_answer_method: String,
        streak_length: u32,
    ) -> Self {
        Self {
            cards_per_set,
            test_answer_method,
            streak_length,
        }
    }
}

impl Default for CardSettingsDto {
    fn default() -> Self {
        Self {
            cards_per_set: 10,
            test_answer_method: "manual".to_string(),
            streak_length: 5,
        }
    }
}
