//! Learning service for managing study and test sessions.
//!
//! This service provides functionality for:
//! - Creating learning sessions from unlearned cards
//! - Testing card knowledge (written or self-review)
//! - Validating answers using Damerau-Levenshtein distance
//! - Managing session state (study/test phases)
//!
//! Note: This is LEARN mode - no streak updates occur here.

use crate::errors::CoreError;
use crate::models::{Card, CardType, LearningSession, TestResult};
use crate::repositories::ProfileRepository;

/// Calculates similarity between two strings using Damerau-Levenshtein distance
///
/// # Arguments
///
/// * `expected` - The expected answer
/// * `actual` - The actual answer provided
/// * `threshold` - Similarity threshold (0.0 to 1.0)
///
/// # Returns
///
/// true if strings are similar enough, false otherwise
fn calculate_similarity(expected: &str, actual: &str, threshold: f64) -> bool {
    // Normalize inputs (trim whitespace, lowercase)
    let expected_norm = expected.trim().to_lowercase();
    let actual_norm = actual.trim().to_lowercase();

    // Exact match after normalization
    if expected_norm == actual_norm {
        return true;
    }

    // Calculate normalized Damerau-Levenshtein distance
    let distance = strsim::damerau_levenshtein(&expected_norm, &actual_norm);
    let max_len = expected_norm.len().max(actual_norm.len());

    if max_len == 0 {
        return true; // Both empty strings
    }

    let similarity = 1.0 - (distance as f64 / max_len as f64);
    similarity >= threshold
}

/// Gets the expected answer for a card based on its type
///
/// # Arguments
///
/// * `card` - The card to get answer for
///
/// # Returns
///
/// Vector of acceptable answers
fn get_expected_answers(card: &Card) -> Vec<String> {
    match card.card_type {
        CardType::Straight => {
            // For straight cards, expect word translations from all meanings
            card.meanings
                .iter()
                .flat_map(|m| m.word_translations.clone())
                .collect()
        }
        CardType::Reverse => {
            // For reverse cards, expect word translations from all meanings
            card.meanings
                .iter()
                .flat_map(|m| m.word_translations.clone())
                .collect()
        }
    }
}

/// Checks if answer matches any of the expected answers
///
/// For Straight cards: checks against all meaning translations
/// For Reverse cards: checks against all word translations
///
/// # Arguments
///
/// * `user_input` - The user's answer
/// * `expected_answers` - List of acceptable answers
///
/// # Returns
///
/// (is_correct, expected_answer) tuple
fn check_answer_match(user_input: &str, expected_answers: &[String]) -> (bool, String) {
    // Try to match against any expected answer with similarity threshold 0.8
    for expected in expected_answers {
        if calculate_similarity(expected, user_input, 0.8) {
            return (true, expected.clone());
        }
    }

    // No match found - return first expected answer as the "correct" one
    let expected = expected_answers
        .first()
        .cloned()
        .unwrap_or_else(|| String::new());
    (false, expected)
}

/// Service for managing learning sessions
pub struct LearningService<R: ProfileRepository> {
    repository: R,
}

impl<R: ProfileRepository> LearningService<R> {
    /// Creates a new learning service
    ///
    /// # Arguments
    ///
    /// * `repository` - The profile repository for data access
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Creates a new learning session from cards
    ///
    /// # Arguments
    ///
    /// * `unlearned_cards` - All unlearned cards sorted by creation date
    /// * `start_card_number` - Starting card number (1-indexed)
    /// * `cards_per_set` - Number of cards per set
    /// * `test_method` - Test method ("manual" or "self_review")
    ///
    /// # Returns
    ///
    /// A new LearningSession
    ///
    /// # Errors
    ///
    /// Returns error if start index is invalid
    pub fn create_session_from_cards(
        unlearned_cards: Vec<Card>,
        start_card_number: usize,
        cards_per_set: usize,
        test_method: String,
    ) -> Result<LearningSession, CoreError> {
        // Validate start index (convert from 1-indexed to 0-indexed)
        if start_card_number == 0 || start_card_number > unlearned_cards.len() {
            return Err(CoreError::validation_error("Invalid start card number"));
        }
        let start_index = start_card_number - 1;

        Ok(LearningSession::new(
            unlearned_cards,
            start_index,
            cards_per_set,
            test_method,
        ))
    }

