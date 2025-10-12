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
}
