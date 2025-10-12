//! System requirements data transfer objects.

use serde::{Deserialize, Serialize};

/// Ollama installation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStatusDto {
    /// Whether Ollama is installed and accessible
    pub is_installed: bool,
    /// Ollama version if installed (e.g., "0.1.23")
    pub version: Option<String>,
    /// Message to display (installation instructions if not installed)
    pub message: String,
}

/// Individual requirement status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementStatusDto {
    /// Type of requirement (CPU, RAM, VRAM)
    pub requirement_type: String,
    /// Required amount (e.g., "4 cores", "8GB")
    pub required: String,
    /// Available amount (e.g., "8 cores", "16GB")
    pub available: String,
    /// Whether the requirement is met
    pub is_met: bool,
}

/// System compatibility information for an AI model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCompatibilityDto {
    /// Name of the model (Tiny, Light, Weak, Medium, Strong, API)
    pub model_name: String,
    /// Whether the model can run on the current system
    pub is_compatible: bool,
    /// List of missing requirements if not compatible
    pub missing_requirements: Vec<String>,
    /// Detailed status for each requirement
    pub requirement_details: Vec<RequirementStatusDto>,
}

impl SystemCompatibilityDto {
    /// Creates a new SystemCompatibilityDto
    pub fn new(
        model_name: String,
        is_compatible: bool,
        missing_requirements: Vec<String>,
        requirement_details: Vec<RequirementStatusDto>,
    ) -> Self {
        Self {
            model_name,
            is_compatible,
            missing_requirements,
            requirement_details,
        }
    }

    /// Creates a compatible result
    pub fn compatible(model_name: String, requirement_details: Vec<RequirementStatusDto>) -> Self {
        Self {
            model_name,
            is_compatible: true,
            missing_requirements: vec![],
            requirement_details,
        }
    }

    /// Creates an incompatible result with missing requirements
    pub fn incompatible(
        model_name: String,
        missing_requirements: Vec<String>,
        requirement_details: Vec<RequirementStatusDto>,
    ) -> Self {
        Self {
            model_name,
            is_compatible: false,
            missing_requirements,
            requirement_details,
        }
    }
}
