//! System requirements API.
//!
//! This module provides the trait definition for system requirements checking operations.

use crate::{
    errors::api_error::ApiError,
    models::system_requirements::{OllamaStatusDto, SystemCompatibilityDto},
};

/// API for checking system requirements and model compatibility.
///
/// This trait defines the interface for operations related to checking
/// if the current system can run specific AI models.
pub trait SystemRequirementsApi: Send + Sync {
    /// Checks if a specific model can run on the current system.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The name of the model to check (e.g., "Weak", "Medium", "Strong", "API")
    ///
    /// # Returns
    ///
    /// A `SystemCompatibilityDto` containing compatibility information and any missing requirements.
    ///
    /// # Errors
    ///
    /// Returns an error if the system check fails.
    fn check_model_compatibility(&self, model_name: &str) -> Result<SystemCompatibilityDto, ApiError>;

    /// Gets a list of all models that are compatible with the current system.
    ///
    /// # Returns
    ///
    /// A vector of model names that can run on the current system.
    ///
    /// # Errors
    ///
    /// Returns an error if the system check fails.
    fn get_compatible_models(&self) -> Result<Vec<String>, ApiError>;

    /// Checks compatibility for multiple models at once.
    ///
    /// # Arguments
    ///
    /// * `model_names` - A slice of model names to check
    ///
    /// # Returns
    ///
    /// A vector of `SystemCompatibilityDto` for each requested model.
    ///
    /// # Errors
    ///
    /// Returns an error if the system check fails.
    fn check_multiple_models(&self, model_names: &[&str]) -> Result<Vec<SystemCompatibilityDto>, ApiError>;

    /// Checks if Ollama is installed and accessible.
    ///
    /// Runs the "ollama --version" command to verify installation.
    ///
    /// # Returns
    ///
    /// An `OllamaStatusDto` containing installation status, version (if installed),
    /// and a message with installation instructions if not installed.
    ///
    /// # Errors
    ///
    /// Returns an error only if there's a system-level failure (not if Ollama is just missing).
    fn check_ollama_status(&self) -> Result<OllamaStatusDto, ApiError>;
}
