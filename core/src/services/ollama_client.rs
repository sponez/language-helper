//! Ollama CLI client for managing running models.

use std::process::Command;

/// Get list of currently running models from Ollama.
///
/// Executes the `ollama ps` command and parses the output.
///
/// # Returns
///
/// Returns a `Vec<String>` containing the names of running models.
/// Returns an empty vector if:
/// - Ollama is not installed
/// - No models are currently active
/// - The command fails to execute
///
/// # Output Format
///
/// The `ollama ps` command outputs in this format:
/// ```text
/// NAME                                    ID              SIZE    PROCESSOR  CONTEXT  UNTIL
/// phi3:3.8b-mini-4k-instruct-q4_K_M      d5ea514251bf    4.8 GB  100% GPU   4096     4 minutes from now
/// ```
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
    // Execute "ollama ps" command
    match Command::new("ollama").arg("ps").output() {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Ollama ps command failed with status: {}", output.status);
                return vec![];
            }

            // Parse the output
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Split into lines and skip the header line
            let lines: Vec<&str> = stdout.lines().collect();
            if lines.len() <= 1 {
                // Only header or empty output
                return vec![];
            }

            // Extract model names from each line (first column)
            lines.iter()
                .skip(1) // Skip header
                .filter_map(|line| {
                    // Split by whitespace and get first column (model name)
                    line.split_whitespace().next().map(|s| s.to_string())
                })
                .filter(|name| !name.is_empty())
                .collect()
        }
        Err(e) => {
            eprintln!("Failed to execute ollama ps: {:?}", e);
            vec![]
        }
    }
}

/// Stop a running model in Ollama.
///
/// Executes the `ollama stop <model_name>` command.
///
/// # Arguments
///
/// * `model_name` - The name of the model to stop (e.g., "qwen2.5:7b-instruct-q5_K_M")
///
/// # Returns
///
/// Returns `Ok(())` if the model was stopped successfully.
/// Returns `Err(String)` if the command failed.
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::ollama_client;
///
/// match ollama_client::stop_model("qwen2.5:7b-instruct-q5_K_M") {
///     Ok(_) => println!("Model stopped successfully"),
///     Err(e) => eprintln!("Failed to stop model: {}", e),
/// }
/// ```
pub fn stop_model(model_name: &str) -> Result<(), String> {
    // Execute "ollama stop <model_name>" command
    match Command::new("ollama")
        .arg("stop")
        .arg(model_name)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to stop model: {}", stderr.trim()))
            }
        }
        Err(e) => {
            Err(format!("Failed to execute ollama stop: {}", e))
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
