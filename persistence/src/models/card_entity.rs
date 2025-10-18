//! Card persistence entities.
//!
//! This module defines the entities for storing flashcards in the database.

use crate::errors::PersistenceError;
use lh_core::models::{Card, CardType, Meaning};

/// Persistence entity for a flashcard.
///
/// This struct represents a card as stored in the database.
/// The word_name serves as the primary key.
#[derive(Debug, Clone, PartialEq)]
pub struct CardEntity {
    /// The word text itself (PRIMARY KEY).
    pub word_name: String,
    /// Card type ("straight" or "reverse").
    pub card_type: String,
    /// Pronunciation readings as JSON array string.
    pub word_readings: String,
    /// Current streak of correct answers.
    pub streak: i32,
    /// Unix timestamp of creation.
    pub created_at: i64,
}

impl CardEntity {
    /// Creates a new CardEntity from domain Card.
    pub fn from_domain(card: &Card) -> Result<Self, PersistenceError> {
        let readings_json = serde_json::to_string(&card.word.readings).map_err(|e| {
            PersistenceError::serialization_error(format!("Failed to serialize readings: {}", e))
        })?;

        Ok(Self {
            word_name: card.word.name.clone(),
            card_type: card.card_type.as_str().to_string(),
            word_readings: readings_json,
            streak: card.streak,
            created_at: card.created_at,
        })
    }
}

/// Persistence entity for a word meaning.
///
/// This struct represents a meaning as stored in the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeaningEntity {
    /// Unique identifier for the meaning.
    pub id: i64,
    /// Foreign key to cards table (word_name).
    pub word_name: String,
    /// Definition of the word.
    pub definition: String,
    /// Translated definition.
    pub translated_definition: String,
    /// Word translations as JSON array string.
    pub word_translations: String,
}

impl MeaningEntity {
    /// Converts domain Meaning to entity.
    pub fn from_domain(word_name: &str, meaning: &Meaning) -> Result<Self, PersistenceError> {
        let word_translations_json =
            serde_json::to_string(&meaning.word_translations).map_err(|e| {
                PersistenceError::serialization_error(format!(
                    "Failed to serialize word translations: {}",
                    e
                ))
            })?;

        Ok(Self {
            id: 0, // Will be set by database
            word_name: word_name.to_string(),
            definition: meaning.definition.clone(),
            translated_definition: meaning.translated_definition.clone(),
            word_translations: word_translations_json,
        })
    }

    /// Converts entity to domain Meaning.
    pub fn to_domain(&self) -> Result<Meaning, PersistenceError> {
        let word_translations: Vec<String> = serde_json::from_str(&self.word_translations)
            .map_err(|e| {
                PersistenceError::serialization_error(format!(
                    "Failed to deserialize word translations: {}",
                    e
                ))
            })?;

        Ok(Meaning::new_unchecked(
            self.definition.clone(),
            self.translated_definition.clone(),
            word_translations,
        ))
    }
}

/// Complete card data with meanings.
pub struct CardWithRelations {
    pub card: CardEntity,
    pub meanings: Vec<MeaningEntity>,
}

impl CardWithRelations {
    /// Converts to domain Card.
    pub fn to_domain(&self) -> Result<Card, PersistenceError> {
        let card_type = CardType::parse(&self.card.card_type)
            .map_err(|e| PersistenceError::serialization_error(e.to_string()))?;

        let word_readings: Vec<String> =
            serde_json::from_str(&self.card.word_readings).map_err(|e| {
                PersistenceError::serialization_error(format!(
                    "Failed to deserialize word readings: {}",
                    e
                ))
            })?;

        let word = lh_core::models::Word::new_unchecked(self.card.word_name.clone(), word_readings);

        let meanings: Result<Vec<Meaning>, PersistenceError> =
            self.meanings.iter().map(|m| m.to_domain()).collect();

        Ok(Card::new_unchecked(
            card_type,
            word,
            meanings?,
            self.card.streak,
            self.card.created_at,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lh_core::models::Word;

    #[test]
    fn test_card_entity_from_domain() {
        let word = Word::new_unchecked("test", vec!["reading1".to_string()]);
        let meanings = vec![Meaning::new_unchecked(
            "definition",
            "traducción",
            vec!["trans1".to_string()],
        )];
        let card = Card::new_unchecked(CardType::Straight, word, meanings, 5, 0);

        let entity = CardEntity::from_domain(&card).unwrap();
        assert_eq!(entity.word_name, "test");
        assert_eq!(entity.card_type, "straight");
        assert_eq!(entity.streak, 5);
        assert!(entity.word_readings.contains("reading1"));
    }

    #[test]
    fn test_meaning_entity_from_domain() {
        let meaning =
            Meaning::new_unchecked("definition", "traducción", vec!["trans1".to_string()]);
        let entity = MeaningEntity::from_domain("test_word", &meaning).unwrap();
        assert_eq!(entity.word_name, "test_word");
        assert_eq!(entity.definition, "definition");
        assert_eq!(entity.translated_definition, "traducción");
    }

    #[test]
    fn test_meaning_entity_to_domain() {
        let entity = MeaningEntity {
            id: 1,
            word_name: "test".to_string(),
            definition: "def".to_string(),
            translated_definition: "trad".to_string(),
            word_translations: r#"["trans1"]"#.to_string(),
        };
        let meaning = entity.to_domain().unwrap();
        assert_eq!(meaning.definition, "def");
        assert_eq!(meaning.word_translations.len(), 1);
    }
}
