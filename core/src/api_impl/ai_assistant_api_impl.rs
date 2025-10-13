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
use lh_api::models::card::{CardDto, CardType, MeaningDto, WordDto};

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

    fn fill_card(
        &self,
        assistant_settings: AssistantSettingsDto,
        card_name: String,
        card_type: String,
        user_language: String,
        profile_language: String,
    ) -> Pin<Box<dyn Future<Output = Result<CardDto, ApiError>> + Send + '_>> {
        Box::pin(async move {
            // Determine if we're using API mode or local Ollama
            let is_api_mode = assistant_settings
                .ai_model
                .as_ref()
                .map(|m| m.to_lowercase() == "api")
                .unwrap_or(false);

            if is_api_mode {
                // External API mode
                Self::fill_card_with_external_api(assistant_settings, &card_name, &card_type, &user_language, &profile_language)
                    .await
            } else {
                // Local Ollama mode
                Self::fill_card_with_ollama(assistant_settings, &card_name, &card_type, &user_language, &profile_language)
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

        // Find the first message in the output array (skip reasoning items)
        let message = api_response
            .output
            .iter()
            .find(|output| output.output_type == "message")
            .ok_or_else(|| {
                ApiError::internal_error("Response does not contain a message in output array")
            })?;

        // Extract text from content[0].text
        let text = message
            .content
            .first()
            .map(|content| content.text.clone())
            .ok_or_else(|| {
                ApiError::internal_error("Message does not contain expected text content")
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

    /// Builds the fill card prompt for AI generation.
    fn build_fill_card_prompt(
        card_name: &str,
        card_type: &str,
        user_language: &str,
        profile_language: &str,
    ) -> String {
        format!(
            r#"System:
You are a precise dictionary card builder. Output must be valid JSON only (UTF-8, no comments, no trailing commas), exactly matching the schema below.
Do not add any extra text before or after the JSON.

Task:
Fill a vocabulary card for the given word.

Input variables:
- card_type: "{card_type}"            # "straight" or "reverse"
- user_language: "{user_language}"    # the learner's native/app language
- profile_language: "{profile_language}"  # the target/study language
- card_name: "{card_name}"            # the headword; its language depends on card_type (see rules)

Language rules:
- If card_type == "straight":
  - card_name is in profile_language.
  - meanings[].definition must be in profile_language.
  - meanings[].translated_definition must be in user_language.
  - meanings[].word_translations must be in user_language.
- If card_type == "reverse":
  - card_name is in user_language.
  - meanings[].definition must be in user_language.
  - meanings[].translated_definition must be in profile_language.
  - meanings[].word_translations must be in profile_language.

Content rules:
- Include **all valid and common meanings** of the word.
  Each distinct sense or major usage should appear as a separate object in "meanings".
- Exclude only clearly obsolete, idiomatic, or unrelated senses.
- "word.readings": include transliteration, pronunciation, kana/pinyin/IPA if applicable; otherwise, an empty array.
- Deduplicate identical meanings or translations.
- Use concise, dictionary-style wording.
- If the input word is invalid or meaningless, return `"meanings": []`.
- No examples, notes, or extra fields outside this schema.

Output JSON schema (must match exactly):
{{
  "cardType": "straight",
  "word": {{
    "name": "{card_name}",
    "readings": [
      "string"
    ]
  }},
  "meanings": [
    {{
      "definition": "string",
      "translated_definition": "string",
      "word_translations": [
        "string"
      ]
    }}
  ]
}}"#,
            card_type = card_type,
            user_language = user_language,
            profile_language = profile_language,
            card_name = card_name
        )
    }

    /// Fills a card using a local Ollama model.
    async fn fill_card_with_ollama(
        assistant_settings: AssistantSettingsDto,
        card_name: &str,
        card_type: &str,
        user_language: &str,
        profile_language: &str,
    ) -> Result<CardDto, ApiError> {
        // Get the model name from settings
        let model_name = assistant_settings
            .ai_model
            .ok_or_else(|| ApiError::validation_error("No AI model configured"))?;

        // Map friendly name to Ollama model identifier
        let ollama_model = Self::map_to_ollama_model(&model_name);

        // Build the fill card prompt
        let full_prompt = Self::build_fill_card_prompt(card_name, card_type, user_language, profile_language);

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

        // Parse the JSON response from the AI
        Self::parse_card_from_json(&ollama_response.response)
    }

    /// Fills a card using an external API.
    async fn fill_card_with_external_api(
        assistant_settings: AssistantSettingsDto,
        card_name: &str,
        card_type: &str,
        user_language: &str,
        profile_language: &str,
    ) -> Result<CardDto, ApiError> {
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

        // Build the fill card prompt
        let full_prompt = Self::build_fill_card_prompt(card_name, card_type, user_language, profile_language);

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

        // Find the first message in the output array (skip reasoning items)
        let message = api_response
            .output
            .iter()
            .find(|output| output.output_type == "message")
            .ok_or_else(|| {
                ApiError::internal_error("Response does not contain a message in output array")
            })?;

        // Extract text from content[0].text
        let text = message
            .content
            .first()
            .map(|content| content.text.clone())
            .ok_or_else(|| {
                ApiError::internal_error("Message does not contain expected text content")
            })?;

        // Parse the JSON response from the AI
        Self::parse_card_from_json(&text)
    }

    /// Parses a CardDto from JSON text returned by AI.
    fn parse_card_from_json(json_text: &str) -> Result<CardDto, ApiError> {
        use serde::Deserialize;

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct AiCardResponse {
            card_type: String,
            word: AiWord,
            meanings: Vec<AiMeaning>,
        }

        #[derive(Deserialize)]
        struct AiWord {
            name: String,
            readings: Vec<String>,
        }

        #[derive(Deserialize)]
        struct AiMeaning {
            definition: String,
            translated_definition: String,
            word_translations: Vec<String>,
        }

        // Try to extract JSON if the response contains markdown code blocks
        let json_str = if json_text.contains("```json") {
            // Extract content between ```json and ```
            json_text
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(json_text)
                .trim()
        } else if json_text.contains("```") {
            // Extract content between ``` and ```
            json_text
                .split("```")
                .nth(1)
                .unwrap_or(json_text)
                .trim()
        } else {
            json_text.trim()
        };

        // Parse the AI response
        let ai_response: AiCardResponse = serde_json::from_str(json_str)
            .map_err(|e| ApiError::internal_error(format!("Failed to parse AI JSON response: {}", e)))?;

        // Convert card_type string to CardType enum
        let card_type = match ai_response.card_type.to_lowercase().as_str() {
            "straight" => CardType::Straight,
            "reverse" => CardType::Reverse,
            _ => return Err(ApiError::validation_error(format!(
                "Invalid card_type: {}. Must be 'straight' or 'reverse'",
                ai_response.card_type
            ))),
        };

        // Convert to CardDto
        let word_dto = WordDto {
            name: ai_response.word.name,
            readings: ai_response.word.readings,
        };

        let meanings_dto: Vec<MeaningDto> = ai_response
            .meanings
            .into_iter()
            .map(|m| MeaningDto {
                definition: m.definition,
                translated_definition: m.translated_definition,
                word_translations: m.word_translations,
            })
            .collect();

        Ok(CardDto {
            id: None,
            card_type,
            word: word_dto,
            meanings: meanings_dto,
            streak: 0,
            created_at: chrono::Utc::now().timestamp(),
        })
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
