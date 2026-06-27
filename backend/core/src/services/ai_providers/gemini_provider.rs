//! Google Gemini API provider implementation.
//!
//! This module provides an implementation of the AiProvider trait
//! for Google's Gemini AI API service.

use std::future::Future;
use std::pin::Pin;

use crate::services::ai_provider::AiProvider;
use lh_api::models::ai_explain::{
    GeminiContent, GeminiGenerateRequest, GeminiGenerateResponse, GeminiPart,
};

/// AI provider for Google Gemini API.
///
/// This provider communicates with Google's Gemini API to generate responses
/// using cloud-hosted models like gemini-pro.
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::ai_providers::GeminiProvider;
/// use lh_core::services::ai_provider::AiProvider;
///
/// # async fn example() {
/// let provider = GeminiProvider::new(
///     "AIza...".to_string(),
///     "gemini-pro".to_string()
/// );
/// match provider.get_response("Hello, world!").await {
///     Ok(response) => println!("Response: {}", response),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// # }
/// ```
pub struct GeminiProvider {
    /// API key for authentication
    api_key: String,
    /// Model name (e.g., "gemini-pro", "gemini-1.5-flash")
    model_name: String,
}

impl GeminiProvider {
    /// Creates a new Gemini provider with the specified credentials.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The Google API key for authentication
    /// * `model_name` - The name of the model to use (e.g., "gemini-pro")
    ///
    /// # Returns
    ///
    /// A new `GeminiProvider` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::services::ai_providers::GeminiProvider;
    ///
    /// let provider = GeminiProvider::new(
    ///     "AIza...".to_string(),
    ///     "gemini-pro".to_string()
    /// );
    /// ```
    pub fn new(api_key: String, model_name: String) -> Self {
        Self {
            api_key,
            model_name,
        }
    }
}

impl AiProvider for GeminiProvider {
    fn get_response<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(async move {
            // Construct Gemini endpoint
            let api_endpoint = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                self.model_name
            );

            // Create Gemini request
            let request = GeminiGenerateRequest {
                contents: vec![GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiPart {
                        text: prompt.to_string(),
                    }],
                }],
            };

            // Send request to Gemini API
            let client = reqwest::Client::new();
            let response = client
                .post(&api_endpoint)
                .header("x-goog-api-key", &self.api_key)
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Failed to send request to Gemini: {}", e))?;

            // Check response status
            if !response.status().is_success() {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(format!(
                    "Gemini API request failed with status {}: {}",
                    status, error_text
                ));
            }

            // Parse response
            let gemini_response: GeminiGenerateResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse Gemini response: {}", e))?;

            // Extract text from first candidate
            let text = gemini_response
                .candidates
                .first()
                .and_then(|candidate| candidate.content.parts.first())
                .map(|part| part.text.clone())
                .ok_or_else(|| {
                    "Gemini response does not contain expected text content".to_string()
                })?;

            Ok(text)
        })
    }

    fn provider_name(&self) -> &str {
        "Gemini"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = GeminiProvider::new("test-key".to_string(), "gemini-pro".to_string());
        assert_eq!(provider.provider_name(), "Gemini");
    }

    #[test]
    fn test_new_provider() {
        let api_key = "AIza-test123".to_string();
        let model_name = "gemini-pro".to_string();
        let provider = GeminiProvider::new(api_key.clone(), model_name.clone());

        assert_eq!(provider.api_key, api_key);
        assert_eq!(provider.model_name, model_name);
    }
}
