//! AiAssistantApi trait implementation.
//!
//! This module provides the concrete implementation of the AiAssistantApi trait
//! using the ollama_client utilities from the core layer.

use lh_api::apis::ai_assistant_api::AiAssistantApi;
use lh_api::errors::api_error::ApiError;

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
