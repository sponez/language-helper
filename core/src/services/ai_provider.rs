//! AI Provider trait for abstracting different AI service implementations.
//!
//! This module defines the core trait that all AI providers must implement,
//! allowing for a unified interface to interact with different AI services
//! (Ollama, OpenAI, Gemini, etc.).

use std::future::Future;
use std::pin::Pin;

/// Trait for AI service providers.
///
/// All AI providers (Ollama, OpenAI, Gemini, etc.) must implement this trait
/// to provide a consistent interface for generating responses.
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::ai_provider::AiProvider;
///
/// async fn get_ai_response(provider: &dyn AiProvider, prompt: &str) -> Result<String, String> {
///     provider.get_response(prompt).await
/// }
/// ```
pub trait AiProvider: Send + Sync {
    /// Generates a response from the AI provider given a prompt.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The input prompt/message to send to the AI
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` containing the AI's response text.
    /// Returns `Err(String)` if the request fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::ai_provider::AiProvider;
    /// # async fn example(provider: &dyn AiProvider) {
    /// let prompt = "Explain the word 'hello'";
    /// match provider.get_response(prompt).await {
    ///     Ok(response) => println!("AI Response: {}", response),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// # }
    /// ```
    fn get_response<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>>;

    /// Returns the name of this provider (e.g., "Ollama", "OpenAI", "Gemini").
    ///
    /// # Returns
    ///
    /// A string slice identifying the provider.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::ai_provider::AiProvider;
    /// # fn example(provider: &dyn AiProvider) {
    /// println!("Using provider: {}", provider.provider_name());
    /// # }
    /// ```
    fn provider_name(&self) -> &str;
}
