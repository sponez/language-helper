//! AI Assistant configuration state for a profile.
//!
//! This module provides state for tracking the current AI assistant configuration.
//! If the assistant is not configured or not running, this state should be None.

/// AI Assistant configuration state
///
/// This state contains the assistant configuration loaded from the database.
/// The `is_started` flag indicates if the assistant is currently running:
/// - For Ollama mode: true if the model is actively running
/// - For API mode: always true (API is assumed available)
#[derive(Debug, Clone)]
pub struct AssistantState {
    /// Assistant type: "Ollama" or "API"
    pub assistant_type: String,
    /// Model name (e.g., "phi4", "gemma2:9b", "gpt-4")
    pub model_name: String,
    /// Optional API key (for API mode only)
    pub api_key: Option<String>,
    /// Whether the assistant is currently running/available
    pub is_started: bool,
}

impl AssistantState {
    /// Creates a new AssistantState for Ollama mode.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The Ollama model name
    /// * `is_started` - Whether the Ollama model is currently running
    ///
    /// # Returns
    ///
    /// A new AssistantState configured for Ollama
    pub fn new_ollama(model_name: String, is_started: bool) -> Self {
        Self {
            assistant_type: "Ollama".to_string(),
            model_name,
            api_key: None,
            is_started,
        }
    }

    /// Creates a new AssistantState for API mode.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The API model name (e.g., "gpt-4")
    /// * `api_key` - Optional API key for authentication
    ///
    /// # Returns
    ///
    /// A new AssistantState configured for API (always started)
    pub fn new_api(model_name: String, api_key: Option<String>) -> Self {
        Self {
            assistant_type: "API".to_string(),
            model_name,
            api_key,
            is_started: true, // API mode is always considered started
        }
    }

    /// Returns true if this is API mode
    pub fn is_api_mode(&self) -> bool {
        self.assistant_type.to_lowercase() == "api"
    }

    /// Returns true if this is Ollama mode
    pub fn is_ollama_mode(&self) -> bool {
        self.assistant_type.to_lowercase() == "ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assistant_state_ollama() {
        let state = AssistantState::new_ollama("phi4".to_string(), true);

        assert_eq!(state.assistant_type, "Ollama");
        assert_eq!(state.model_name, "phi4");
        assert!(state.api_key.is_none());
        assert!(state.is_started);
        assert!(state.is_ollama_mode());
        assert!(!state.is_api_mode());
    }

    #[test]
    fn test_assistant_state_api_with_key() {
        let state = AssistantState::new_api("gpt-4".to_string(), Some("sk-123".to_string()));

        assert_eq!(state.assistant_type, "API");
        assert_eq!(state.model_name, "gpt-4");
        assert_eq!(state.api_key, Some("sk-123".to_string()));
        assert!(state.is_started); // API mode always started
        assert!(state.is_api_mode());
        assert!(!state.is_ollama_mode());
    }

    #[test]
    fn test_assistant_state_api_without_key() {
        let state = AssistantState::new_api("gpt-4".to_string(), None);

        assert_eq!(state.assistant_type, "API");
        assert!(state.api_key.is_none());
        assert!(state.is_api_mode());
    }

    #[test]
    fn test_is_api_mode_case_insensitive() {
        let mut state = AssistantState::new_api("gpt-4".to_string(), None);

        state.assistant_type = "api".to_string();
        assert!(state.is_api_mode());

        state.assistant_type = "API".to_string();
        assert!(state.is_api_mode());

        state.assistant_type = "Api".to_string();
        assert!(state.is_api_mode());
    }

    #[test]
    fn test_is_ollama_mode_case_insensitive() {
        let mut state = AssistantState::new_ollama("phi4".to_string(), true);

        state.assistant_type = "ollama".to_string();
        assert!(state.is_ollama_mode());

        state.assistant_type = "OLLAMA".to_string();
        assert!(state.is_ollama_mode());

        state.assistant_type = "Ollama".to_string();
        assert!(state.is_ollama_mode());
    }

    #[test]
    fn test_assistant_state_clone() {
        let state = AssistantState::new_api("gpt-4".to_string(), Some("sk-123".to_string()));
        let cloned = state.clone();

        assert_eq!(state.assistant_type, cloned.assistant_type);
        assert_eq!(state.model_name, cloned.model_name);
        assert_eq!(state.api_key, cloned.api_key);
    }

    #[test]
    fn test_assistant_state_debug() {
        let state = AssistantState::new_ollama("phi4".to_string(), true);
        let debug_str = format!("{:?}", state);

        assert!(debug_str.contains("AssistantState"));
        assert!(debug_str.contains("phi4"));
        assert!(debug_str.contains("Ollama"));
    }
}
