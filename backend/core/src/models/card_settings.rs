//! Card settings domain model.
//!
//! This module defines the card learning settings for a profile,
//! including cards per set, test answer method, and streak length.

/// Card settings for a learning profile.
///
/// These settings control how vocabulary cards are presented and tested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSettings {
    /// Number of cards to show per learning set
    pub cards_per_set: u32,
    /// Method for testing answers ("manual" or "self_review")
    pub test_answer_method: String,
    /// Number of consecutive correct answers needed to mark as "remembered"
    pub streak_length: u32,
}

impl CardSettings {
    /// Creates a new CardSettings instance.
    ///
    /// # Arguments
    ///
    /// * `cards_per_set` - Number of cards per set (1-100)
    /// * `test_answer_method` - Method for testing ("manual" or "self_review")
    /// * `streak_length` - Streak length for remembering (1-50)
    pub fn new(cards_per_set: u32, test_answer_method: String, streak_length: u32) -> Self {
        Self {
            cards_per_set,
            test_answer_method,
            streak_length,
        }
    }
}

impl Default for CardSettings {
    fn default() -> Self {
        Self {
            cards_per_set: 10,
            test_answer_method: "manual".to_string(),
            streak_length: 5,
        }
    }
}
