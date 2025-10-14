//! Card data transfer objects.
//!
//! This module defines the DTOs for flashcards and their components.

use serde::{Deserialize, Serialize};

/// Card type indicating the learning direction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CardType {
    /// Straight: from target language to user's native language
    Straight,
    /// Reverse: from user's native language to target language
    Reverse,
}

/// Data transfer object for a flashcard.
///
/// A card represents a learning unit with a word and its meanings.
/// The card type determines the direction of learning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardDto {
    /// Card type (Straight or Reverse).
    pub card_type: CardType,
    /// The word being learned.
    pub word: WordDto,
    /// List of meanings for the word.
    pub meanings: Vec<MeaningDto>,
    /// Current streak of correct answers.
    pub streak: i32,
    /// Unix timestamp of creation.
    pub created_at: i64,
}

/// Data transfer object for a word.
///
/// A word consists of its text and pronunciation readings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordDto {
    /// The word text itself.
    pub name: String,
    /// Pronunciation readings (e.g., hiragana for Japanese kanji).
    pub readings: Vec<String>,
}

/// Data transfer object for a word meaning.
///
/// A meaning includes the definition and translations.
/// The language of definition and translated_definition depends on card type:
/// - Straight: definition is in target language, translated_definition is in user's native language
/// - Reverse: definition is in user's native language, translated_definition is in target language
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeaningDto {
    /// Definition of the word.
    pub definition: String,
    /// Translated definition.
    pub translated_definition: String,
    /// Translations of the word itself.
    pub word_translations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_type_serialization() {
        let straight = CardType::Straight;
        let json = serde_json::to_string(&straight).unwrap();
        assert_eq!(json, r#""straight""#);

        let reverse = CardType::Reverse;
        let json = serde_json::to_string(&reverse).unwrap();
        assert_eq!(json, r#""reverse""#);
    }

    #[test]
    fn test_card_type_deserialization() {
        let json = r#""straight""#;
        let card_type: CardType = serde_json::from_str(json).unwrap();
        assert_eq!(card_type, CardType::Straight);

        let json = r#""reverse""#;
        let card_type: CardType = serde_json::from_str(json).unwrap();
        assert_eq!(card_type, CardType::Reverse);
    }

    #[test]
    fn test_word_dto_creation() {
        let word = WordDto {
            name: "食べる".to_string(),
            readings: vec!["たべる".to_string()],
        };
        assert_eq!(word.name, "食べる");
        assert_eq!(word.readings.len(), 1);
    }

    #[test]
    fn test_meaning_dto_creation() {
        let meaning = MeaningDto {
            definition: "to eat".to_string(),
            translated_definition: "comer".to_string(),
            word_translations: vec!["eat".to_string(), "consume".to_string()],
        };
        assert_eq!(meaning.definition, "to eat");
        assert_eq!(meaning.word_translations.len(), 2);
    }

    #[test]
    fn test_card_dto_serialization() {
        let card = CardDto {
            card_type: CardType::Straight,
            word: WordDto {
                name: "hello".to_string(),
                readings: vec!["heh-loh".to_string()],
            },
            meanings: vec![MeaningDto {
                definition: "greeting".to_string(),
                translated_definition: "saludo".to_string(),
                word_translations: vec!["hola".to_string()],
            }],
            streak: 0,
            created_at: 1000,
        };

        let json = serde_json::to_string(&card).unwrap();
        assert!(json.contains("straight"));
        assert!(json.contains("hello"));
    }
}
