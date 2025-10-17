//! Factory for creating AI provider instances.
//!
//! This module provides a factory function that creates the appropriate
//! AI provider instance based on assistant settings.

use lh_api::errors::api_error::ApiError;
use lh_api::models::assistant_settings::AssistantSettingsDto;

use crate::services::ai_provider::AiProvider;
use crate::services::ai_providers::{GeminiProvider, OllamaProvider, OpenAiProvider};

/// Creates an AI provider instance based on assistant settings.
///
/// This factory function examines the assistant settings and returns
/// the appropriate provider implementation (Ollama, OpenAI, or Gemini).
///
/// # Arguments
///
/// * `settings` - The assistant settings containing provider and model configuration
///
/// # Returns
///
/// Returns `Ok(Box<dyn AiProvider>)` containing the configured provider.
/// Returns `Err(ApiError)` if the settings are invalid or missing required fields.
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::ai_providers::create_provider;
/// use lh_api::models::assistant_settings::AssistantSettingsDto;
///
/// # async fn example() -> Result<(), lh_api::errors::api_error::ApiError> {
/// let settings = AssistantSettingsDto::new(
///     Some("api".to_string()),
///     Some("gemini".to_string()),
///     Some("AIza-key".to_string()),
///     Some("gemini-pro".to_string()),
/// );
///
/// let provider = create_provider(&settings)?;
/// let response = provider.get_response("Hello!").await;
/// # Ok(())
/// # }
/// ```
///
/// # Provider Selection Logic
///
/// 1. If `ai_model` is "api":
///    - Check `api_provider`:
///      - "gemini" → Creates `GeminiProvider`
///      - "openai" or missing → Creates `OpenAiProvider` (default)
///    - Requires: `api_key` and `api_model_name`
///
/// 2. Otherwise (local Ollama):
///    - Creates `OllamaProvider`
///    - Uses `ai_model` as the model name (mapped to Ollama identifier)
pub fn create_provider(settings: &AssistantSettingsDto) -> Result<Box<dyn AiProvider>, ApiError> {
    // Check if we're using API mode or local Ollama
    let is_api_mode = settings
        .ai_model
        .as_ref()
        .map(|m| m.to_lowercase() == "api")
        .unwrap_or(false);

    if is_api_mode {
        // External API mode - determine which provider
        let api_key = settings
            .api_key
            .clone()
            .ok_or_else(|| ApiError::validation_error("No API key configured"))?;

        let model_name = settings
            .api_model_name
            .clone()
            .ok_or_else(|| ApiError::validation_error("No API model name configured"))?;

        let api_provider = settings.api_provider.as_deref().unwrap_or("openai"); // Default to OpenAI for backward compatibility

        match api_provider.to_lowercase().as_str() {
            "gemini" => Ok(Box::new(GeminiProvider::new(api_key, model_name))),
            "openai" | _ => Ok(Box::new(OpenAiProvider::new(api_key, model_name))),
        }
    } else {
        // Local Ollama mode
        let model_name = settings
            .ai_model
            .clone()
            .ok_or_else(|| ApiError::validation_error("No AI model configured"))?;

        // Map friendly name to Ollama model identifier
        let ollama_model = OllamaProvider::map_to_ollama_model(&model_name);

        Ok(Box::new(OllamaProvider::new(ollama_model)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ollama_provider() {
        let settings = AssistantSettingsDto::new(Some("tiny".to_string()), None, None, None);

        let result = create_provider(&settings);
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "Ollama");
    }

    #[test]
    fn test_create_openai_provider() {
        let settings = AssistantSettingsDto::new(
            Some("api".to_string()),
            Some("openai".to_string()),
            Some("sk-test".to_string()),
            Some("gpt-4".to_string()),
        );

        let result = create_provider(&settings);
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "OpenAI");
    }

    #[test]
    fn test_create_gemini_provider() {
        let settings = AssistantSettingsDto::new(
            Some("api".to_string()),
            Some("gemini".to_string()),
            Some("AIza-test".to_string()),
            Some("gemini-pro".to_string()),
        );

        let result = create_provider(&settings);
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "Gemini");
    }

    #[test]
    fn test_create_provider_missing_api_key() {
        let settings = AssistantSettingsDto::new(
            Some("api".to_string()),
            Some("openai".to_string()),
            None, // Missing API key
            Some("gpt-4".to_string()),
        );

        let result = create_provider(&settings);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_provider_missing_model_name() {
        let settings = AssistantSettingsDto::new(
            Some("api".to_string()),
            Some("openai".to_string()),
            Some("sk-test".to_string()),
            None, // Missing model name
        );

        let result = create_provider(&settings);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_provider_missing_ai_model() {
        let settings = AssistantSettingsDto::new(
            None, // Missing ai_model
            None, None, None,
        );

        let result = create_provider(&settings);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_to_openai_when_provider_missing() {
        let settings = AssistantSettingsDto::new(
            Some("api".to_string()),
            None, // No provider specified
            Some("sk-test".to_string()),
            Some("gpt-4".to_string()),
        );

        let result = create_provider(&settings);
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "OpenAI");
    }
}
