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
use lh_api::models::card::CardDto;

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
    fn get_running_models(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, ApiError>> + Send + '_>> {
        Box::pin(async {
            let models = ollama_client::get_running_models().await;
            Ok(models)
        })
    }

    fn stop_model(
        &self,
        model_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        let model_name = model_name.to_string();
        Box::pin(async move {
            ollama_client::stop_model(&model_name)
                .await
                .map_err(|e| ApiError::internal_error(e))
        })
    }

    fn check_server_status(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<bool, ApiError>> + Send + '_>> {
        Box::pin(async {
            ollama_client::check_server_status()
                .await
                .map_err(|e| ApiError::internal_error(e))
        })
    }

    fn start_server_and_wait(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async {
            ollama_client::start_server_and_wait()
                .await
                .map_err(|e| ApiError::internal_error(e))
        })
    }

    fn get_available_models(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, ApiError>> + Send + '_>> {
        Box::pin(async {
            let models = ollama_client::get_available_models().await;
            Ok(models)
        })
    }

    fn pull_model(
        &self,
        model_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        let model_name = model_name.to_string();
        Box::pin(async move {
            ollama_client::pull_model(&model_name)
                .await
                .map_err(|e| ApiError::internal_error(e))
        })
    }

    fn run_model(
        &self,
        model_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        let model_name = model_name.to_string();
        Box::pin(async move {
            ollama_client::run_model(&model_name)
                .await
                .map_err(|e| ApiError::internal_error(e))
        })
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
                Self::explain_with_external_api(
                    assistant_settings,
                    &user_language,
                    &profile_language,
                    &message,
                )
                .await
            } else {
                // Local Ollama mode
                Self::explain_with_ollama(
                    assistant_settings,
                    &user_language,
                    &profile_language,
                    &message,
                )
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
                Self::fill_card_with_external_api(
                    assistant_settings,
                    &card_name,
                    &card_type,
                    &user_language,
                    &profile_language,
                )
                .await
            } else {
                // Local Ollama mode
                Self::fill_card_with_ollama(
                    assistant_settings,
                    &card_name,
                    &card_type,
                    &user_language,
                    &profile_language,
                )
                .await
            }
        })
    }

    fn merge_inverse_cards(
        &self,
        assistant_settings: AssistantSettingsDto,
        new_card: CardDto,
        existing_cards: Vec<CardDto>,
        user_language: String,
        profile_language: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<CardDto>, ApiError>> + Send + '_>> {
        Box::pin(async move {
            // Determine if we're using API mode or local Ollama
            let is_api_mode = assistant_settings
                .ai_model
                .as_ref()
                .map(|m| m.to_lowercase() == "api")
                .unwrap_or(false);

            if is_api_mode {
                // External API mode
                Self::merge_inverse_cards_with_external_api(
                    assistant_settings,
                    new_card,
                    existing_cards,
                    &user_language,
                    &profile_language,
                )
                .await
            } else {
                // Local Ollama mode
                Self::merge_inverse_cards_with_ollama(
                    assistant_settings,
                    new_card,
                    existing_cards,
                    &user_language,
                    &profile_language,
                )
                .await
            }
        })
    }
}

