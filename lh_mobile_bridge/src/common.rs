//! Common FFI utilities and helpers.
//!
//! This module provides shared functionality for all FFI modules including:
//! - Global state management (AppApi, Tokio runtime)
//! - Response formatting (success/error JSON)
//! - Macros for reducing boilerplate

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Arc, Once};

use lh_api::app_api::AppApi;
use tokio::runtime::Runtime;

/// Global application API instance
pub static mut APP_API: Option<Arc<dyn AppApi>> = None;

/// Global Tokio runtime for async operations
pub static mut RUNTIME: Option<Runtime> = None;

/// Initialization guard
pub static INIT: Once = Once::new();

/// Create a success JSON response
pub fn success_response<T: serde::Serialize>(data: &T) -> *const c_char {
    let json = match serde_json::to_string(data) {
        Ok(j) => j,
        Err(e) => return error_response(&format!("Serialization error: {}", e)),
    };

    match CString::new(json) {
        Ok(c_str) => c_str.into_raw(),
        Err(e) => error_response(&format!("CString creation error: {}", e)),
    }
}

/// Create an error JSON response
pub fn error_response(message: &str) -> *const c_char {
    let error = serde_json::json!({
        "error": message
    });

    let json = serde_json::to_string(&error).unwrap_or_else(|_| {
        r#"{"error": "Failed to serialize error message"}"#.to_string()
    });

    CString::new(json)
        .unwrap_or_else(|_| CString::new(r#"{"error": "Fatal error"}"#).unwrap())
        .into_raw()
}

/// Helper to extract a string from a C pointer
pub unsafe fn c_str_to_rust(ptr: *const c_char, field_name: &str) -> Result<String, *const c_char> {
    match CStr::from_ptr(ptr).to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(error_response(&format!("Invalid {} string: {}", field_name, e))),
    }
}

/// Helper macro to wrap async API calls with error handling
#[macro_export]
macro_rules! ffi_call {
    ($expr:expr) => {
        unsafe {
            let _api = match $crate::common::APP_API.as_ref() {
                Some(api) => api,
                None => return $crate::common::error_response("App not initialized. Call init_app() first."),
            };

            let runtime = match $crate::common::RUNTIME.as_ref() {
                Some(rt) => rt,
                None => return $crate::common::error_response("Runtime not initialized"),
            };

            match runtime.block_on($expr) {
                Ok(data) => $crate::common::success_response(&data),
                Err(e) => $crate::common::error_response(&format!("{}", e)),
            }
        }
    };
}
