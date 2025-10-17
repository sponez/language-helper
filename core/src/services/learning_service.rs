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
use crate::models::{Card, LearningSession, TestResult};
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

/// Gets ALL expected answers for a card based on its type
///
/// # Arguments
///
/// * `card` - The card to get answer for
///
/// # Returns
///
/// Vector of all acceptable answers
fn get_all_expected_answers(card: &Card) -> Vec<String> {
    // For both card types, all word translations are acceptable
    card.meanings
        .iter()
        .flat_map(|m| m.word_translations.clone())
        .collect()
}

/// Gets the expected answers for the NEXT answer for a card
///
/// For Straight cards: returns all translations (user can hit any meaning)
/// For Reverse cards: returns only translations that haven't been provided yet
///
/// # Arguments
///
/// * `card` - The card being tested
/// * `provided_answers` - Answers already provided (normalized)
///
/// # Returns
///
/// Vector of acceptable answers for the next input
fn get_next_expected_answers(card: &Card, provided_answers: &[String]) -> Vec<String> {
    use crate::models::CardType;

    let all_translations: Vec<String> = card
        .meanings
        .iter()
        .flat_map(|m| m.word_translations.clone())
        .collect();

    match card.card_type {
        CardType::Straight => {
            // For Straight cards, all translations are always acceptable
            // (multiple hits on same meaning are fine)
            all_translations
        }
        CardType::Reverse => {
            // For Reverse cards, return only translations not yet provided
            all_translations
                .into_iter()
                .filter(|trans| {
                    // Check if this translation hasn't been provided yet (case-insensitive)
                    !provided_answers
                        .iter()
                        .any(|provided| provided.to_lowercase() == trans.to_lowercase())
                })
                .collect()
        }
    }
}

/// Validates if all provided answers are correct for a card
///
/// For Straight cards:
/// - Checks if all meanings are covered (at least one correct translation per meaning)
/// - Returns false if any answer doesn't match any translation
/// - Multiple hits on the same meaning are allowed
///
/// For Reverse cards:
/// - Checks if all translations have been provided
/// - Returns false if any answer doesn't match or if any translation is missing
///
/// # Arguments
///
/// * `card` - The card being tested
/// * `provided_answers` - All answers provided by the user
///
/// # Returns
///
/// true if all requirements are met, false otherwise
#[allow(dead_code)] // Used in tests and will be used by GUI layer
fn validate_all_answers(card: &Card, provided_answers: &[String]) -> bool {
    use crate::models::CardType;

    let all_translations: Vec<String> = card
        .meanings
        .iter()
        .flat_map(|m| m.word_translations.clone())
        .collect();

    // First check: all provided answers must match some translation
    for provided in provided_answers {
        let matches_any = all_translations
            .iter()
            .any(|trans| calculate_similarity(trans, provided, 0.8));
        if !matches_any {
            return false; // Non-existent translation provided
        }
    }

    match card.card_type {
        CardType::Straight => {
            // For Straight cards: check if all meanings are covered
            // For each meaning, check if at least one provided answer matches any of its translations
            card.meanings.iter().all(|meaning| {
                meaning.word_translations.iter().any(|trans| {
                    provided_answers
                        .iter()
                        .any(|provided| calculate_similarity(trans, provided, 0.8))
                })
            })
        }
        CardType::Reverse => {
            // For Reverse cards: check if all translations are provided
            all_translations.iter().all(|trans| {
                provided_answers
                    .iter()
                    .any(|provided| calculate_similarity(trans, provided, 0.8))
            })
        }
    }
}

