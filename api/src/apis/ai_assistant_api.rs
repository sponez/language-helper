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

    /// Checks if the Ollama server is running and responding.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the server is running and responding.
    /// Returns `Ok(false)` if the server is not running.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a system-level failure during the check.
    fn check_server_status(&self) -> Result<bool, ApiError>;

    /// Starts the Ollama server and waits for it to be ready.
    ///
    /// This operation blocks until the server is ready or a timeout occurs (30 seconds).
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the server started successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the server failed to start or timeout occurred.
    fn start_server_and_wait(&self) -> Result<(), ApiError>;

    /// Gets the list of available (downloaded) models.
    ///
    /// # Returns
    ///
    /// A `Vec<String>` containing the names of available models.
    /// Returns an empty vector if no models are installed.
    ///
    /// # Errors
    ///
    /// Returns an error only if there's a system-level failure during the check.
    fn get_available_models(&self) -> Result<Vec<String>, ApiError>;

    /// Pulls (downloads) a model from the Ollama registry.
    ///
    /// This operation blocks until the download completes, which can take several minutes.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The name of the model to pull (e.g., "phi3:3.8b-mini-4k-instruct-q4_K_M")
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the model was pulled successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the pull operation failed.
    fn pull_model(&self, model_name: &str) -> Result<(), ApiError>;

    /// Runs (starts) a model and waits for it to be ready.
    ///
    /// This operation blocks until the model is verified as running (up to 10 seconds).
    ///
    /// # Arguments
    ///
    /// * `model_name` - The name of the model to run (e.g., "phi3:3.8b-mini-4k-instruct-q4_K_M")
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the model started successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the model failed to start or timeout occurred.
    fn run_model(&self, model_name: &str) -> Result<(), ApiError>;
}
