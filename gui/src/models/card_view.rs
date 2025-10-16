//! Card view models for GUI presentation.

use lh_api::models::card::{CardDto, CardType, MeaningDto, WordDto};

/// View model for displaying a flashcard in the GUI.
#[derive(Debug, Clone, PartialEq)]
pub struct CardView {
    /// Card type display string
    pub card_type_display: String,
    /// The underlying card type
    pub card_type: CardType,
    /// Word being learned
    pub word: WordView,
    /// List of meanings
    pub meanings: Vec<MeaningView>,
    /// Current streak
    pub streak: i32,
    /// Human-readable creation date
    pub created_at_display: String,
}

impl CardView {
    /// Creates a CardView from a CardDto.
    pub fn from_dto(dto: CardDto) -> Self {
        let card_type_display = match &dto.card_type {
            CardType::Straight => "Straight".to_string(),
            CardType::Reverse => "Reverse".to_string(),
        };

        let created_at_display = format_timestamp(dto.created_at);

        Self {
            card_type_display,
            card_type: dto.card_type,
            word: WordView::from_dto(dto.word),
            meanings: dto
                .meanings
                .into_iter()
                .map(MeaningView::from_dto)
                .collect(),
            streak: dto.streak,
            created_at_display,
        }
    }

    /// Checks if the card is learned (based on a threshold).
    pub fn is_learned(&self, streak_threshold: i32) -> bool {
        self.streak >= streak_threshold
    }
}

/// View model for displaying a word in the GUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordView {
    /// The word text
    pub name: String,
    /// Pronunciation readings
    pub readings: Vec<String>,
    /// Display string with readings
    pub display_with_readings: String,
}

impl WordView {
    /// Creates a WordView from a WordDto.
    pub fn from_dto(dto: WordDto) -> Self {
        let display_with_readings = if dto.readings.is_empty() {
            dto.name.clone()
        } else {
            format!("{} ({})", dto.name, dto.readings.join(", "))
        };

        Self {
            name: dto.name,
            readings: dto.readings,
            display_with_readings,
        }
    }
}

/// View model for displaying a meaning in the GUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeaningView {
    /// Definition text
    pub definition: String,
    /// Translated definition
    pub translated_definition: String,
    /// Word translations
    pub word_translations: Vec<String>,
    /// Display string combining translations
    pub translations_display: String,
}

impl MeaningView {
    /// Creates a MeaningView from a MeaningDto.
    pub fn from_dto(dto: MeaningDto) -> Self {
        let translations_display = if dto.word_translations.is_empty() {
            String::new()
        } else {
            dto.word_translations.join(", ")
        };

        Self {
            definition: dto.definition,
            translated_definition: dto.translated_definition,
            word_translations: dto.word_translations,
            translations_display,
        }
    }
}

/// Helper function to format Unix timestamp to human-readable string.
fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, Utc};

    match DateTime::<Utc>::from_timestamp(timestamp, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => "Invalid date".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_view_from_dto() {
        let dto = WordDto {
            name: "食べる".to_string(),
            readings: vec!["たべる".to_string()],
        };

        let view = WordView::from_dto(dto);
        assert_eq!(view.name, "食べる");
        assert_eq!(view.display_with_readings, "食べる (たべる)");
    }

    #[test]
    fn test_word_view_no_readings() {
        let dto = WordDto {
            name: "hello".to_string(),
            readings: vec![],
        };

        let view = WordView::from_dto(dto);
        assert_eq!(view.display_with_readings, "hello");
    }

    #[test]
    fn test_meaning_view_from_dto() {
        let dto = MeaningDto {
            definition: "to eat".to_string(),
            translated_definition: "comer".to_string(),
            word_translations: vec!["eat".to_string(), "consume".to_string()],
        };

        let view = MeaningView::from_dto(dto);
        assert_eq!(view.translations_display, "eat, consume");
    }

    #[test]
    fn test_card_view_is_learned() {
        let dto = CardDto {
            card_type: CardType::Straight,
            word: WordDto {
                name: "test".to_string(),
                readings: vec![],
            },
            meanings: vec![MeaningDto {
                definition: "def".to_string(),
                translated_definition: "trad".to_string(),
                word_translations: vec![],
            }],
            streak: 5,
            created_at: 1000,
        };

        let view = CardView::from_dto(dto);
        assert!(view.is_learned(5));
        assert!(view.is_learned(4));
        assert!(!view.is_learned(6));
    }
}
