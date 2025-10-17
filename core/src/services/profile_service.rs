//! Profile service for managing individual profile databases.
//!
//! This module provides the business logic for managing profile-specific databases
//! where learning content (vocabulary cards, progress, etc.) is stored.
//! Each profile gets its own database file at `data/{username}/{target_language}_profile.db`.

use crate::errors::CoreError;
use crate::models::{AssistantSettings, Card, CardSettings, CardType};
use crate::repositories::profile_repository::ProfileRepository;
use std::path::PathBuf;

/// Service for profile database management.
///
/// This struct implements the business logic for creating and managing profile-specific
/// databases. Each profile has its own database file for storing learning content.
///
/// # Type Parameters
///
/// * `R` - The profile repository implementation type
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::profile_service::ProfileService;
/// use lh_core::repositories::profile_repository::ProfileRepository;
///
/// async fn example(repo: impl ProfileRepository) {
///     let service = ProfileService::new(repo, "data");
///
///     match service.create_profile_database("john_doe", "spanish").await {
///         Ok(path) => println!("Created database at: {:?}", path),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub struct ProfileService<R: ProfileRepository> {
    repository: R,
    data_dir: String,
}

impl<R: ProfileRepository> ProfileService<R> {
    /// Creates a new ProfileService instance.
    ///
    /// # Arguments
    ///
    /// * `repository` - The profile repository implementation
    /// * `data_dir` - The base data directory path (e.g., "data")
    ///
    /// # Returns
    ///
    /// A new `ProfileService` instance.
    pub fn new(repository: R, data_dir: impl Into<String>) -> Self {
        Self {
            repository,
            data_dir: data_dir.into(),
        }
    }

    /// Creates a new profile database.
    ///
    /// This creates a database file at `data/{username}/{target_language}_profile.db`
    /// and initializes it with the necessary schema (currently empty, for future use).
    ///
    /// # Arguments
    ///
    /// * `username` - The username this profile belongs to
    /// * `target_language` - The target language being learned
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The path to the created database file
    /// * `Err(CoreError)` - If an error occurs during creation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::ValidationError` if parameters are invalid,
    /// or `CoreError::RepositoryError` if there's a problem creating the database.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::profile_service::ProfileService;
    /// # use lh_core::repositories::profile_repository::ProfileRepository;
    /// # async fn example(service: &ProfileService<impl ProfileRepository>) {
    /// match service.create_profile_database("jane_doe", "french").await {
    ///     Ok(path) => println!("Database created at: {:?}", path),
    ///     Err(e) => eprintln!("Failed to create database: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn create_profile_database(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<PathBuf, CoreError> {
        // Validate inputs
        if username.is_empty() {
            return Err(CoreError::validation_error("Username cannot be empty"));
        }
        if target_language.is_empty() {
            return Err(CoreError::validation_error(
                "Target language cannot be empty",
            ));
        }

        // Build the path: data/{username}/{target_language}_profile.db
        let user_dir = PathBuf::from(&self.data_dir).join(username);
        let db_filename = format!("{}_profile.db", target_language);
        let db_path = user_dir.join(db_filename);

        // Create the database using the repository
        self.repository.create_database(db_path.clone()).await?;

