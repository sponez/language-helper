//! Data models for AI explanation API.

use serde::{Deserialize, Serialize};

// ============================================================================
// Ollama Models
// ============================================================================

/// Request body for Ollama generate API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaGenerateRequest {
    /// Model name (e.g., "llama3.2", "qwen2.5:7b-instruct-q5_K_M")
    pub model: String,
    /// The prompt/question to send to the model
    pub prompt: String,
    /// Whether to stream the response (always false for our use case)
    pub stream: bool,
}

/// Response from Ollama generate API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaGenerateResponse {
    /// Model name that generated the response
    pub model: String,
    /// Timestamp when the response was created
    pub created_at: String,
    /// The generated text response
    pub response: String,
    /// Whether the generation is complete
    pub done: bool,
    /// Context vector (can be ignored for our use case)
    #[serde(default)]
    pub context: Vec<i64>,
    /// Total duration in nanoseconds
    #[serde(default)]
    pub total_duration: i64,
    /// Model load duration in nanoseconds
    #[serde(default)]
    pub load_duration: i64,
    /// Number of tokens in the prompt
    #[serde(default)]
    pub prompt_eval_count: i32,
    /// Duration of prompt evaluation in nanoseconds
    #[serde(default)]
    pub prompt_eval_duration: i64,
    /// Number of tokens in the response
    #[serde(default)]
    pub eval_count: i32,
    /// Duration of response generation in nanoseconds
    #[serde(default)]
    pub eval_duration: i64,
}

// ============================================================================
// External API Models
// ============================================================================

/// Request body for external API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalApiRequest {
    /// Model name (e.g., "gpt-4.1")
    pub model: String,
    /// The input/question to send to the model
    pub input: String,
    /// Whether to stream the response (always false for our use case)
    pub stream: bool,
}

/// Response from external API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalApiResponse {
    /// Unique response ID
    pub id: String,
    /// Object type (usually "response")
    pub object: String,
    /// Status of the response
    pub status: String,
    /// Array of output messages
    pub output: Vec<ExternalApiOutput>,
    // All other fields are ignored using flatten + default
    #[serde(flatten)]
    #[serde(default)]
    pub _ignored: serde_json::Value,
}

/// Output message from external API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalApiOutput {
    /// Message ID
    pub id: String,
    /// Type of output (usually "message" or "reasoning")
    #[serde(rename = "type")]
    pub output_type: String,
    /// Status of the message (optional, not present in reasoning objects)
    #[serde(default)]
    pub status: Option<String>,
    /// Array of content items (optional, not present in reasoning objects)
    #[serde(default)]
    pub content: Vec<ExternalApiContent>,
    /// Role (optional, usually "assistant" in messages)
    #[serde(default)]
    pub role: Option<String>,
    // All other fields are ignored
    #[serde(flatten)]
    #[serde(default)]
    pub _ignored: serde_json::Value,
}

/// Content item from external API output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalApiContent {
    /// Type of content (usually "output_text")
    #[serde(rename = "type")]
    pub content_type: String,
    /// The actual text content
    pub text: String,
    /// Annotations (can be ignored)
    #[serde(default)]
    pub annotations: Vec<serde_json::Value>,
    // All other fields are ignored
    #[serde(flatten)]
    #[serde(default)]
    pub _ignored: serde_json::Value,
}
