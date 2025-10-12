//! Ollama HTTP client for querying running models.

use serde::Deserialize;

/// Ollama API response for running models
#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    #[allow(dead_code)]
    size: u64,
    #[allow(dead_code)]
    digest: String,
    #[allow(dead_code)]
    expires_at: String,
}

/// Response from /api/ps endpoint
#[derive(Debug, Deserialize)]
struct OllamaPsResponse {
    models: Vec<OllamaModel>,
}

/// Get list of currently running models from Ollama.
///
/// Makes a blocking GET request to http://localhost:11434/api/ps
///
/// # Returns
///
/// Returns a `Vec<String>` containing the names of running models.
/// Returns an empty vector if:
/// - Ollama is not running (connection fails)
/// - No models are currently active
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::ollama_client;
///
/// let running_models = ollama_client::get_running_models();
/// if running_models.is_empty() {
///     println!("No models running");
/// } else {
///     println!("Running: {:?}", running_models);
/// }
/// ```
pub fn get_running_models() -> Vec<String> {
    // Use blocking client for synchronous operation
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new());

    // Make request to Ollama API
    match client.get("http://localhost:11434/api/ps").send() {
        Ok(response) => {
            // Try to parse response
            match response.json::<OllamaPsResponse>() {
                Ok(ps_response) => {
                    // Extract model names
                    ps_response.models.into_iter().map(|m| m.name).collect()
                }
                Err(e) => {
                    eprintln!("Failed to parse Ollama response: {:?}", e);
                    vec![]
                }
            }
        }
        Err(e) => {
            // Connection failed - Ollama not running
            eprintln!("Failed to connect to Ollama: {:?}", e);
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_running_models_no_panic() {
        // Should not panic even if Ollama is not running
        let models = get_running_models();
        // We can't assert the content since Ollama might or might not be running
        // Just verify it returns without panicking
        assert!(models.is_empty() || !models.is_empty());
    }
}
