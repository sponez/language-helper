//! Card persistence entities.
//!
//! This module defines the entities for storing flashcards in the database.

use lh_core::models::{Card, CardType, Meaning, Word};
use crate::errors::PersistenceError;

/// Persistence entity for a flashcard.
///
/// This struct represents a card as stored in the database.
#[derive(Debug, Clone, PartialEq)]
pub struct CardEntity {
    /// Unique identifier for the card.
    pub id: i64,
    /// Card type ("straight" or "reverse").
    pub card_type: String,
    /// Foreign key to words table.
    pub word_id: i64,
    /// Current streak of correct answers.
    pub streak: i32,
    /// Unix timestamp of creation.
    pub created_at: i64,
}

impl CardEntity {
    /// Creates a new CardEntity.
    pub fn new(card_type: String, word_id: i64, streak: i32) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: 0, // Will be set by database
            card_type,
            word_id,
            streak,
            created_at: now,
        }
    }

    /// Creates a CardEntity with all fields.
    pub fn with_fields(
        id: i64,
        card_type: String,
        word_id: i64,
        streak: i32,
        created_at: i64,
    ) -> Self {
        Self {
            id,
            card_type,
            word_id,
            streak,
            created_at,
        }
    }
}

/// Persistence entity for a word.
///
/// This struct represents a word as stored in the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordEntity {
    /// Unique identifier for the word.
    pub id: i64,
    /// The word text itself.
    pub name: String,
    /// Pronunciation readings as JSON array string.
    pub readings: String,
}

impl WordEntity {
    /// Creates a new WordEntity.
    pub fn new(name: String, readings: String) -> Self {
        Self {
            id: 0, // Will be set by database
            name,
            readings,
        }
    }

    /// Creates a WordEntity with all fields.
    pub fn with_fields(id: i64, name: String, readings: String) -> Self {
        Self { id, name, readings }
    }

    /// Converts domain Word to entity.
    pub fn from_domain(word: &Word) -> Result<Self, PersistenceError> {
        let readings_json = serde_json::to_string(&word.readings)
            .map_err(|e| PersistenceError::serialization_error(format!(
                "Failed to serialize readings: {}", e
            )))?;

        Ok(Self {
            id: 0,
            name: word.name.clone(),
            readings: readings_json,
        })
    }

    /// Converts entity to domain Word.
    pub fn to_domain(&self) -> Result<Word, PersistenceError> {
        let readings: Vec<String> = serde_json::from_str(&self.readings)
            .map_err(|e| PersistenceError::deserialization_error(format!(
                "Failed to deserialize readings: {}", e
            )))?;

        Ok(Word::new_unchecked(self.name.clone(), readings))
    }
}

/// Persistence entity for a word meaning.
///
/// This struct represents a meaning as stored in the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeaningEntity {
    /// Unique identifier for the meaning.
    pub id: i64,
    /// Foreign key to cards table.
    pub card_id: i64,
    /// Definition of the word.
    pub definition: String,
    /// Translated definition.
    pub translated_definition: String,
    /// Word translations as JSON array string.
    pub word_translations: String,
}

impl MeaningEntity {
    /// Creates a new MeaningEntity.
    pub fn new(
        card_id: i64,
        definition: String,
        translated_definition: String,
        word_translations: String,
    ) -> Self {
        Self {
            id: 0, // Will be set by database
            card_id,
            definition,
            translated_definition,
            word_translations,
        }
    }

    /// Creates a MeaningEntity with all fields.
    pub fn with_fields(
        id: i64,
        card_id: i64,
        definition: String,
        translated_definition: String,
        word_translations: String,
    ) -> Self {
        Self {
            id,
            card_id,
            definition,
            translated_definition,
            word_translations,
        }
    }

    /// Converts domain Meaning to entity.
    pub fn from_domain(card_id: i64, meaning: &Meaning) -> Result<Self, PersistenceError> {
        let word_translations_json = serde_json::to_string(&meaning.word_translations)
            .map_err(|e| PersistenceError::serialization_error(format!(
                "Failed to serialize word translations: {}", e
            )))?;

        Ok(Self {
            id: 0,
            card_id,
            definition: meaning.definition.clone(),
            translated_definition: meaning.translated_definition.clone(),
            word_translations: word_translations_json,
        })
    }

    /// Converts entity to domain Meaning.
    pub fn to_domain(&self) -> Result<Meaning, PersistenceError> {
        let word_translations: Vec<String> = serde_json::from_str(&self.word_translations)
            .map_err(|e| PersistenceError::deserialization_error(format!(
                "Failed to deserialize word translations: {}", e
            )))?;

        Ok(Meaning::new_unchecked(
            self.definition.clone(),
            self.translated_definition.clone(),
            word_translations,
        ))
    }
}

/// Complete card data with word and meanings.
pub struct CardWithRelations {
    pub card: CardEntity,
    pub word: WordEntity,
    pub meanings: Vec<MeaningEntity>,
}

impl CardWithRelations {
    /// Converts to domain Card.
    pub fn to_domain(&self) -> Result<Card, PersistenceError> {
        let card_type = CardType::from_str(&self.card.card_type)
            .map_err(|e| PersistenceError::deserialization_error(e.to_string()))?;

        let word = self.word.to_domain()?;

        let meanings: Result<Vec<Meaning>, PersistenceError> = self
            .meanings
            .iter()
            .map(|m| m.to_domain())
            .collect();

        Ok(Card::new_unchecked(
            Some(self.card.id),
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

    #[test]
    fn test_card_entity_creation() {
        let entity = CardEntity::new("straight".to_string(), 1, 5);
        assert_eq!(entity.card_type, "straight");
        assert_eq!(entity.word_id, 1);
        assert_eq!(entity.streak, 5);
    }

    #[test]
    fn test_word_entity_from_domain() {
        let word = Word::new_unchecked("test", vec!["reading1".to_string()]);
        let entity = WordEntity::from_domain(&word).unwrap();
        assert_eq!(entity.name, "test");
        assert!(entity.readings.contains("reading1"));
    }

    #[test]
    fn test_word_entity_to_domain() {
        let entity = WordEntity::with_fields(1, "test".to_string(), r#"["reading1"]"#.to_string());
        let word = entity.to_domain().unwrap();
        assert_eq!(word.name, "test");
        assert_eq!(word.readings.len(), 1);
        assert_eq!(word.readings[0], "reading1");
    }

    #[test]
    fn test_meaning_entity_from_domain() {
        let meaning = Meaning::new_unchecked(
            "definition",
            "traducción",
            vec!["trans1".to_string()],
        );
        let entity = MeaningEntity::from_domain(1, &meaning).unwrap();
        assert_eq!(entity.definition, "definition");
        assert_eq!(entity.translated_definition, "traducción");
    }

    #[test]
    fn test_meaning_entity_to_domain() {
        let entity = MeaningEntity::with_fields(
            1,
            1,
            "def".to_string(),
            "trad".to_string(),
            r#"["trans1"]"#.to_string(),
        );
        let meaning = entity.to_domain().unwrap();
        assert_eq!(meaning.definition, "def");
        assert_eq!(meaning.word_translations.len(), 1);
    }
}
