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
///
/// # Examples
///
/// ```
/// use gui::runtime_util::block_on;
///
/// async fn fetch_data() -> i32 {
///     42
/// }
///
/// // Execute async function from sync context
/// let result = block_on(fetch_data());
/// assert_eq!(result, 42);
/// ```
pub fn block_on<F: Future>(future: F) -> F::Output {
    RUNTIME.block_on(future)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn async_add(a: i32, b: i32) -> i32 {
        a + b
    }

    async fn async_string() -> String {
        "Hello, World!".to_string()
    }

    async fn async_option(value: Option<i32>) -> Option<i32> {
        value.map(|v| v * 2)
    }

    #[test]
    fn test_block_on_simple_future() {
        let result = block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_block_on_async_function() {
        let result = block_on(async_add(10, 32));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_block_on_string_result() {
        let result = block_on(async_string());
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_block_on_option_result() {
        let result = block_on(async_option(Some(21)));
        assert_eq!(result, Some(42));

        let result = block_on(async_option(None));
        assert_eq!(result, None);
    }

    #[test]
    fn test_block_on_multiple_calls() {
        // Test that multiple calls to block_on work correctly
        let result1 = block_on(async { 1 });
        let result2 = block_on(async { 2 });
        let result3 = block_on(async { 3 });

        assert_eq!(result1, 1);
        assert_eq!(result2, 2);
        assert_eq!(result3, 3);
    }

    #[test]
    fn test_block_on_with_tokio_sleep() {
        use std::time::Duration;

        let result = block_on(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            "completed"
        });

        assert_eq!(result, "completed");
    }

    #[test]
    fn test_runtime_reusability() {
        // Ensure the runtime can be reused across multiple operations
        for i in 0..10 {
            let result = block_on(async move { i * 2 });
            assert_eq!(result, i * 2);
        }
    }
}
