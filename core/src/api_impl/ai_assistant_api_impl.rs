//! AiAssistantApi trait implementation.
//!
//! This module provides the concrete implementation of the AiAssistantApi trait
//! using the ollama_client utilities from the core layer.

use std::future::Future;
use std::pin::Pin;

use lh_api::apis::ai_assistant_api::AiAssistantApi;
use lh_api::errors::api_error::ApiError;
use lh_api::models::ai_explain::{
    ExternalApiRequest, ExternalApiResponse, OllamaGenerateRequest, OllamaGenerateResponse,
};
use lh_api::models::assistant_settings::AssistantSettingsDto;

use crate::services::ollama_client;

/// Implementation of the AiAssistantApi trait.
///
/// This struct uses the ollama_client utilities to check which AI models
/// are currently running in Ollama.
pub struct AiAssistantApiImpl;

impl AiAssistantApiImpl {
    /// Creates a new AiAssistantApiImpl instance.
    ///
    /// # Returns
    ///
    /// A new `AiAssistantApiImpl` instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AiAssistantApiImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl AiAssistantApi for AiAssistantApiImpl {
    fn get_running_models(&self) -> Result<Vec<String>, ApiError> {
        let models = ollama_client::get_running_models();
        Ok(models)
    }

    fn stop_model(&self, model_name: &str) -> Result<(), ApiError> {
        ollama_client::stop_model(model_name)
            .map_err(|e| ApiError::internal_error(e))
    }

    fn check_server_status(&self) -> Result<bool, ApiError> {
        ollama_client::check_server_status()
            .map_err(|e| ApiError::internal_error(e))
    }

    fn start_server_and_wait(&self) -> Result<(), ApiError> {
        ollama_client::start_server_and_wait()
            .map_err(|e| ApiError::internal_error(e))
    }

    fn get_available_models(&self) -> Result<Vec<String>, ApiError> {
        let models = ollama_client::get_available_models();
        Ok(models)
    }

    fn pull_model(&self, model_name: &str) -> Result<(), ApiError> {
        ollama_client::pull_model(model_name)
            .map_err(|e| ApiError::internal_error(e))
    }

    fn run_model(&self, model_name: &str) -> Result<(), ApiError> {
        ollama_client::run_model(model_name)
            .map_err(|e| ApiError::internal_error(e))
    }

    fn explain(
        &self,
        assistant_settings: AssistantSettingsDto,
        user_language: String,
        profile_language: String,
        message: String,
    ) -> Pin<Box<dyn Future<Output = Result<String, ApiError>> + Send + '_>> {
        Box::pin(async move {
            // Determine if we're using API mode or local Ollama
            let is_api_mode = assistant_settings
                .ai_model
                .as_ref()
                .map(|m| m.to_lowercase() == "api")
                .unwrap_or(false);

            if is_api_mode {
                // External API mode
                Self::explain_with_external_api(assistant_settings, &user_language, &profile_language, &message)
                    .await
            } else {
                // Local Ollama mode
                Self::explain_with_ollama(assistant_settings, &user_language, &profile_language, &message)
                    .await
            }
        })
    }
}

impl AiAssistantApiImpl {
    /// Builds the comprehensive explain prompt with language learning instructions.
    fn build_explain_prompt(
        user_language: &str,
        profile_language: &str,
        message: &str,
    ) -> String {
        format!(
            r#"You are a language assistant.
Your goal is to explain a phrase or a word in the learner's language.

Input:
- {} — the language in which you must reply
- {} — a word or expression in the foreign language ({})

Instructions:
1. Write your entire answer in {}.
2. Explain the general meaning of "{}" clearly and naturally, as you would to a language learner.
3. Describe the grammatical or morphological structure: what part of speech it is, any important forms, tenses, or patterns.
4. Explain typical usage and context (formal/informal, spoken/written, emotional tone, etc.).
5. Give 2–3 short example sentences in {} using the phrase, each with a translation into {}.
6. Mention common mistakes or nuances learners should watch out for."#,
            user_language,
            message,
            profile_language,
            user_language,
            message,
            profile_language,
            user_language
        )
    }