/// Checks if answer matches any of the expected answers
///
/// Uses Damerau-Levenshtein distance with 0.8 threshold to allow typos
///
/// # Arguments
///
/// * `user_input` - The user's answer
/// * `expected_answers` - List of acceptable answers
///
/// # Returns
///
/// (is_correct, matched_answer) tuple where matched_answer is the expected answer that matched,
/// or the first expected answer if no match was found
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

    /// Creates a new learning session from cards with cyclic shift
    ///
    /// Implements cyclic card iteration:
    /// - If user inputs k (1-indexed) where k <= n, cards are arranged from k to n, then 1 to k-1
    /// - If k > n, then k_actual = ((k-1) mod n) + 1 and the same logic applies
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
    /// A new LearningSession with cyclically shifted cards
    ///
    /// # Errors
    ///
    /// Returns error if there are no cards or invalid parameters
    pub fn create_session_from_cards(
        unlearned_cards: Vec<Card>,
        start_card_number: usize,
        cards_per_set: usize,
        test_method: String,
    ) -> Result<LearningSession, CoreError> {
        // Validate we have cards
        if unlearned_cards.is_empty() {
            return Err(CoreError::validation_error("No unlearned cards available"));
        }

        // Validate start card number
        if start_card_number == 0 {
            return Err(CoreError::validation_error(
                "Start card number must be at least 1",
            ));
        }

        let n = unlearned_cards.len();

        // Apply modulo if k > n: k_actual = ((k-1) mod n) + 1
        let actual_start_number = if start_card_number > n {
            ((start_card_number - 1) % n) + 1
        } else {
            start_card_number
        };

        // Convert to 0-indexed
        let start_index = actual_start_number - 1;

        // Perform cyclic shift: cards from start_index to end, then from 0 to start_index-1
        let mut shifted_cards = Vec::with_capacity(n);
        shifted_cards.extend_from_slice(&unlearned_cards[start_index..]);
        shifted_cards.extend_from_slice(&unlearned_cards[..start_index]);

        // Create session starting from index 0 (since we already shifted the cards)
        Ok(LearningSession::new(
            shifted_cards,
            0,
            cards_per_set,
            test_method,
        ))
    }

    /// Checks a written answer against the session's current card state
    ///
    /// Uses Damerau-Levenshtein distance to allow typos.
    /// Takes into account which answers have already been provided.
    ///
    /// # Arguments
    ///
    /// * `session` - The current learning session
    /// * `user_input` - The user's answer
    ///
    /// # Returns
    ///
    /// (is_correct, matched_answer) tuple
    pub fn check_answer_for_session(
        &self,
        session: &LearningSession,
        user_input: &str,
    ) -> (bool, String) {
        if let Some(card) = session.current_card() {
            let expected_answers =
                get_next_expected_answers(card, &session.current_card_provided_answers);
            check_answer_match(user_input, &expected_answers)
        } else {
            (false, String::new())
        }
    }

    /// Checks a written answer for a card (legacy method for tests)
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
        let expected_answers = get_all_expected_answers(card);
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
    use crate::models::{CardType, Meaning, Word};

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

        let answers = get_all_expected_answers(&card);

        // All word translations should be returned
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

        let answers = get_all_expected_answers(&card);

        // All word translations should be returned
        assert_eq!(answers.len(), 2);
        assert!(answers.contains(&"hola".to_string()));
        assert!(answers.contains(&"buenos días".to_string()));
    }

    #[test]
    fn test_straight_card_always_accepts_all_translations() {
        let card = create_straight_card(
            "食べる",
            vec!["たべる"],
            vec![
                ("to eat", "comer", vec!["eat", "consume"]),
                ("to have a meal", "tomar una comida", vec!["dine"]),
            ],
        );

        // For Straight cards, all translations are always acceptable
        let next = get_next_expected_answers(&card, &[]);
        assert_eq!(next.len(), 3);
        assert!(next.contains(&"eat".to_string()));
        assert!(next.contains(&"consume".to_string()));
        assert!(next.contains(&"dine".to_string()));

        // Even after providing "dine", all translations are still acceptable
        let next = get_next_expected_answers(&card, &["dine".to_string()]);
        assert_eq!(next.len(), 3);
        assert!(next.contains(&"eat".to_string()));
        assert!(next.contains(&"consume".to_string()));
        assert!(next.contains(&"dine".to_string()));
    }

    #[test]
    fn test_straight_card_validation_one_per_meaning() {
        let card = create_straight_card(
            "ключ",
            vec![],
            vec![
                ("spring (water source)", "fuente", vec!["spring", "source"]),
                ("key (for lock)", "llave", vec!["key"]),
            ],
        );

        // Valid: one hit per meaning
        assert!(validate_all_answers(
            &card,
            &["spring".to_string(), "key".to_string()]
        ));

        // Valid: multiple hits on same meaning are OK
        assert!(validate_all_answers(
            &card,
            &[
                "spring".to_string(),
                "source".to_string(),
                "key".to_string()
            ]
        ));

        // Invalid: missing second meaning
        assert!(!validate_all_answers(&card, &["spring".to_string()]));

        // Invalid: non-existent translation
        assert!(!validate_all_answers(
            &card,
            &["spring".to_string(), "door".to_string()]
        ));
    }

    #[test]
    fn test_reverse_card_next_answer_removes_provided() {
        let card = create_reverse_card(
            "hello",
            vec![
                ("greeting", "saludo", vec!["hola", "buenos días"]),
                ("informal greeting", "saludo informal", vec!["hola"]),
            ],
        );

        // First call should return all unique translations
        let next = get_next_expected_answers(&card, &[]);
        assert!(next.len() >= 2); // "hola" and "buenos días"

        // After providing "hola", should not include it
        let next = get_next_expected_answers(&card, &["hola".to_string()]);
        assert!(!next.iter().any(|a| a.to_lowercase() == "hola"));
        assert!(next.contains(&"buenos días".to_string()));
    }

    #[test]
    fn test_reverse_card_validation_all_required() {
        let card = create_reverse_card(
            "spring",
            vec![
                ("water source", "fuente de agua", vec!["ключ", "источник"]),
                ("season", "temporada", vec!["весна"]),
            ],
        );

        // Valid: all translations provided
        assert!(validate_all_answers(
            &card,
            &[
                "ключ".to_string(),
                "источник".to_string(),
                "весна".to_string()
            ]
        ));

        // Invalid: missing "весна"
        assert!(!validate_all_answers(
            &card,
            &["ключ".to_string(), "источник".to_string()]
        ));

        // Invalid: non-existent translation
        assert!(!validate_all_answers(
            &card,
            &[
                "ключ".to_string(),
                "источник".to_string(),
                "лето".to_string()
            ]
        ));
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

        let expected_answers = get_all_expected_answers(&card);
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

        let expected_answers = get_all_expected_answers(&card);
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

        let expected_answers = get_all_expected_answers(&card);
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
