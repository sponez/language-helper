//! Card domain models.
//!
//! This module defines the flashcard entities and their components.

use crate::errors::CoreError;

/// Card type indicating the learning direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardType {
    /// Straight: from target language to user's native language
    Straight,
    /// Reverse: from user's native language to target language
    Reverse,
}

impl CardType {
    /// Converts card type to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            CardType::Straight => "straight",
            CardType::Reverse => "reverse",
        }
    }

    /// Creates card type from string.
    pub fn from_str(s: &str) -> Result<Self, CoreError> {
        match s.to_lowercase().as_str() {
            "straight" => Ok(CardType::Straight),
            "reverse" => Ok(CardType::Reverse),
            _ => Err(CoreError::validation_error(format!(
                "Invalid card type: {}. Must be 'straight' or 'reverse'",
                s
            ))),
        }
    }
}

/// Domain model for a flashcard.
///
/// A card represents a learning unit with a word and its meanings.
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    /// Unique identifier (None for new cards).
    pub id: Option<i64>,
    /// Card type determining learning direction.
    pub card_type: CardType,
    /// The word being learned.
    pub word: Word,
    /// List of meanings for the word.
    pub meanings: Vec<Meaning>,
    /// Current streak of correct answers.
    pub streak: i32,
    /// Unix timestamp of creation.
    pub created_at: i64,
}

impl Card {
    /// Creates a new Card with validation.
    ///
    /// # Arguments
    ///
    /// * `card_type` - The type of card (Straight or Reverse)
    /// * `word` - The word being learned
    /// * `meanings` - List of meanings (must not be empty)
    ///
    /// # Returns
    ///
    /// * `Ok(Card)` - If validation passes
    /// * `Err(CoreError)` - If validation fails
    pub fn new(card_type: CardType, word: Word, meanings: Vec<Meaning>) -> Result<Self, CoreError> {
        if meanings.is_empty() {
            return Err(CoreError::validation_error(
                "Card must have at least one meaning",
            ));
        }

        let now = chrono::Utc::now().timestamp();

        Ok(Self {
            id: None,
            card_type,
            word,
            meanings,
            streak: 0,
            created_at: now,
        })
    }

    /// Creates a Card without validation (for loading from database).
    pub fn new_unchecked(
        id: Option<i64>,
        card_type: CardType,
        word: Word,
        meanings: Vec<Meaning>,
        streak: i32,
        created_at: i64,
    ) -> Self {
        Self {
            id,
            card_type,
            word,
            meanings,
            streak,
            created_at,
        }
    }

    /// Increments the streak counter.
    pub fn increment_streak(&mut self) {
        self.streak += 1;
    }

    /// Resets the streak counter to zero.
    pub fn reset_streak(&mut self) {
        self.streak = 0;
    }

    /// Updates the streak to a specific value.
    pub fn update_streak(&mut self, streak: i32) -> Result<(), CoreError> {
        if streak < 0 {
            return Err(CoreError::validation_error("Streak cannot be negative"));
        }
        self.streak = streak;
        Ok(())
    }
}

/// Domain model for a word.
///
/// A word consists of its text and pronunciation readings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    /// The word text itself.
    pub name: String,
    /// Pronunciation readings.
    pub readings: Vec<String>,
}

impl Word {
    /// Creates a new Word with validation.
    ///
    /// # Arguments
    ///
    /// * `name` - The word text
    /// * `readings` - Pronunciation readings (can be empty)
    ///
    /// # Returns
    ///
    /// * `Ok(Word)` - If validation passes
    /// * `Err(CoreError)` - If validation fails
    pub fn new<S: Into<String>>(name: S, readings: Vec<String>) -> Result<Self, CoreError> {
        let name = name.into();

        if name.trim().is_empty() {
            return Err(CoreError::validation_error("Word name cannot be empty"));
        }

        if name.len() > 200 {
            return Err(CoreError::validation_error(
                "Word name cannot exceed 200 characters",
            ));
        }

        Ok(Self { name, readings })
    }

    /// Creates a Word without validation.
    pub fn new_unchecked<S: Into<String>>(name: S, readings: Vec<String>) -> Self {
        Self {
            name: name.into(),
            readings,
        }
    }
}

/// Domain model for a word meaning.
///
/// A meaning includes the definition and translations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meaning {
    /// Definition of the word.
    pub definition: String,
    /// Translated definition.
    pub translated_definition: String,
    /// Translations of the word itself.
    pub word_translations: Vec<String>,
}

