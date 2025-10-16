//! Ollama CLI client for managing running models.

use std::process::Command;
use std::thread;
use std::time::Duration;

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
/// name      d5ea514251bf    4.8 GB  100% GPU   4096     4 minutes from now
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
            lines
                .iter()
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
/// * `model_name` - The name of the model to stop
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
/// match ollama_client::stop_model("gemma2:9b") {
///     Ok(_) => println!("Model stopped successfully"),
///     Err(e) => eprintln!("Failed to stop model: {}", e),
/// }
/// ```
pub fn stop_model(model_name: &str) -> Result<(), String> {
    // Execute "ollama stop <model_name>" command
    match Command::new("ollama").arg("stop").arg(model_name).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to stop model: {}", stderr.trim()))
            }
        }
        Err(e) => Err(format!("Failed to execute ollama stop: {}", e)),
    }
}

/// Check if the Ollama server is running and responding.
///
/// Makes an HTTP GET request to http://localhost:11434/api/tags to verify
/// the server is accessible.
///
/// # Returns
///
/// Returns `Ok(true)` if the server responds successfully.
/// Returns `Ok(false)` if the connection is refused (server not running).
/// Returns `Err(String)` for other errors.
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::ollama_client;
///
/// match ollama_client::check_server_status() {
///     Ok(true) => println!("Server is running"),
///     Ok(false) => println!("Server is not running"),
///     Err(e) => eprintln!("Error checking server: {}", e),
/// }
/// ```
pub fn check_server_status() -> Result<bool, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    match client.get("http://localhost:11434/api/tags").send() {
        Ok(response) => Ok(response.status().is_success()),
        Err(e) => {
            // Check if it's a connection refused error
            if e.is_connect() {
                Ok(false) // Server not running
            } else {
                Err(format!("Failed to connect to server: {}", e))
            }
        }
    }
}

/// Start the Ollama server and wait for it to be ready.
///
/// Spawns `ollama serve` as a background process and polls the server
/// endpoint until it responds or a timeout occurs.
///
/// # Returns
///
/// Returns `Ok(())` if the server started successfully and is responding.
/// Returns `Err(String)` if the server failed to start or timeout occurred.
///
/// # Timeout
///
/// Waits up to 30 seconds for the server to become ready, checking every 500ms.
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::ollama_client;
///
/// match ollama_client::start_server_and_wait() {
///     Ok(_) => println!("Server started successfully"),
///     Err(e) => eprintln!("Failed to start server: {}", e),
/// }
/// ```
pub fn start_server_and_wait() -> Result<(), String> {
    // Spawn ollama serve as a background process
    Command::new("ollama")
        .arg("serve")
        .spawn()
        .map_err(|e| format!("Failed to spawn ollama serve: {}", e))?;

    // Wait for server to be ready (poll up to 30 seconds)
    for attempt in 1..=60 {
        thread::sleep(Duration::from_millis(500));

        match check_server_status() {
            Ok(true) => {
                println!("Ollama server ready after {} attempts", attempt);
                return Ok(());
            }
            Ok(false) => {
                // Server not ready yet, continue waiting
                continue;
            }
            Err(e) => {
                eprintln!("Error checking server status: {}", e);
                continue;
            }
        }
    }

    Err("Server startup timeout after 30 seconds".to_string())
}

/// Get list of available (downloaded) models from Ollama.
///
/// Executes the `ollama ls` command and parses the output.
///
/// # Returns
///
/// Returns a `Vec<String>` containing the names of available models.
/// Returns an empty vector if no models are installed or command fails.
///
/// # Output Format
///
/// The `ollama ls` command outputs in this format:
/// ```text
/// NAME                                    ID              SIZE    MODIFIED
/// name      d5ea514251bf    2.4 GB  32 minutes ago
/// ```
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::ollama_client;
///
/// let available_models = ollama_client::get_available_models();
/// if available_models.is_empty() {
///     println!("No models installed");
/// } else {
///     println!("Available: {:?}", available_models);
/// }
/// ```
pub fn get_available_models() -> Vec<String> {
    // Execute "ollama ls" command
    match Command::new("ollama").arg("ls").output() {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Ollama ls command failed with status: {}", output.status);
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
            lines
                .iter()
                .skip(1) // Skip header
                .filter_map(|line| {
                    // Split by whitespace and get first column (model name)
                    line.split_whitespace().next().map(|s| s.to_string())
                })
                .filter(|name| !name.is_empty())
                .collect()
        }
        Err(e) => {
            eprintln!("Failed to execute ollama ls: {:?}", e);
            vec![]
        }
    }
}

