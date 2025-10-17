//! AI provider implementations.
//!
//! This module contains concrete implementations of the AiProvider trait
//! for different AI services (Ollama, OpenAI, Gemini).

pub mod factory;
pub mod gemini_provider;
pub mod ollama_provider;
pub mod openai_provider;

pub use factory::create_provider;
pub use gemini_provider::GeminiProvider;
pub use ollama_provider::OllamaProvider;
pub use openai_provider::OpenAiProvider;
