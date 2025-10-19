//! Memory management FFI functions.
//!
//! This module provides functions for managing memory allocated by Rust
//! and returned to the FFI caller (e.g., Flutter).

use std::ffi::CString;
use std::os::raw::c_char;

/// Free a string allocated by Rust
///
/// This MUST be called for every string returned by FFI functions to prevent memory leaks.
///
/// # Arguments
///
/// * `ptr` - Pointer to the string to free
///
/// # Safety
///
/// This function is unsafe because it:
/// - Dereferences a raw pointer
/// - Must only be called once per pointer
/// - The pointer must have been allocated by Rust (via CString::into_raw)
///
/// # Examples
///
/// From Dart:
/// ```dart
/// final jsonPtr = getUsernames();
/// final jsonStr = jsonPtr.toDartString();
/// freeString(jsonPtr);  // Important: prevent memory leak!
/// ```
#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *const c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr as *mut c_char);
    }
}