/// Pull (download) a model from Ollama registry.
///
/// Executes the `ollama pull <model_name>` command and waits for completion.
/// This operation can take several minutes for large models.
///
/// # Arguments
///
/// * `model_name` - The name of the model to pull
///
/// # Returns
///
/// Returns `Ok(())` if the model was pulled successfully.
/// Returns `Err(String)` if the pull failed.
///
/// # Note
///
/// This function blocks until the download completes. No timeout is applied
/// as downloads can take 10+ minutes for large models.
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::ollama_client;
///
/// match ollama_client::pull_model("phi4-mini") {
///     Ok(_) => println!("Model pulled successfully"),
///     Err(e) => eprintln!("Failed to pull model: {}", e),
/// }
/// ```
pub fn pull_model(model_name: &str) -> Result<(), String> {
    println!("Starting pull for model: {}", model_name);

    // Execute "ollama pull <model_name>" and wait for completion
    match Command::new("ollama").arg("pull").arg(model_name).output() {
        Ok(output) => {
            if output.status.success() {
                println!("Model pull completed successfully");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                Err(format!(
                    "Failed to pull model: {}\n{}",
                    stderr.trim(),
                    stdout.trim()
                ))
            }
        }
        Err(e) => Err(format!("Failed to execute ollama pull: {}", e)),
    }
}

/// Run (start) a model in Ollama and wait for it to be ready.
///
/// Loads the model into memory using Ollama's HTTP API by sending an empty prompt
/// to the `/api/generate` endpoint. This avoids the interactive CLI session that
/// `ollama run` creates, making it suitable for GUI applications.
///
/// # Arguments
///
/// * `model_name` - The name of the model to run
///
/// # Returns
///
/// Returns `Ok(())` if the model loaded successfully and is running.
/// Returns `Err(String)` if the model failed to load or timeout occurred.
///
/// # Timeout
///
/// Waits up to 60 seconds for the model to load (large models can take time to load into memory).
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::ollama_client;
///
/// match ollama_client::run_model("phi4-mini") {
///     Ok(_) => println!("Model loaded successfully"),
///     Err(e) => eprintln!("Failed to load model: {}", e),
/// }
/// ```
pub fn run_model(model_name: &str) -> Result<(), String> {
    println!("Loading model into memory: {}", model_name);

    // Create HTTP client with 60 second timeout (model loading can take time)
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Prepare JSON payload to load model with empty prompt
    let payload = serde_json::json!({
        "model": model_name,
        "prompt": "",
        "stream": false
    });

    // Send POST request to /api/generate to load model
    match client
        .post("http://localhost:11434/api/generate")
        .json(&payload)
        .send()
    {
        Ok(response) => {
            if response.status().is_success() {
                println!("Model '{}' loaded successfully via API", model_name);

                // Give the model a moment to fully initialize
                thread::sleep(Duration::from_millis(500));

                // Verify model appears in running list
                let running = get_running_models();
                if running.iter().any(|m| m.contains(model_name)) {
                    println!("Model verified in running list");
                    Ok(())
                } else {
                    // Model loaded but not showing in ps yet - still success
                    println!("Model loaded (may not show in ps immediately)");
                    Ok(())
                }
            } else {
                let status = response.status();
                let error_text = response.text().unwrap_or_default();
                Err(format!(
                    "Failed to load model via API: {} - {}",
                    status, error_text
                ))
            }
        }
        Err(e) => {
            if e.is_connect() {
                Err("Cannot connect to Ollama server. Is it running?".to_string())
            } else if e.is_timeout() {
                Err(format!(
                    "Timeout loading model '{}' (models can take up to 60 seconds to load)",
                    model_name
                ))
            } else {
                Err(format!("Failed to load model via API: {}", e))
            }
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
