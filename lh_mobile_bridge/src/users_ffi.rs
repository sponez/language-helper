//! Users API FFI functions.
//!
//! This module provides FFI wrappers for all user management operations.

use std::os::raw::c_char;

use crate::common::{c_str_to_rust, error_response, APP_API};
use crate::ffi_call;

/// Get all usernames
///
/// # Returns
///
/// JSON string: `["username1", "username2", ...]` or `{"error": "message"}`
///
/// # Safety
///
/// This function accesses global mutable statics.
#[no_mangle]
pub extern "C" fn get_usernames() -> *const c_char {
    ffi_call!(APP_API.as_ref().unwrap().users_api().get_usernames())
}

/// Get user by username
///
/// # Arguments
///
/// * `username` - Null-terminated C string containing the username
///
/// # Returns
///
/// JSON string: `{"username": "...", "settings": {...}, "profiles": [...]}` or `{"error": "message"}`
///
/// # Safety
///
/// This function dereferences a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn get_user_by_username(username: *const c_char) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .users_api()
        .get_user_by_username(&username_str))
}

/// Create a new user
///
/// # Arguments
///
/// * `username` - Null-terminated C string containing the username
/// * `language` - Null-terminated C string containing the UI language code (e.g., "en-US")
///
/// # Returns
///
/// JSON string: `{"username": "...", "settings": {...}, "profiles": [...]}` or `{"error": "message"}`
///
/// # Safety
///
/// This function dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn create_user(username: *const c_char, language: *const c_char) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let language_str = match c_str_to_rust(language, "language") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .users_api()
        .create_user(&username_str, &language_str))
}

/// Update user's theme preference
///
/// # Arguments
///
/// * `username` - Null-terminated C string containing the username
/// * `theme` - Null-terminated C string containing the theme ("Dark" or "Light")
///
/// # Returns
///
/// JSON string: `{"success": true}` or `{"error": "message"}`
///
/// # Safety
///
/// This function dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn update_user_theme(username: *const c_char, theme: *const c_char) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let theme_str = match c_str_to_rust(theme, "theme") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .users_api()
        .update_user_theme(&username_str, &theme_str))
}

/// Update user's language preference
///
/// # Arguments
///
/// * `username` - Null-terminated C string containing the username
/// * `language` - Null-terminated C string containing the language code (e.g., "en-US")
///
/// # Returns
///
/// JSON string: `{"success": true}` or `{"error": "message"}`
///
/// # Safety
///
/// This function dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn update_user_language(username: *const c_char, language: *const c_char) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let language_str = match c_str_to_rust(language, "language") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .users_api()
        .update_user_language(&username_str, &language_str))
}

/// Delete a user and all associated data
///
/// # Arguments
///
/// * `username` - Null-terminated C string containing the username
///
/// # Returns
///
/// JSON string: `{"success": true}` or `{"error": "message"}`
///
/// # Safety
///
/// This function dereferences a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn delete_user(username: *const c_char) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    ffi_call!(APP_API.as_ref().unwrap().users_api().delete_user(&username_str))
}

/// Create a new learning profile for a user
///
/// # Arguments
///
/// * `username` - Null-terminated C string containing the username
/// * `profile_name` - Null-terminated C string containing the profile name
/// * `target_language` - Null-terminated C string containing the target language
///
/// # Returns
///
/// JSON string: `{"username": "...", "profile_name": "...", "target_language": "...", ...}` or `{"error": "message"}`
///
/// # Safety
///
/// This function dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn create_profile(
    username: *const c_char,
    profile_name: *const c_char,
    target_language: *const c_char,
) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_name_str = match c_str_to_rust(profile_name, "profile_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let target_language_str = match c_str_to_rust(target_language, "target_language") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .users_api()
        .create_profile(&username_str, &profile_name_str, &target_language_str))
}

/// Delete a profile and its associated database file
///
/// # Arguments
///
/// * `username` - Null-terminated C string containing the username
/// * `profile_name` - Null-terminated C string containing the profile name
///
/// # Returns
///
/// JSON string: `{"success": true}` or `{"error": "message"}`
///
/// # Safety
///
/// This function dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn delete_profile(username: *const c_char, profile_name: *const c_char) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_name_str = match c_str_to_rust(profile_name, "profile_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .users_api()
        .delete_profile(&username_str, &profile_name_str))
}
