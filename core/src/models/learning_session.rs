//! Learning session model for managing a study and test session.
//!
//! This module defines the state of a learning session, including the current
//! set of cards being studied, the phase (study or test), and test results.

use super::card::Card;
use super::test_result::TestResult;

/// Phase of the learning session
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearningPhase {
    /// Studying cards (showing full information)
    Study,
    /// Testing cards (asking for answers)
    Test,
}

/// A learning session tracks progress through sets of cards
#[derive(Debug, Clone)]
pub struct LearningSession {
    /// All unlearned cards available for this session (sorted by created_at)
    pub all_cards: Vec<Card>,
    /// Index of the first card in the current set (within all_cards)
    pub current_set_start_index: usize,
    /// Number of cards per set
    pub cards_per_set: usize,
    /// Current phase (study or test)
    pub phase: LearningPhase,
    /// Index of current card within the current set (0 to cards_per_set-1)
    pub current_card_in_set: usize,
    /// Test method ("manual" for written, "self_review" for self-evaluation)
    pub test_method: String,
    /// Test results for the current set
    pub test_results: Vec<TestResult>,
}

impl LearningSession {
    /// Creates a new learning session
    ///
    /// # Arguments
    ///
    /// * `all_cards` - All unlearned cards sorted by creation date
    /// * `start_index` - Starting card index (0-based)
    /// * `cards_per_set` - Number of cards per learning set
    /// * `test_method` - Test method ("manual" or "self_review")
    pub fn new(
        all_cards: Vec<Card>,
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
        }
    }

    /// Gets the current set of cards being studied/tested
    pub fn current_set(&self) -> &[Card] {
        let end_index =
            (self.current_set_start_index + self.cards_per_set).min(self.all_cards.len());
        &self.all_cards[self.current_set_start_index..end_index]
    }

    /// Gets the current card being displayed or tested
    pub fn current_card(&self) -> Option<&Card> {
        self.current_set().get(self.current_card_in_set)
    }

    /// Advances to the next card in the current phase
    ///
    /// Returns true if advanced successfully, false if reached end of set
    pub fn advance_to_next_card(&mut self) -> bool {
        let set_size = self.current_set().len();
        if self.current_card_in_set + 1 < set_size {
            self.current_card_in_set += 1;
            true
        } else {
            false
        }
    }

    /// Moves from study phase to test phase
    pub fn start_test_phase(&mut self) {
        self.phase = LearningPhase::Test;
        self.current_card_in_set = 0;
        self.test_results.clear();
    }

    /// Checks if all cards in the current set have been studied
    pub fn is_study_complete(&self) -> bool {
        self.phase == LearningPhase::Study
            && self.current_card_in_set + 1 >= self.current_set().len()
    }

    /// Checks if all cards in the current set have been tested
    pub fn is_test_complete(&self) -> bool {
        self.phase == LearningPhase::Test && self.test_results.len() >= self.current_set().len()
    }

    /// Adds a test result for the current card
    pub fn add_test_result(&mut self, result: TestResult) {
        self.test_results.push(result);
    }

    /// Checks if the current set was passed (all answers correct)
    pub fn is_set_passed(&self) -> bool {
        !self.test_results.is_empty() && self.test_results.iter().all(|r| r.is_correct)
    }

    /// Moves to the next set after passing current set
    ///
    /// Returns true if there are more sets available, false if all cards completed
    pub fn advance_to_next_set(&mut self) -> bool {
        self.current_set_start_index += self.cards_per_set;

        if self.current_set_start_index < self.all_cards.len() {
            self.phase = LearningPhase::Study;
            self.current_card_in_set = 0;
            self.test_results.clear();
            true
        } else {
            false
        }
    }

    /// Retries the current set (resets to study phase)
    pub fn retry_current_set(&mut self) {
        self.phase = LearningPhase::Study;
        self.current_card_in_set = 0;
        self.test_results.clear();
    }

    /// Checks if there are more cards available for learning
    pub fn has_more_cards(&self) -> bool {
        self.current_set_start_index < self.all_cards.len()
    }

    /// Gets the total number of sets in this session
    pub fn total_sets(&self) -> usize {
        (self.all_cards.len() + self.cards_per_set - 1) / self.cards_per_set
    }

    /// Gets the current set number (1-indexed for display)
    pub fn current_set_number(&self) -> usize {
        (self.current_set_start_index / self.cards_per_set) + 1
    }
}
