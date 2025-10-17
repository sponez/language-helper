//! OpenAI API provider implementation.
//!
//! This module provides an implementation of the AiProvider trait
//! for OpenAI's API service.

use std::future::Future;
use std::pin::Pin;

use crate::services::ai_provider::AiProvider;
use lh_api::models::ai_explain::{ExternalApiRequest, ExternalApiResponse};

/// AI provider for OpenAI API.
///
/// This provider communicates with OpenAI's API to generate responses
/// using cloud-hosted models like GPT-4.
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::ai_providers::OpenAiProvider;
/// use lh_core::services::ai_provider::AiProvider;
///
/// # async fn example() {
/// let provider = OpenAiProvider::new(
///     "sk-...".to_string(),
///     "gpt-4".to_string()
/// );
/// match provider.get_response("Hello, world!").await {
///     Ok(response) => println!("Response: {}", response),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// # }
/// ```
pub struct OpenAiProvider {
    /// API key for authentication
    api_key: String,
    /// Model name (e.g., "gpt-4", "gpt-3.5-turbo")
    model_name: String,
}

impl OpenAiProvider {
    /// Creates a new OpenAI provider with the specified credentials.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The OpenAI API key for authentication
    /// * `model_name` - The name of the model to use (e.g., "gpt-4")
    ///
    /// # Returns
    ///
    /// A new `OpenAiProvider` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::services::ai_providers::OpenAiProvider;
    ///
    /// let provider = OpenAiProvider::new(
    ///     "sk-...".to_string(),
    ///     "gpt-4".to_string()
    /// );
    /// ```
    pub fn new(api_key: String, model_name: String) -> Self {
        Self {
            api_key,
            model_name,
        }
    }
}

impl AiProvider for OpenAiProvider {
    fn get_response<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(async move {
            let api_endpoint = "https://api.openai.com/v1/responses";

            // Create request
            let request = ExternalApiRequest {
                model: self.model_name.clone(),
                input: prompt.to_string(),
                stream: false,
            };

            // Send request to OpenAI API
            let client = reqwest::Client::new();
            let response = client
                .post(api_endpoint)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Failed to send request to OpenAI: {}", e))?;

            // Check response status
            if !response.status().is_success() {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(format!(
                    "OpenAI API request failed with status {}: {}",
                    status, error_text
                ));
            }

            // Parse response
            let api_response: ExternalApiResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

            // Find the first message in the output array (skip reasoning items)
            let message = api_response
                .output
                .iter()
                .find(|output| output.output_type == "message")
                .ok_or_else(|| {
                    "OpenAI response does not contain a message in output array".to_string()
                })?;

            // Extract text from content[0].text
            let text = message
                .content
                .first()
                .map(|content| content.text.clone())
                .ok_or_else(|| {
                    "OpenAI message does not contain expected text content".to_string()
                })?;

            Ok(text)
        })
    }

    fn provider_name(&self) -> &str {
        "OpenAI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = OpenAiProvider::new("test-key".to_string(), "gpt-4".to_string());
        assert_eq!(provider.provider_name(), "OpenAI");
    }

    #[test]
    fn test_new_provider() {
        let api_key = "sk-test123".to_string();
        let model_name = "gpt-4".to_string();
        let provider = OpenAiProvider::new(api_key.clone(), model_name.clone());

        assert_eq!(provider.api_key, api_key);
        assert_eq!(provider.model_name, model_name);
    }
}
