//! Test result model for tracking individual card test results.
//!
//! This module defines the result of testing a single card during a learning session.

/// Result of testing a single card
#[derive(Debug, Clone, PartialEq)]
pub struct TestResult {
    /// The word name that was tested
    pub word_name: String,
    /// Whether the answer was correct
    pub is_correct: bool,
    /// The user's answer (for written tests)
    pub user_answer: Option<String>,
    /// The expected answer (for written tests)
    pub expected_answer: Option<String>,
}

impl TestResult {
    /// Creates a new test result for a written test
    ///
    /// # Arguments
    ///
    /// * `word_name` - The word that was tested
    /// * `is_correct` - Whether the answer was correct
    /// * `user_answer` - The answer provided by the user
    /// * `expected_answer` - The expected correct answer
    pub fn new_written(
        word_name: String,
        is_correct: bool,
        user_answer: String,
        expected_answer: String,
    ) -> Self {
        Self {
            word_name,
            is_correct,
            user_answer: Some(user_answer),
            expected_answer: Some(expected_answer),
        }
    }

    /// Creates a new test result for a self-review test
    ///
    /// # Arguments
    ///
    /// * `word_name` - The word that was tested
    /// * `is_correct` - Whether the user marked it as correct
    pub fn new_self_review(word_name: String, is_correct: bool) -> Self {
        Self {
            word_name,
            is_correct,
            user_answer: None,
            expected_answer: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_written_result_creation() {
        let result = TestResult::new_written(
            "hello".to_string(),
            true,
            "hola".to_string(),
            "hola".to_string(),
        );

        assert_eq!(result.word_name, "hello");
        assert!(result.is_correct);
        assert_eq!(result.user_answer, Some("hola".to_string()));
        assert_eq!(result.expected_answer, Some("hola".to_string()));
    }

    #[test]
    fn test_self_review_result_creation() {
        let result = TestResult::new_self_review("hello".to_string(), false);

        assert_eq!(result.word_name, "hello");
        assert!(!result.is_correct);
        assert_eq!(result.user_answer, None);
        assert_eq!(result.expected_answer, None);
    }
}
