//! Test result data transfer objects.
//!
//! This module defines DTOs for individual card test results.

use serde::{Deserialize, Serialize};

/// Data transfer object for a test result.
///
/// Tracks the result of testing a single card, including whether
/// the answer was correct and what answers were provided/expected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResultDto {
    /// The word that was tested.
    pub word_name: String,
    /// Whether the answer was correct.
    pub is_correct: bool,
    /// The user's answer (None for self-review).
    pub user_answer: Option<String>,
    /// The expected answer (None for self-review).
    pub expected_answer: Option<String>,
}

impl TestResultDto {
    /// Creates a new test result for a written test.
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

    /// Creates a new test result for a self-review test.
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
    fn test_written_test_result() {
        let result = TestResultDto::new_written(
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
    fn test_self_review_test_result() {
        let result = TestResultDto::new_self_review("hello".to_string(), false);

        assert_eq!(result.word_name, "hello");
        assert!(!result.is_correct);
        assert_eq!(result.user_answer, None);
        assert_eq!(result.expected_answer, None);
    }

    #[test]
    fn test_serialization() {
        let result = TestResultDto::new_written(
            "test".to_string(),
            true,
            "answer".to_string(),
            "expected".to_string(),
        );

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("answer"));
    }
}