        Ok(db_path)
    }

    /// Checks if a profile database exists.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - true if the database exists, false otherwise
    /// * `Err(CoreError)` - If an error occurs during the check
    pub async fn profile_database_exists(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<bool, CoreError> {
        let user_dir = PathBuf::from(&self.data_dir).join(username);
        let db_filename = format!("{}_profile.db", target_language);
        let db_path = user_dir.join(db_filename);

        Ok(db_path.exists())
    }

    /// Deletes a profile database.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - true if the database was deleted, false if it didn't exist
    /// * `Err(CoreError)` - If an error occurs during deletion
    pub async fn delete_profile_database(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<bool, CoreError> {
        let user_dir = PathBuf::from(&self.data_dir).join(username);
        let db_filename = format!("{}_profile.db", target_language);
        let db_path = user_dir.join(db_filename);

        self.repository.delete_database(db_path).await
    }

    /// Deletes the entire user data folder.
    ///
    /// This removes the folder `data/{username}/` and all its contents.
    ///
    /// # Arguments
    ///
    /// * `username` - The username whose folder should be deleted
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - true if the folder was deleted, false if it didn't exist
    /// * `Err(CoreError)` - If an error occurs during deletion
    pub async fn delete_user_folder(&self, username: &str) -> Result<bool, CoreError> {
        let user_dir = PathBuf::from(&self.data_dir).join(username);

        if !user_dir.exists() {
            return Ok(false);
        }

        std::fs::remove_dir_all(&user_dir).map_err(|e| {
            CoreError::repository_error(format!(
                "Failed to delete user folder {}: {}",
                user_dir.display(),
                e
            ))
        })?;

        Ok(true)
    }

    /// Helper method to construct database path from username and target language
    fn get_db_path(&self, username: &str, target_language: &str) -> PathBuf {
        let user_dir = PathBuf::from(&self.data_dir).join(username);
        let db_filename = format!("{}_profile.db", target_language);
        user_dir.join(db_filename)
    }

    /// Gets card settings from a profile database.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language
    ///
    /// # Returns
    ///
    /// * `Ok(CardSettings)` - The card settings from the database
    /// * `Err(CoreError)` - If an error occurs
    pub async fn get_card_settings(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<CardSettings, CoreError> {
        let db_path = self.get_db_path(username, target_language);
        self.repository.get_card_settings(db_path).await
    }

    /// Updates card settings in a profile database.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language
    /// * `settings` - The card settings to save
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the settings were successfully saved
    /// * `Err(CoreError)` - If an error occurs
    pub async fn update_card_settings(
        &self,
        username: &str,
        target_language: &str,
        settings: CardSettings,
    ) -> Result<(), CoreError> {
        let db_path = self.get_db_path(username, target_language);
        self.repository
            .update_card_settings(db_path, settings)
            .await
    }

    /// Gets assistant settings from a profile database.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language
    ///
    /// # Returns
    ///
    /// * `Ok(AssistantSettings)` - The assistant settings from the database
    /// * `Err(CoreError)` - If an error occurs
    pub async fn get_assistant_settings(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<AssistantSettings, CoreError> {
        let db_path = self.get_db_path(username, target_language);
        self.repository.get_assistant_settings(db_path).await
    }

    /// Updates assistant settings in a profile database.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language
    /// * `settings` - The assistant settings to save
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the settings were successfully saved
    /// * `Err(CoreError)` - If an error occurs
    pub async fn update_assistant_settings(
        &self,
        username: &str,
        target_language: &str,
        settings: AssistantSettings,
    ) -> Result<(), CoreError> {
        let db_path = self.get_db_path(username, target_language);
        self.repository
            .update_assistant_settings(db_path, settings)
            .await
    }

    /// Clears assistant settings in a profile database (sets all AI fields to None).
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the settings were successfully cleared
    /// * `Err(CoreError)` - If an error occurs
    pub async fn clear_assistant_settings(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<(), CoreError> {
        let db_path = self.get_db_path(username, target_language);
        self.repository.clear_assistant_settings(db_path).await
    }

    /// Saves a card to the profile database (creates or updates based on word_name).
    pub async fn save_card(
        &self,
        username: &str,
        target_language: &str,
        card: Card,
    ) -> Result<(), CoreError> {
        let db_path = self.get_db_path(username, target_language);
        self.repository.save_card(db_path, card).await
    }

    /// Gets all cards from the profile database.
    pub async fn get_all_cards(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<Vec<Card>, CoreError> {
        let db_path = self.get_db_path(username, target_language);
        self.repository.get_all_cards(db_path).await
    }

    /// Gets unlearned cards (streak below threshold).
    pub async fn get_unlearned_cards(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<Vec<Card>, CoreError> {
        let db_path = self.get_db_path(username, target_language);
        // Get streak threshold from card settings
        let settings = self.repository.get_card_settings(db_path.clone()).await?;
        self.repository
            .get_cards_by_learned_status(db_path, settings.streak_length as i32, false)
            .await
    }

    /// Gets learned cards (streak at or above threshold).
    pub async fn get_learned_cards(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<Vec<Card>, CoreError> {
        let db_path = self.get_db_path(username, target_language);
        // Get streak threshold from card settings
        let settings = self.repository.get_card_settings(db_path.clone()).await?;
        self.repository
            .get_cards_by_learned_status(db_path, settings.streak_length as i32, true)
            .await
    }

    /// Gets a single card by word name.
    pub async fn get_card_by_word_name(
        &self,
        username: &str,
        target_language: &str,
        word_name: &str,
    ) -> Result<Card, CoreError> {
        let db_path = self.get_db_path(username, target_language);
        self.repository
            .get_card_by_word_name(db_path, word_name.to_string())
            .await
    }

    /// Updates a card's streak.
    pub async fn update_card_streak(
        &self,
        username: &str,
        target_language: &str,
        word_name: &str,
        streak: i32,
    ) -> Result<(), CoreError> {
        let db_path = self.get_db_path(username, target_language);
        self.repository
            .update_card_streak(db_path, word_name.to_string(), streak)
            .await
    }

    /// Deletes a card from the database.
    pub async fn delete_card(
        &self,
        username: &str,
        target_language: &str,
        word_name: &str,
    ) -> Result<bool, CoreError> {
        let db_path = self.get_db_path(username, target_language);
        self.repository
            .delete_card(db_path, word_name.to_string())
            .await
    }

    /// Generates inverted cards from an original card.
    ///
    /// For each translation in the original card's meanings:
    /// - If a card with that word_name exists, add a new meaning to it
    /// - If no card exists, create a new inverse card
    /// - Swap definition and translated_definition
    /// - Set card_type to opposite (Straight ↔ Reverse)
    /// - Set word_readings to empty, streak to 0
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language
    /// * `original_card` - The card to generate inverses from
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Card>)` - List of generated inverse cards
    /// * `Err(CoreError)` - If an error occurs
    pub async fn get_inverted_cards(
        &self,
        username: &str,
        target_language: &str,
        original_card: &Card,
    ) -> Result<Vec<Card>, CoreError> {
        use std::collections::HashMap;

        let db_path = self.get_db_path(username, target_language);

        // Collect all translations from all meanings
        let mut translations_with_meanings: Vec<(String, &crate::models::card::Meaning)> =
            Vec::new();
        for meaning in &original_card.meanings {
            for translation in &meaning.word_translations {
                translations_with_meanings.push((translation.clone(), meaning));
            }
        }

        // Group by translation to handle duplicates
        let mut translation_map: HashMap<String, Vec<&crate::models::card::Meaning>> =
            HashMap::new();
        for (translation, meaning) in translations_with_meanings {
            translation_map
                .entry(translation)
                .or_insert_with(Vec::new)
                .push(meaning);
        }

        // Generate inverse cards
        let mut inverse_cards = Vec::new();
        let inverse_card_type = match original_card.card_type {
            CardType::Straight => CardType::Reverse,
            CardType::Reverse => CardType::Straight,
        };

        for (translation, meanings) in translation_map {
            // Try to get existing card with this word_name
            let existing_card_result = self
                .repository
                .get_card_by_word_name(db_path.clone(), translation.clone())
                .await;

            let mut inverse_card = if let Ok(existing_card) = existing_card_result {
                // Card exists - we'll add new meanings to it
                existing_card
            } else {
                // Card doesn't exist - create new one
                let word = crate::models::card::Word::new_unchecked(translation.clone(), vec![]);
                crate::models::card::Card::new_unchecked(
                    inverse_card_type.clone(),
                    word,
                    vec![],
                    0,
                    chrono::Utc::now().timestamp(),
                )
            };

            // Add inverted meanings (one for each occurrence of this translation)
            for meaning in meanings {
                // Swap definition and translated_definition
                let inverted_meaning = crate::models::card::Meaning::new_unchecked(
                    meaning.translated_definition.clone(),
                    meaning.definition.clone(),
                    vec![original_card.word.name.clone()],
                );
                inverse_card.meanings.push(inverted_meaning);
            }

            inverse_cards.push(inverse_card);
        }

        Ok(inverse_cards)
    }

    /// Processes test results and updates card streaks based on performance.
    ///
    /// For Test mode (unlearned cards):
    /// - Correct answer: streak += 1
    /// - Incorrect answer: streak = 0
    ///
    /// For Repeat mode (learned cards):
    /// - Any incorrect answer: streak = 0 (moves back to unlearned)
    /// - All correct: streak remains unchanged (stays learned)
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `target_language` - The target language
    /// * `results` - Test results containing word_name and is_correct
    /// * `is_repeat_mode` - true for Repeat mode, false for Test mode
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all streak updates succeeded
    /// * `Err(CoreError)` - If an error occurs
    pub async fn process_test_results(
        &self,
        username: &str,
        target_language: &str,
        results: Vec<crate::models::TestResult>,
        is_repeat_mode: bool,
    ) -> Result<(), CoreError> {
        let db_path = self.get_db_path(username, target_language);

        for result in results {
            // Get current card to know its current streak
            let card = self
                .repository
                .get_card_by_word_name(db_path.clone(), result.word_name.clone())
                .await?;

            let new_streak = if result.is_correct {
                if is_repeat_mode {
                    // Repeat mode: correct answers don't change streak
                    card.streak
                } else {
                    // Test mode: correct answers increment streak
                    card.streak + 1
                }
            } else {
                // Both modes: incorrect answers reset streak to 0
                0
            };

            // Update the streak
            self.repository
                .update_card_streak(db_path.clone(), result.word_name, new_streak)
                .await?;
        }

        Ok(())
    }
}