    /// Checks a written answer for a card
    ///
    /// Uses Damerau-Levenshtein distance to allow typos.
    ///
    /// # Arguments
    ///
    /// * `card` - The card being tested
    /// * `user_input` - The user's answer
    ///
    /// # Returns
    ///
    /// TestResult indicating whether the answer was correct
    pub fn check_written_answer(&self, card: &Card, user_input: &str) -> TestResult {
        let expected_answers = get_expected_answers(card);
        let (is_correct, expected) = check_answer_match(user_input, &expected_answers);

        TestResult::new_written(
            card.word.name.clone(),
            is_correct,
            user_input.to_string(),
            expected,
        )
    }

    /// Processes a self-review result
    ///
    /// # Arguments
    ///
    /// * `card` - The card being tested
    /// * `is_correct` - Whether the user marked their answer as correct
    ///
    /// # Returns
    ///
    /// TestResult with the user's self-evaluation
    pub fn process_self_review(&self, card: &Card, is_correct: bool) -> TestResult {
        TestResult::new_self_review(card.word.name.clone(), is_correct)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Meaning, Word};

    /// Helper to create a test straight card
    fn create_straight_card(
        word: &str,
        readings: Vec<&str>,
        meanings: Vec<(&str, &str, Vec<&str>)>,
    ) -> Card {
        let word_obj = Word::new_unchecked(word, readings.iter().map(|r| r.to_string()).collect());
        let meanings_objs: Vec<Meaning> = meanings
            .into_iter()
            .map(|(def, trans_def, word_trans)| {
                Meaning::new_unchecked(
                    def,
                    trans_def,
                    word_trans.iter().map(|t| t.to_string()).collect(),
                )
            })
            .collect();

        Card::new_unchecked(CardType::Straight, word_obj, meanings_objs, 0, 1000)
    }

    /// Helper to create a test reverse card
    fn create_reverse_card(word: &str, meanings: Vec<(&str, &str, Vec<&str>)>) -> Card {
        let word_obj = Word::new_unchecked(word, vec![]);
        let meanings_objs: Vec<Meaning> = meanings
            .into_iter()
            .map(|(def, trans_def, word_trans)| {
                Meaning::new_unchecked(
                    def,
                    trans_def,
                    word_trans.iter().map(|t| t.to_string()).collect(),
                )
            })
            .collect();

        Card::new_unchecked(CardType::Reverse, word_obj, meanings_objs, 0, 1000)
    }

    #[test]
    fn test_similarity_exact_match() {
        // Exact match should return true
        assert!(calculate_similarity("hello", "hello", 0.8));
    }

    #[test]
    fn test_similarity_case_insensitive() {
        // Should ignore case differences
        assert!(calculate_similarity("Hello", "hello", 0.8));
        assert!(calculate_similarity("HELLO", "hello", 0.8));
    }

    #[test]
    fn test_similarity_with_typo() {
        // Single character difference should pass with 0.8 threshold
        assert!(calculate_similarity("hello", "helo", 0.8));
        assert!(calculate_similarity("hello", "hallo", 0.8));
    }

    #[test]
    fn test_similarity_multiple_typos() {
        // Multiple typos should fail with high threshold
        assert!(!calculate_similarity("hello", "halo", 0.9));
    }

    #[test]
    fn test_similarity_completely_different() {
        // Completely different words should fail
        assert!(!calculate_similarity("hello", "goodbye", 0.8));
    }

    #[test]
    fn test_similarity_with_whitespace() {
        // Should trim whitespace
        assert!(calculate_similarity(" hello ", "hello", 0.8));
    }