    /// Explains a phrase using a local Ollama model.
    async fn explain_with_ollama(
        assistant_settings: AssistantSettingsDto,
        user_language: &str,
        profile_language: &str,
        message: &str,
    ) -> Result<String, ApiError> {
        // Get the model name from settings
        let model_name = assistant_settings
            .ai_model
            .ok_or_else(|| ApiError::validation_error("No AI model configured"))?;

        // Map friendly name to Ollama model identifier
        let ollama_model = Self::map_to_ollama_model(&model_name);

        // Build the comprehensive explain prompt
        let full_prompt = Self::build_explain_prompt(user_language, profile_language, message);

        // Create request
        let request = OllamaGenerateRequest {
            model: ollama_model,
            prompt: full_prompt,
            stream: false,
        };

        // Send request to Ollama
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:11434/api/generate")
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::internal_error(format!("Failed to send request: {}", e)))?;

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ApiError::internal_error(format!(
                "Ollama request failed with status {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let ollama_response: OllamaGenerateResponse = response
            .json()
            .await
            .map_err(|e| ApiError::internal_error(format!("Failed to parse response: {}", e)))?;

        Ok(ollama_response.response)
    }

    /// Explains a phrase using an external API.
    async fn explain_with_external_api(
        assistant_settings: AssistantSettingsDto,
        user_language: &str,
        profile_language: &str,
        message: &str,
    ) -> Result<String, ApiError> {
        // Get API configuration
        let api_endpoint = assistant_settings
            .api_endpoint
            .ok_or_else(|| ApiError::validation_error("No API endpoint configured"))?;
        let api_key = assistant_settings
            .api_key
            .ok_or_else(|| ApiError::validation_error("No API key configured"))?;
        let api_model = assistant_settings
            .api_model_name
            .ok_or_else(|| ApiError::validation_error("No API model name configured"))?;

        // Build the comprehensive explain prompt
        let full_prompt = Self::build_explain_prompt(user_language, profile_language, message);

        // Create request
        let request = ExternalApiRequest {
            model: api_model,
            input: full_prompt,
            stream: false,
        };

        // Send request to external API
        let client = reqwest::Client::new();
        let response = client
            .post(&api_endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::internal_error(format!("Failed to send request: {}", e)))?;

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ApiError::internal_error(format!(
                "External API request failed with status {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let api_response: ExternalApiResponse = response
            .json()
            .await
            .map_err(|e| ApiError::internal_error(format!("Failed to parse response: {}", e)))?;

        // Extract text from nested structure: output[0].content[0].text
        let text = api_response
            .output
            .first()
            .and_then(|output| output.content.first())
            .map(|content| content.text.clone())
            .ok_or_else(|| {
                ApiError::internal_error("Response does not contain expected text content")
            })?;

        Ok(text)
    }

    /// Maps friendly model names to Ollama model identifiers.
    fn map_to_ollama_model(model_name: &str) -> String {
        match model_name.to_lowercase().as_str() {
            "tiny" => "phi3:3.8b-mini-4k-instruct-q4_K_M".to_string(),
            "light" => "phi4".to_string(),
            "weak" => "llama3.2:3b-instruct-q8_0".to_string(),
            "medium" => "qwen2.5:7b-instruct-q5_K_M".to_string(),
            "strong" => "qwen2.5:14b-instruct-q4_K_M".to_string(),
            _ => model_name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_running_models_no_panic() {
        let api = AiAssistantApiImpl::new();
        let result = api.get_running_models();

        assert!(result.is_ok());
        // We can't assert the content since Ollama might or might not be running
        // Just verify it returns without panicking
        let models = result.unwrap();
        assert!(models.is_empty() || !models.is_empty());
    }
}