impl Meaning {
    /// Creates a new Meaning with validation.
    ///
    /// # Arguments
    ///
    /// * `definition` - The definition text
    /// * `translated_definition` - The translated definition
    /// * `word_translations` - List of word translations
    ///
    /// # Returns
    ///
    /// * `Ok(Meaning)` - If validation passes
    /// * `Err(CoreError)` - If validation fails
    pub fn new<S1: Into<String>, S2: Into<String>>(
        definition: S1,
        translated_definition: S2,
        word_translations: Vec<String>,
    ) -> Result<Self, CoreError> {
        let definition = definition.into();
        let translated_definition = translated_definition.into();

        if definition.trim().is_empty() {
            return Err(CoreError::validation_error("Definition cannot be empty"));
        }

        if translated_definition.trim().is_empty() {
            return Err(CoreError::validation_error(
                "Translated definition cannot be empty",
            ));
        }

        if definition.len() > 1000 {
            return Err(CoreError::validation_error(
                "Definition cannot exceed 1000 characters",
            ));
        }

        if translated_definition.len() > 1000 {
            return Err(CoreError::validation_error(
                "Translated definition cannot exceed 1000 characters",
            ));
        }

        Ok(Self {
            definition,
            translated_definition,
            word_translations,
        })
    }

    /// Creates a Meaning without validation.
    pub fn new_unchecked<S1: Into<String>, S2: Into<String>>(
        definition: S1,
        translated_definition: S2,
        word_translations: Vec<String>,
    ) -> Self {
        Self {
            definition: definition.into(),
            translated_definition: translated_definition.into(),
            word_translations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_type_as_str() {
        assert_eq!(CardType::Straight.as_str(), "straight");
        assert_eq!(CardType::Reverse.as_str(), "reverse");
    }

    #[test]
    fn test_card_type_from_str() {
        assert_eq!(CardType::from_str("straight").unwrap(), CardType::Straight);
        assert_eq!(CardType::from_str("reverse").unwrap(), CardType::Reverse);
        assert_eq!(CardType::from_str("STRAIGHT").unwrap(), CardType::Straight);
        assert!(CardType::from_str("invalid").is_err());
    }

    #[test]
    fn test_word_creation() {
        let word = Word::new("hello", vec!["heh-loh".to_string()]).unwrap();
        assert_eq!(word.name, "hello");
        assert_eq!(word.readings.len(), 1);
    }

    #[test]
    fn test_word_validation() {
        assert!(Word::new("", vec![]).is_err());
        assert!(Word::new("   ", vec![]).is_err());
        assert!(Word::new("a".repeat(201), vec![]).is_err());
    }

    #[test]
    fn test_meaning_creation() {
        let meaning = Meaning::new(
            "to eat",
            "comer",
            vec!["eat".to_string(), "consume".to_string()],
        )
        .unwrap();
        assert_eq!(meaning.definition, "to eat");
        assert_eq!(meaning.translated_definition, "comer");
        assert_eq!(meaning.word_translations.len(), 2);
    }

    #[test]
    fn test_meaning_validation() {
        assert!(Meaning::new("", "valid", vec![]).is_err());
        assert!(Meaning::new("valid", "", vec![]).is_err());
        assert!(Meaning::new("a".repeat(1001), "valid", vec![]).is_err());
    }

    #[test]
    fn test_card_creation() {
        let word = Word::new("test", vec![]).unwrap();
        let meaning = Meaning::new("definition", "traducción", vec![]).unwrap();
        let card = Card::new(CardType::Straight, word, vec![meaning]).unwrap();

        assert_eq!(card.card_type, CardType::Straight);
        assert_eq!(card.streak, 0);
        assert!(card.id.is_none());
    }

    #[test]
    fn test_card_validation() {
        let word = Word::new("test", vec![]).unwrap();
        assert!(Card::new(CardType::Straight, word, vec![]).is_err());
    }

    #[test]
    fn test_card_streak_operations() {
        let word = Word::new("test", vec![]).unwrap();
        let meaning = Meaning::new("def", "trad", vec![]).unwrap();
        let mut card = Card::new(CardType::Straight, word, vec![meaning]).unwrap();

        assert_eq!(card.streak, 0);

        card.increment_streak();
        assert_eq!(card.streak, 1);

        card.increment_streak();
        assert_eq!(card.streak, 2);

        card.reset_streak();
        assert_eq!(card.streak, 0);

        card.update_streak(5).unwrap();
        assert_eq!(card.streak, 5);

        assert!(card.update_streak(-1).is_err());
    }
}
