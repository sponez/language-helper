//! AI Assistant API.
//!
//! This module provides the trait definition for AI assistant management operations.

use crate::errors::api_error::ApiError;

/// API for managing AI assistants and checking running models.
///
/// This trait defines the interface for operations related to querying
/// which AI models are currently running in Ollama.
pub trait AiAssistantApi: Send + Sync {
    /// Gets the list of currently running AI models.
    ///
    /// Makes a request to Ollama to check which models are currently active.
    ///
    /// # Returns
    ///
    /// A `Vec<String>` containing the names of running models.
    /// Returns an empty vector if:
    /// - Ollama is not running
    /// - No models are currently active
    ///
    /// # Errors
    ///
    /// Returns an error only if there's a system-level failure during the check.
    fn get_running_models(&self) -> Result<Vec<String>, ApiError>;

    /// Stops a running AI model in Ollama.
    ///
    /// Makes a POST request to Ollama to stop the specified model.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The name of the model to stop (e.g., "qwen2.5:7b-instruct-q5_K_M")
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the model was stopped successfully or if Ollama is not running.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails for reasons other than Ollama not being available.
    fn stop_model(&self, model_name: &str) -> Result<(), ApiError>;
}
