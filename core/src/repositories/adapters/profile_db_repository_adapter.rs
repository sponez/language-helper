//! Adapter for ProfileRepository that maps persistence errors to core errors.
//!
//! This module provides an adapter that wraps a persistence-layer profile database
//! repository and converts its errors to core domain errors.

use async_trait::async_trait;
use crate::errors::CoreError;
use crate::models::{AssistantSettings, Card, CardSettings};
use crate::repositories::profile_repository::ProfileRepository;
use std::fmt::Display;
use std::path::PathBuf;

/// Trait for persistence-layer profile database repositories.
///
/// This trait must be implemented by the persistence layer to handle
/// profile database file operations with persistence-specific error types.
#[async_trait]
pub trait PersistenceProfileDbRepository: Send + Sync {
    /// The error type from the persistence layer
    type Error: Display;

    /// Creates a new profile database file.
    async fn create_database(&self, db_path: PathBuf) -> Result<(), Self::Error>;

    /// Deletes a profile database file.
    async fn delete_database(&self, db_path: PathBuf) -> Result<bool, Self::Error>;

    /// Gets card settings from a profile database.
    async fn get_card_settings(&self, db_path: PathBuf) -> Result<CardSettings, Self::Error>;

    /// Updates card settings in a profile database.
    async fn update_card_settings(&self, db_path: PathBuf, settings: CardSettings) -> Result<(), Self::Error>;

    /// Gets assistant settings from a profile database.
    async fn get_assistant_settings(&self, db_path: PathBuf) -> Result<AssistantSettings, Self::Error>;

    /// Updates assistant settings in a profile database.
    async fn update_assistant_settings(&self, db_path: PathBuf, settings: AssistantSettings) -> Result<(), Self::Error>;

    /// Clears assistant settings in a profile database.
    async fn clear_assistant_settings(&self, db_path: PathBuf) -> Result<(), Self::Error>;

    /// Creates a new card in the profile database.
    async fn create_card(&self, db_path: PathBuf, card: Card) -> Result<i64, Self::Error>;

    /// Gets all cards from the profile database.
    async fn get_all_cards(&self, db_path: PathBuf) -> Result<Vec<Card>, Self::Error>;

    /// Gets cards filtered by streak threshold.
    async fn get_cards_by_learned_status(
        &self,
        db_path: PathBuf,
        streak_threshold: i32,
        learned: bool,
    ) -> Result<Vec<Card>, Self::Error>;

    /// Gets a single card by ID.
    async fn get_card_by_id(&self, db_path: PathBuf, card_id: i64) -> Result<Card, Self::Error>;

    /// Updates a card's streak.
    async fn update_card_streak(&self, db_path: PathBuf, card_id: i64, streak: i32) -> Result<(), Self::Error>;

    /// Deletes a card from the database.
    async fn delete_card(&self, db_path: PathBuf, card_id: i64) -> Result<bool, Self::Error>;
}

/// Adapter that wraps a persistence repository and converts errors to CoreError.
///
/// This struct implements the core ProfileRepository trait by delegating to
/// a persistence-layer implementation and mapping errors.
pub struct ProfileDbRepositoryAdapter<R: PersistenceProfileDbRepository> {
    repository: R,
}

impl<R: PersistenceProfileDbRepository> ProfileDbRepositoryAdapter<R> {
    /// Creates a new ProfileDbRepositoryAdapter.
    ///
    /// # Arguments
    ///
    /// * `repository` - The persistence-layer repository to wrap
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: PersistenceProfileDbRepository> ProfileRepository for ProfileDbRepositoryAdapter<R> {
    async fn create_database(&self, db_path: PathBuf) -> Result<(), CoreError> {
        self.repository
            .create_database(db_path)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn delete_database(&self, db_path: PathBuf) -> Result<bool, CoreError> {
        self.repository
            .delete_database(db_path)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn get_card_settings(&self, db_path: PathBuf) -> Result<CardSettings, CoreError> {
        self.repository
            .get_card_settings(db_path)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn update_card_settings(&self, db_path: PathBuf, settings: CardSettings) -> Result<(), CoreError> {
        self.repository
            .update_card_settings(db_path, settings)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn get_assistant_settings(&self, db_path: PathBuf) -> Result<AssistantSettings, CoreError> {
        self.repository
            .get_assistant_settings(db_path)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn update_assistant_settings(&self, db_path: PathBuf, settings: AssistantSettings) -> Result<(), CoreError> {
        self.repository
            .update_assistant_settings(db_path, settings)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn clear_assistant_settings(&self, db_path: PathBuf) -> Result<(), CoreError> {
        self.repository
            .clear_assistant_settings(db_path)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn create_card(&self, db_path: PathBuf, card: Card) -> Result<i64, CoreError> {
        self.repository
            .create_card(db_path, card)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn get_all_cards(&self, db_path: PathBuf) -> Result<Vec<Card>, CoreError> {
        self.repository
            .get_all_cards(db_path)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn get_cards_by_learned_status(
        &self,
        db_path: PathBuf,
        streak_threshold: i32,
        learned: bool,
    ) -> Result<Vec<Card>, CoreError> {
        self.repository
            .get_cards_by_learned_status(db_path, streak_threshold, learned)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn get_card_by_id(&self, db_path: PathBuf, card_id: i64) -> Result<Card, CoreError> {
        self.repository
            .get_card_by_id(db_path, card_id)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn update_card_streak(&self, db_path: PathBuf, card_id: i64, streak: i32) -> Result<(), CoreError> {
        self.repository
            .update_card_streak(db_path, card_id, streak)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn delete_card(&self, db_path: PathBuf, card_id: i64) -> Result<bool, CoreError> {
        self.repository
            .delete_card(db_path, card_id)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }
}
