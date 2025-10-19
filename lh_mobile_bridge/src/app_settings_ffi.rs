//! App Settings API FFI functions.
//!
//! This module provides FFI wrappers for global application settings operations.

use std::os::raw::c_char;

use crate::common::{c_str_to_rust, APP_API};
use crate::ffi_call;

/// Get global application settings
///
/// # Returns
///
/// JSON string: `{"theme": "Dark", "language": "en-US"}` or `{"error": "message"}`
///
/// # Safety
///
/// This function accesses global mutable statics.
#[no_mangle]
pub extern "C" fn get_app_settings() -> *const c_char {
    ffi_call!(APP_API.as_ref().unwrap().app_settings_api().get_app_settings())
}

/// Update global theme setting
///
/// # Arguments
///
/// * `theme` - Null-terminated C string containing the theme ("Dark" or "Light")
///
/// # Returns
///
/// JSON string: `{"success": true}` or `{"error": "message"}`
///
/// # Safety
///
/// This function dereferences a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn update_app_theme(theme: *const c_char) -> *const c_char {
    let theme_str = match c_str_to_rust(theme, "theme") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .app_settings_api()
        .update_app_theme(&theme_str))
}

/// Update global language setting
///
/// # Arguments
///
/// * `language` - Null-terminated C string containing the language code (e.g., "en-US")
///
/// # Returns
///
/// JSON string: `{"success": true}` or `{"error": "message"}`
///
/// # Safety
///
/// This function dereferences a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn update_app_language(language: *const c_char) -> *const c_char {
    let language_str = match c_str_to_rust(language, "language") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .app_settings_api()
        .update_app_language(&language_str))
}
