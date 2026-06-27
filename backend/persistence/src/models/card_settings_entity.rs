//! Card settings persistence entity.
//!
//! This module defines the database representation of card settings.

use crate::errors::PersistenceError;
use lh_core::models::CardSettings;

/// Database representation of card settings.
///
/// This entity maps directly to the card_settings table in the profile database.
#[derive(Debug, Clone)]
pub struct CardSettingsEntity {
    /// Primary key (always 1, as there's only one settings record per profile)
    pub id: i64,
    /// Number of cards to show per learning set
    pub cards_per_set: i64,
    /// Method for testing answers ("manual" or "self_review")
    pub test_answer_method: String,
    /// Number of consecutive correct answers needed to mark as "remembered"
    pub streak_length: i64,
}

impl CardSettingsEntity {
    /// Converts this persistence entity to a domain model.
    pub fn to_domain(self) -> Result<CardSettings, PersistenceError> {
        Ok(CardSettings::new(
            self.cards_per_set as u32,
            self.test_answer_method,
            self.streak_length as u32,
        ))
    }

    /// Creates a persistence entity from a domain model.
    pub fn from_domain(settings: CardSettings) -> Self {
        Self {
            id: 1, // Always use ID 1 for the single settings record
            cards_per_set: settings.cards_per_set as i64,
            test_answer_method: settings.test_answer_method,
            streak_length: settings.streak_length as i64,
        }
    }
}

impl Default for CardSettingsEntity {
    fn default() -> Self {
        Self::from_domain(CardSettings::default())
    }
}
