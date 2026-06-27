//! Profile state aggregating all profile-related data.
//!
//! This module provides the top-level state for a learning profile,
//! including profile metadata and nested card/assistant states.

use super::{AssistantState, CardState};

/// Top-level profile state
///
/// Aggregates all profile-related data including profile metadata,
/// card statistics, and assistant configuration.
#[derive(Debug, Clone)]
pub struct ProfileState {
    /// Profile name (user-defined identifier)
    pub profile_name: String,
    /// Target language being learned
    pub target_language: String,
    /// Card statistics (None until loaded asynchronously)
    pub card_state: Option<CardState>,
    /// Assistant configuration (None if not configured/running)
    pub assistant_state: Option<AssistantState>,
}

impl ProfileState {
    /// Creates a new ProfileState with minimal initialization.
    ///
    /// Card and assistant states start as None and will be loaded asynchronously.
    ///
    /// # Arguments
    ///
    /// * `profile_name` - The profile name
    /// * `target_language` - The target language code
    ///
    /// # Returns
    ///
    /// A new ProfileState with unloaded nested states
    pub fn new(profile_name: String, target_language: String) -> Self {
        Self {
            profile_name,
            target_language,
            card_state: None,
            assistant_state: None,
        }
    }

    /// Returns true if all required async data has been loaded.
    ///
    /// This means:
    /// - Card state has been loaded (even if empty)
    /// - Assistant state has been checked (Some means configured, None means not configured)
    ///
    /// Note: assistant_state can remain None (not an error condition),
    /// but we need to wait for the check to complete before showing settings.
    pub fn is_fully_loaded(&self) -> bool {
        self.card_state.is_some()
    }

    /// Returns true if the assistant is available (configured and running)
    ///
    /// AI button should be enabled only when this returns true.
    pub fn has_assistant(&self) -> bool {
        self.assistant_state
            .as_ref()
            .is_some_and(|state| state.is_started)
    }

    /// Returns true if assistant configuration exists (even if not started)
    ///
    /// This is used for settings - we can configure assistant even if not running.
    pub fn has_assistant_config(&self) -> bool {
        self.assistant_state.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_state_new() {
        let state = ProfileState::new("My Spanish".to_string(), "spanish".to_string());

        assert_eq!(state.profile_name, "My Spanish");
        assert_eq!(state.target_language, "spanish");
        assert!(state.card_state.is_none());
        assert!(state.assistant_state.is_none());
    }

    #[test]
    fn test_is_fully_loaded_false_initially() {
        let state = ProfileState::new("My Spanish".to_string(), "spanish".to_string());
        assert!(!state.is_fully_loaded());
    }

    #[test]
    fn test_is_fully_loaded_true_with_card_state() {
        let mut state = ProfileState::new("My Spanish".to_string(), "spanish".to_string());
        state.card_state = Some(CardState::new(10, "manual".to_string(), 5));

        assert!(state.is_fully_loaded());
    }

    #[test]
    fn test_is_fully_loaded_true_without_assistant() {
        let mut state = ProfileState::new("My Spanish".to_string(), "spanish".to_string());
        state.card_state = Some(CardState::new(10, "manual".to_string(), 5));
        // assistant_state remains None - this is OK

        assert!(state.is_fully_loaded());
    }

    #[test]
    fn test_has_assistant_false_when_none() {
        let state = ProfileState::new("My Spanish".to_string(), "spanish".to_string());
        assert!(!state.has_assistant());
        assert!(!state.has_assistant_config());
    }

    #[test]
    fn test_has_assistant_true_when_started() {
        let mut state = ProfileState::new("My Spanish".to_string(), "spanish".to_string());
        state.assistant_state = Some(AssistantState::new_ollama("phi4".to_string(), true));

        assert!(state.has_assistant()); // Running
        assert!(state.has_assistant_config()); // Configured
    }

    #[test]
    fn test_has_assistant_false_when_not_started() {
        let mut state = ProfileState::new("My Spanish".to_string(), "spanish".to_string());
        state.assistant_state = Some(AssistantState::new_ollama("phi4".to_string(), false));

        assert!(!state.has_assistant()); // Not running
        assert!(state.has_assistant_config()); // But configured
    }

    #[test]
    fn test_profile_state_clone() {
        let mut state = ProfileState::new("My Spanish".to_string(), "spanish".to_string());
        state.card_state = Some(CardState::new(10, "manual".to_string(), 5));
        state.assistant_state = Some(AssistantState::new_ollama("phi4".to_string(), true));

        let cloned = state.clone();

        assert_eq!(state.profile_name, cloned.profile_name);
        assert_eq!(state.target_language, cloned.target_language);
        assert!(cloned.card_state.is_some());
        assert!(cloned.assistant_state.is_some());
    }

    #[test]
    fn test_profile_state_debug() {
        let state = ProfileState::new("My Spanish".to_string(), "spanish".to_string());
        let debug_str = format!("{:?}", state);

        assert!(debug_str.contains("ProfileState"));
        assert!(debug_str.contains("My Spanish"));
        assert!(debug_str.contains("spanish"));
    }
}
