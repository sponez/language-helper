//! AI assistant data transfer objects.

use serde::{Deserialize, Serialize};

/// Running models information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningModelsDto {
    /// List of currently running model names
    /// Empty if Ollama is not running or no models are active
    pub models: Vec<String>,
}

impl RunningModelsDto {
    /// Creates a new RunningModelsDto with the given models
    pub fn new(models: Vec<String>) -> Self {
        Self { models }
    }

    /// Creates a RunningModelsDto with no running models
    pub fn empty() -> Self {
        Self { models: vec![] }
    }
}