impl AiAssistantApiImpl {
    /// Builds the comprehensive explain prompt with language learning instructions.
    fn build_explain_prompt(user_language: &str, profile_language: &str, message: &str) -> String {
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
            "tiny" => "phi4-mini".to_string(),
            "light" => "phi4".to_string(),
            "weak" => "gemma2:2b".to_string(),
            "medium" => "aya:8b".to_string(),
            "strong" => "gemma2:9b".to_string(),
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
        let card_language: String;
        let target_language: String;
        if card_type == "straight" {
            card_language = profile_language.to_string();
            target_language = user_language.to_string();
        } else {
            card_language = user_language.to_string();
            target_language = profile_language.to_string();
        }

        format!(
            r#"You are a precise dictionary card builder. Output must be valid JSON only (UTF-8, no comments, no trailing commas), exactly matching the schema below.
Do not add any extra text before or after the JSON.

Task:
Fill a vocabulary card for the given word.

Output JSON schema (must match exactly):
{{
  "cardType": "{card_type}",
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
}}

For the word "{card_name}" you must fill all possible readings and meanings. Do not repeat similar meanings.
"readings" is transcriptions, kana, romaji, and more.
Each "meaning" must have a "definition". "definition" is the definition of a word, as in a dictionary in the {card_language} language.
"translated_definition" is a translation of the definition into {target_language} language.
"word_translations" are possible translations in the {target_language} language of the word "{card_name}" for this "definition".

OUTPUT: JSON. NO OTHER WORDS AND EXPLANATIONS"#,
            card_type = card_type,
            card_language = card_language,
            target_language = target_language,
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
        let full_prompt =
            Self::build_fill_card_prompt(card_name, card_type, user_language, profile_language);

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
        let full_prompt =
            Self::build_fill_card_prompt(card_name, card_type, user_language, profile_language);

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
        use lh_api::models::card::{CardType, MeaningDto, WordDto};
        use serde::Deserialize;

        // Simplified structures for lenient parsing
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SimpleCard {
            #[serde(default)]
            card_type: Option<String>,
            word: SimpleWord,
            #[serde(default)]
            meanings: Vec<SimpleMeaning>,
            #[serde(default)]
            streak: i32,
            #[serde(default)]
            created_at: i64,
        }

        #[derive(Deserialize)]
        struct SimpleWord {
            name: String,
            #[serde(default)]
            readings: Vec<String>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SimpleMeaning {
            definition: String,
            #[serde(alias = "translated_definition")]
            translated_definition: String,
            #[serde(alias = "word_translations")]
            #[serde(default)]
            word_translations: Vec<String>,
        }

        // Try to extract JSON if the response contains markdown code blocks
        let json_str = if json_text.contains("```json") {
            json_text
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(json_text)
                .trim()
        } else if json_text.contains("```") {
            json_text.split("```").nth(1).unwrap_or(json_text).trim()
        } else {
            json_text.trim()
        };

        // Parse into simplified structure
        let simple: SimpleCard = serde_json::from_str(json_str).map_err(|e| {
            ApiError::internal_error(format!("Failed to parse AI JSON response: {}", e))
        })?;

        // Convert to CardDto
        let card_type = match simple.card_type.as_deref() {
            Some("straight") | Some("forward") => CardType::Straight,
            Some("reverse") => CardType::Reverse,
            _ => CardType::Straight, // Default
        };

        let word_dto = WordDto {
            name: simple.word.name,
            readings: simple.word.readings,
        };

        let meanings_dto: Vec<MeaningDto> = simple
            .meanings
            .into_iter()
            .map(|m| MeaningDto {
                definition: m.definition,
                translated_definition: m.translated_definition,
                word_translations: m.word_translations,
            })
            .collect();

        Ok(CardDto {
            card_type,
            word: word_dto,
            meanings: meanings_dto,
            streak: simple.streak,
            created_at: simple.created_at,
        })
    }

    /// Builds the merge inverse cards prompt for AI batch merging.
    fn build_merge_inverse_cards_prompt(
        new_card: &CardDto,
        existing_cards: &[CardDto],
        _user_language: &str,
        _profile_language: &str,
    ) -> String {
        // Get the new translation (word name from the new card)
        let new_translation = &new_card.word.name;

        // Serialize existing cards to JSON
        let existing_cards_json =
            serde_json::to_string_pretty(existing_cards).unwrap_or_else(|_| "[]".to_string());

        format!(
            r#"You are a precise dictionary card synchronizer.

You need to add a new translation to EACH of the existing cards, if it's not there yet.
If there is no definition in any sense to which this translation can be attributed, then a new meaning must be added to the list of meanings (add the definition and its translation yourself). 
Nothing else needs to be changed. Don't change the card format.

NEW TRANSLATION: {new_translation}

EXISTING CARDS:
{existing_cards_json}

OUTPUT: JSON array of updated cards (or []). NO OTHER WORDS AND EXPLANATIONS"#,
            new_translation = new_translation,
            existing_cards_json = existing_cards_json
        )
    }

    /// Merges inverse cards using a local Ollama model.
    async fn merge_inverse_cards_with_ollama(
        assistant_settings: AssistantSettingsDto,
        new_card: CardDto,
        existing_cards: Vec<CardDto>,
        user_language: &str,
        profile_language: &str,
    ) -> Result<Vec<CardDto>, ApiError> {
        // Get the model name from settings
        let model_name = assistant_settings
            .ai_model
            .ok_or_else(|| ApiError::validation_error("No AI model configured"))?;

        // Map friendly name to Ollama model identifier
        let ollama_model = Self::map_to_ollama_model(&model_name);

        // Build the merge prompt
        let full_prompt = Self::build_merge_inverse_cards_prompt(
            &new_card,
            &existing_cards,
            user_language,
            profile_language,
        );

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

        // Parse the JSON array response from the AI
        Self::parse_card_array_from_json(&ollama_response.response)
    }

    /// Merges inverse cards using an external API.
    async fn merge_inverse_cards_with_external_api(
        assistant_settings: AssistantSettingsDto,
        new_card: CardDto,
        existing_cards: Vec<CardDto>,
        user_language: &str,
        profile_language: &str,
    ) -> Result<Vec<CardDto>, ApiError> {
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

        // Build the merge prompt
        let full_prompt = Self::build_merge_inverse_cards_prompt(
            &new_card,
            &existing_cards,
            user_language,
            profile_language,
        );

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

        // Parse the JSON array response from the AI
        Self::parse_card_array_from_json(&text)
    }

    /// Parses an array of CardDto from JSON text returned by AI.
    fn parse_card_array_from_json(json_text: &str) -> Result<Vec<CardDto>, ApiError> {
        use lh_api::models::card::{CardType, MeaningDto, WordDto};
        use serde::Deserialize;

        // Simplified structures for lenient parsing
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SimpleCard {
            #[serde(default)]
            card_type: Option<String>,
            word: SimpleWord,
            #[serde(default)]
            meanings: Vec<SimpleMeaning>,
            #[serde(default)]
            streak: i32,
            #[serde(default)]
            created_at: i64,
        }

        #[derive(Deserialize)]
        struct SimpleWord {
            name: String,
            #[serde(default)]
            readings: Vec<String>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SimpleMeaning {
            definition: String,
            #[serde(alias = "translated_definition")]
            translated_definition: String,
            #[serde(alias = "word_translations")]
            #[serde(default)]
            word_translations: Vec<String>,
        }

        // Try to extract JSON if the response contains markdown code blocks
        let json_str = if json_text.contains("```json") {
            json_text
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(json_text)
                .trim()
        } else if json_text.contains("```") {
            json_text.split("```").nth(1).unwrap_or(json_text).trim()
        } else {
            json_text.trim()
        };

        // Parse into simplified structures
        let simple_cards: Vec<SimpleCard> = serde_json::from_str(json_str).map_err(|e| {
            ApiError::internal_error(format!("Failed to parse AI JSON array response: {}", e))
        })?;

        // Convert to Vec<CardDto>
        let cards: Vec<CardDto> = simple_cards
            .into_iter()
            .map(|simple| {
                let card_type = match simple.card_type.as_deref() {
                    Some("straight") | Some("forward") => CardType::Straight,
                    Some("reverse") => CardType::Reverse,
                    _ => CardType::Straight, // Default
                };

                let word_dto = WordDto {
                    name: simple.word.name,
                    readings: simple.word.readings,
                };

                let meanings_dto: Vec<MeaningDto> = simple
                    .meanings
                    .into_iter()
                    .map(|m| MeaningDto {
                        definition: m.definition,
                        translated_definition: m.translated_definition,
                        word_translations: m.word_translations,
                    })
                    .collect();

                CardDto {
                    card_type,
                    word: word_dto,
                    meanings: meanings_dto,
                    streak: simple.streak,
                    created_at: simple.created_at,
                }
            })
            .collect();

        Ok(cards)
    }
}