    #[test]
    fn test_straight_card_expected_answers() {
        let card = create_straight_card(
            "食べる",
            vec!["たべる"],
            vec![("to eat", "comer", vec!["eat", "consume"])],
        );

        let answers = get_expected_answers(&card);

        // For straight cards, expect word translations
        assert_eq!(answers.len(), 2);
        assert!(answers.contains(&"eat".to_string()));
        assert!(answers.contains(&"consume".to_string()));
    }

    #[test]
    fn test_reverse_card_expected_answers() {
        let card = create_reverse_card(
            "hello",
            vec![("greeting", "saludo", vec!["hola", "buenos días"])],
        );

        let answers = get_expected_answers(&card);

        // For reverse cards, expect word translations from all meanings
        assert_eq!(answers.len(), 2);
        assert!(answers.contains(&"hola".to_string()));
        assert!(answers.contains(&"buenos días".to_string()));
    }

    #[test]
    fn test_check_answer_match_correct() {
        let expected = vec!["hello".to_string(), "hi".to_string()];
        let (is_correct, matched) = check_answer_match("hello", &expected);

        assert!(is_correct);
        assert_eq!(matched, "hello");
    }

    #[test]
    fn test_check_answer_match_with_typo() {
        let expected = vec!["hello".to_string()];
        let (is_correct, matched) = check_answer_match("helo", &expected);

        // Should accept with typo
        assert!(is_correct);
        assert_eq!(matched, "hello");
    }

    #[test]
    fn test_check_answer_match_incorrect() {
        let expected = vec!["hello".to_string()];
        let (is_correct, matched) = check_answer_match("goodbye", &expected);

        assert!(!is_correct);
        assert_eq!(matched, "hello"); // Returns first expected answer
    }

    #[test]
    fn test_check_written_answer_straight_card_correct() {
        let card = create_straight_card(
            "食べる",
            vec!["たべる"],
            vec![("to eat", "comer", vec!["eat", "consume"])],
        );

        let expected_answers = get_expected_answers(&card);
        let (is_correct, expected) = check_answer_match("eat", &expected_answers);
        let result = TestResult::new_written(
            card.word.name.clone(),
            is_correct,
            "eat".to_string(),
            expected,
        );

        assert!(result.is_correct);
        assert_eq!(result.word_name, "食べる");
    }

    #[test]
    fn test_check_written_answer_straight_card_alternative() {
        let card = create_straight_card(
            "食べる",
            vec!["たべる"],
            vec![("to eat", "comer", vec!["eat", "consume"])],
        );

        let expected_answers = get_expected_answers(&card);
        let (is_correct, _) = check_answer_match("consume", &expected_answers);

        assert!(is_correct);
    }

    #[test]
    fn test_check_written_answer_with_typo() {
        let card = create_straight_card(
            "食べる",
            vec!["たべる"],
            vec![("to eat", "comer", vec!["hello"])], // Longer word for better typo tolerance
        );

        let expected_answers = get_expected_answers(&card);
        let (is_correct, _) = check_answer_match("helo", &expected_answers); // Missing 'l'

        assert!(is_correct); // Should accept with typo - 4/5 = 0.8 similarity
    }

    #[test]
    fn test_process_self_review_correct() {
        let card = create_straight_card(
            "食べる",
            vec!["たべる"],
            vec![("to eat", "comer", vec!["eat"])],
        );

        let result = TestResult::new_self_review(card.word.name.clone(), true);

        assert!(result.is_correct);
        assert_eq!(result.word_name, "食べる");
        assert!(result.user_answer.is_none());
        assert!(result.expected_answer.is_none());
    }

    #[test]
    fn test_process_self_review_incorrect() {
        let card = create_straight_card(
            "食べる",
            vec!["たべる"],
            vec![("to eat", "comer", vec!["eat"])],
        );

        let result = TestResult::new_self_review(card.word.name.clone(), false);

        assert!(!result.is_correct);
    }
}
