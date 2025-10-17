//! Card settings state for a profile.
//!
//! This module provides state for card learning settings within a learning profile.

/// Card settings for a learning profile
#[derive(Debug, Clone)]
pub struct CardState {
    /// Number of cards to show per learning set (1-100)
    pub cards_per_set: u32,
    /// Method for testing answers ("manual" or "self_review")
    pub test_answer_method: String,
    /// Number of consecutive correct answers needed to mark as "remembered" (1-50)
    pub streak_length: u32,
}

impl CardState {
    /// Creates a new CardState with the given settings.
    ///
    /// # Arguments
    ///
    /// * `cards_per_set` - Number of cards per learning set
    /// * `test_answer_method` - Method for testing answers
    /// * `streak_length` - Streak length needed to mark as remembered
    ///
    /// # Returns
    ///
    /// A new CardState instance
    pub fn new(cards_per_set: u32, test_answer_method: String, streak_length: u32) -> Self {
        Self {
            cards_per_set,
            test_answer_method,
            streak_length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_state_new() {
        let state = CardState::new(10, "manual".to_string(), 5);

        assert_eq!(state.cards_per_set, 10);
        assert_eq!(state.test_answer_method, "manual");
        assert_eq!(state.streak_length, 5);
    }

    #[test]
    fn test_card_state_with_self_review() {
        let state = CardState::new(20, "self_review".to_string(), 10);

        assert_eq!(state.cards_per_set, 20);
        assert_eq!(state.test_answer_method, "self_review");
        assert_eq!(state.streak_length, 10);
    }

    #[test]
    fn test_card_state_clone() {
        let state = CardState::new(15, "manual".to_string(), 7);
        let cloned = state.clone();

        assert_eq!(state.cards_per_set, cloned.cards_per_set);
        assert_eq!(state.test_answer_method, cloned.test_answer_method);
        assert_eq!(state.streak_length, cloned.streak_length);
    }

    #[test]
    fn test_card_state_debug() {
        let state = CardState::new(10, "manual".to_string(), 5);
        let debug_str = format!("{:?}", state);

        assert!(debug_str.contains("CardState"));
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("manual"));
        assert!(debug_str.contains("5"));
    }
}
