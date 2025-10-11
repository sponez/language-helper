//! Utility for running async code from synchronous GUI contexts.
//!
//! This module provides a shared Tokio runtime for executing async operations
//! from synchronous GUI code (routers, view methods, etc.).

use once_cell::sync::Lazy;
use std::future::Future;
use tokio::runtime::Runtime;

/// Shared Tokio runtime for GUI async operations
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

/// Execute an async future in the shared runtime and block until completion.
///
/// This function is designed to be called from synchronous GUI code that needs
/// to execute async API calls. It uses a shared Tokio runtime to avoid the
/// overhead of creating a new runtime for each call.
///
/// # Arguments
///
/// * `future` - The async future to execute
///
/// # Returns
///
/// The result of the future
pub fn block_on<F: Future>(future: F) -> F::Output {
    RUNTIME.block_on(future)
}
