//! Ollama AI provider implementation.
//!
//! This module provides an implementation of the AiProvider trait
//! for local Ollama models.

use std::future::Future;
use std::pin::Pin;

use crate::services::ai_provider::AiProvider;
use lh_api::models::ai_explain::{OllamaGenerateRequest, OllamaGenerateResponse};

/// AI provider for local Ollama models.
///
/// This provider communicates with a local Ollama server running on
/// `http://localhost:11434` to generate responses using locally-hosted models.
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::ai_providers::OllamaProvider;
/// use lh_core::services::ai_provider::AiProvider;
///
/// # async fn example() {
/// let provider = OllamaProvider::new("phi4".to_string());
/// match provider.get_response("Hello, world!").await {
///     Ok(response) => println!("Response: {}", response),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// # }
/// ```
pub struct OllamaProvider {
    /// The Ollama model name (e.g., "phi4", "gemma2:9b")
    model_name: String,
}

impl OllamaProvider {
    /// Creates a new Ollama provider with the specified model.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The name of the Ollama model to use
    ///
    /// # Returns
    ///
    /// A new `OllamaProvider` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::services::ai_providers::OllamaProvider;
    ///
    /// let provider = OllamaProvider::new("phi4".to_string());
    /// ```
    pub fn new(model_name: String) -> Self {
        Self { model_name }
    }

    /// Maps friendly model names to Ollama model identifiers.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The friendly model name
    ///
    /// # Returns
    ///
    /// The corresponding Ollama model identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::services::ai_providers::OllamaProvider;
    ///
    /// let ollama_model = OllamaProvider::map_to_ollama_model("tiny");
    /// assert_eq!(ollama_model, "phi4-mini");
    /// ```
    pub fn map_to_ollama_model(model_name: &str) -> String {
        match model_name.to_lowercase().as_str() {
            "tiny" => "phi4-mini".to_string(),
            "light" => "phi4".to_string(),
            "weak" => "gemma2:2b".to_string(),
            "medium" => "aya:8b".to_string(),
            "strong" => "gemma2:9b".to_string(),
            _ => model_name.to_string(),
        }
    }
}

impl AiProvider for OllamaProvider {
    fn get_response<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(async move {
            // Create request
            let request = OllamaGenerateRequest {
                model: self.model_name.clone(),
                prompt: prompt.to_string(),
                stream: false,
            };

            // Send request to Ollama
            let client = reqwest::Client::new();
            let response = client
                .post("http://localhost:11434/api/generate")
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Failed to send request to Ollama: {}", e))?;

            // Check response status
            if !response.status().is_success() {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(format!(
                    "Ollama request failed with status {}: {}",
                    status, error_text
                ));
            }

            // Parse response
            let ollama_response: OllamaGenerateResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

            Ok(ollama_response.response)
        })
    }

    fn provider_name(&self) -> &str {
        "Ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_to_ollama_model_tiny() {
        assert_eq!(OllamaProvider::map_to_ollama_model("tiny"), "phi4-mini");
        assert_eq!(OllamaProvider::map_to_ollama_model("TINY"), "phi4-mini");
    }

    #[test]
    fn test_map_to_ollama_model_light() {
        assert_eq!(OllamaProvider::map_to_ollama_model("light"), "phi4");
    }

    #[test]
    fn test_map_to_ollama_model_weak() {
        assert_eq!(OllamaProvider::map_to_ollama_model("weak"), "gemma2:2b");
    }

    #[test]
    fn test_map_to_ollama_model_medium() {
        assert_eq!(OllamaProvider::map_to_ollama_model("medium"), "aya:8b");
    }

    #[test]
    fn test_map_to_ollama_model_strong() {
        assert_eq!(OllamaProvider::map_to_ollama_model("strong"), "gemma2:9b");
    }

    #[test]
    fn test_map_to_ollama_model_custom() {
        assert_eq!(
            OllamaProvider::map_to_ollama_model("custom-model:7b"),
            "custom-model:7b"
        );
    }

    #[test]
    fn test_provider_name() {
        let provider = OllamaProvider::new("phi4".to_string());
        assert_eq!(provider.provider_name(), "Ollama");
    }
}
