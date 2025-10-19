//! Profiles API FFI functions.
//!
//! This module provides FFI wrappers for profile-specific operations including:
//! - Card management (CRUD)
//! - Card settings
//! - Assistant settings
//! - Learning sessions (learn, test, repeat modes)

use std::os::raw::c_char;

use crate::common::{c_str_to_rust, error_response, success_response, APP_API, RUNTIME};
use crate::ffi_call;

//
// Profile Database Management
//

/// Create a profile database file
///
/// # Arguments
///
/// * `username` - Null-terminated C string containing the username
/// * `profile_name` - Null-terminated C string containing the profile name
///
/// # Returns
///
/// JSON string: `{"success": true}` or `{"error": "message"}`
#[no_mangle]
pub unsafe extern "C" fn create_profile_database(
    username: *const c_char,
    profile_name: *const c_char,
) -> *const c_char {
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
        .profile_api()
        .create_profile_database(&username_str, &profile_name_str))
}

/// Delete a profile database file
///
/// # Arguments
///
/// * `username` - Null-terminated C string
/// * `profile_name` - Null-terminated C string
///
/// # Returns
///
/// JSON string: `{"success": true}` or `{"error": "message"}`
#[no_mangle]
pub unsafe extern "C" fn delete_profile_database(
    username: *const c_char,
    profile_name: *const c_char,
) -> *const c_char {
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
        .profile_api()
        .delete_profile_database(&username_str, &profile_name_str))
}

//
// Card Settings
//

/// Get card settings from a profile database
///
/// # Returns
///
/// JSON string: `{"cards_per_set": 10, "test_method": "manual", "required_streak": 3}` or `{"error": "message"}`
#[no_mangle]
pub unsafe extern "C" fn get_card_settings(
    username: *const c_char,
    profile_name: *const c_char,
) -> *const c_char {
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
        .profile_api()
        .get_card_settings(&username_str, &profile_name_str))
}

/// Update card settings in a profile database
///
/// # Arguments
///
/// * `settings_json` - JSON string of CardSettingsDto
///
/// # Returns
///
/// JSON string: `{"success": true}` or `{"error": "message"}`
#[no_mangle]
pub unsafe extern "C" fn update_card_settings(
    username: *const c_char,
    profile_name: *const c_char,
    settings_json: *const c_char,
) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_name_str = match c_str_to_rust(profile_name, "profile_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let settings_json_str = match c_str_to_rust(settings_json, "settings_json") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    // Deserialize JSON to DTO
    let settings: lh_api::models::card_settings::CardSettingsDto =
        match serde_json::from_str(&settings_json_str) {
            Ok(s) => s,
            Err(e) => return error_response(&format!("Invalid settings JSON: {}", e)),
        };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .profile_api()
        .update_card_settings(&username_str, &profile_name_str, settings))
}

//
// Assistant Settings
//

/// Get assistant settings from a profile database
#[no_mangle]
pub unsafe extern "C" fn get_assistant_settings(
    username: *const c_char,
    profile_name: *const c_char,
) -> *const c_char {
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
        .profile_api()
        .get_assistant_settings(&username_str, &profile_name_str))
}

/// Update assistant settings in a profile database
#[no_mangle]
pub unsafe extern "C" fn update_assistant_settings(
    username: *const c_char,
    profile_name: *const c_char,
    settings_json: *const c_char,
) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_name_str = match c_str_to_rust(profile_name, "profile_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let settings_json_str = match c_str_to_rust(settings_json, "settings_json") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let settings: lh_api::models::assistant_settings::AssistantSettingsDto =
        match serde_json::from_str(&settings_json_str) {
            Ok(s) => s,
            Err(e) => return error_response(&format!("Invalid settings JSON: {}", e)),
        };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .profile_api()
        .update_assistant_settings(&username_str, &profile_name_str, settings))
}

/// Clear assistant settings in a profile database
#[no_mangle]
pub unsafe extern "C" fn clear_assistant_settings(
    username: *const c_char,
    profile_name: *const c_char,
) -> *const c_char {
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
        .profile_api()
        .clear_assistant_settings(&username_str, &profile_name_str))
}

//
// Card Management
//

/// Save a card to the profile database (create or update)
#[no_mangle]
pub unsafe extern "C" fn save_card(
    username: *const c_char,
    profile_name: *const c_char,
    card_json: *const c_char,
) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_name_str = match c_str_to_rust(profile_name, "profile_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let card_json_str = match c_str_to_rust(card_json, "card_json") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let card: lh_api::models::card::CardDto = match serde_json::from_str(&card_json_str) {
        Ok(c) => c,
        Err(e) => return error_response(&format!("Invalid card JSON: {}", e)),
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .profile_api()
        .save_card(&username_str, &profile_name_str, card))
}

/// Get all cards from the profile database
#[no_mangle]
pub unsafe extern "C" fn get_all_cards(username: *const c_char, profile_name: *const c_char) -> *const c_char {
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
        .profile_api()
        .get_all_cards(&username_str, &profile_name_str))
}

/// Get unlearned cards (streak below threshold)
#[no_mangle]
pub unsafe extern "C" fn get_unlearned_cards(username: *const c_char, profile_name: *const c_char) -> *const c_char {
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
        .profile_api()
        .get_unlearned_cards(&username_str, &profile_name_str))
}

/// Get learned cards (streak at or above threshold)
#[no_mangle]
pub unsafe extern "C" fn get_learned_cards(username: *const c_char, profile_name: *const c_char) -> *const c_char {
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
        .profile_api()
        .get_learned_cards(&username_str, &profile_name_str))
}

/// Get a single card by word name
#[no_mangle]
pub unsafe extern "C" fn get_card_by_word_name(
    username: *const c_char,
    profile_name: *const c_char,
    word_name: *const c_char,
) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_name_str = match c_str_to_rust(profile_name, "profile_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let word_name_str = match c_str_to_rust(word_name, "word_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .profile_api()
        .get_card_by_word_name(&username_str, &profile_name_str, &word_name_str))
}

/// Delete a card from the database
#[no_mangle]
pub unsafe extern "C" fn delete_card(
    username: *const c_char,
    profile_name: *const c_char,
    word_name: *const c_char,
) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_name_str = match c_str_to_rust(profile_name, "profile_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let word_name_str = match c_str_to_rust(word_name, "word_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .profile_api()
        .delete_card(&username_str, &profile_name_str, &word_name_str))
}

//
// Learning Sessions
//

/// Create a new learning session from unlearned cards
///
/// # Arguments
///
/// * `start_card_number` - Starting card number (1-indexed)
#[no_mangle]
pub unsafe extern "C" fn create_learning_session(
    username: *const c_char,
    profile_name: *const c_char,
    start_card_number: usize,
) -> *const c_char {
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
        .profile_api()
        .create_learning_session(&username_str, &profile_name_str, start_card_number))
}

/// Create a test session from unlearned cards (shuffled, all cards)
#[no_mangle]
pub unsafe extern "C" fn create_test_session(username: *const c_char, profile_name: *const c_char) -> *const c_char {
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
        .profile_api()
        .create_test_session(&username_str, &profile_name_str))
}

/// Create a repeat session from learned cards (shuffled, all cards)
#[no_mangle]
pub unsafe extern "C" fn create_repeat_session(username: *const c_char, profile_name: *const c_char) -> *const c_char {
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
        .profile_api()
        .create_repeat_session(&username_str, &profile_name_str))
}

/// Check a written answer against the session's current card
///
/// # Arguments
///
/// * `session_json` - JSON string of LearningSessionDto
/// * `user_input` - The user's answer
///
/// # Returns
///
/// JSON string: `{"is_correct": true, "matched_answer": "..."}` or `{"error": "message"}`
#[no_mangle]
pub unsafe extern "C" fn check_answer(session_json: *const c_char, user_input: *const c_char) -> *const c_char {
    let session_json_str = match c_str_to_rust(session_json, "session_json") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let user_input_str = match c_str_to_rust(user_input, "user_input") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let session: lh_api::models::learning_session::LearningSessionDto =
        match serde_json::from_str(&session_json_str) {
            Ok(s) => s,
            Err(e) => return error_response(&format!("Invalid session JSON: {}", e)),
        };

    let api = match APP_API.as_ref() {
        Some(api) => api,
        None => return error_response("App not initialized"),
    };

    let runtime = match RUNTIME.as_ref() {
        Some(rt) => rt,
        None => return error_response("Runtime not initialized"),
    };

    match runtime.block_on(api.profile_api().check_answer(&session, &user_input_str)) {
        Ok((is_correct, matched)) => {
            let result = serde_json::json!({
                "is_correct": is_correct,
                "matched_answer": matched
            });
            success_response(&result)
        }
        Err(e) => error_response(&format!("{}", e)),
    }
}

/// Process a self-review result for a card
#[no_mangle]
pub unsafe extern "C" fn process_self_review(
    username: *const c_char,
    profile_name: *const c_char,
    word_name: *const c_char,
    is_correct: bool,
) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_name_str = match c_str_to_rust(profile_name, "profile_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let word_name_str = match c_str_to_rust(word_name, "word_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    ffi_call!(APP_API.as_ref().unwrap().profile_api().process_self_review(
        &username_str,
        &profile_name_str,
        &word_name_str,
        is_correct
    ))
}

/// Update card streaks based on test results
///
/// # Arguments
///
/// * `results_json` - JSON array of TestResultDto objects
#[no_mangle]
pub unsafe extern "C" fn update_test_streaks(
    username: *const c_char,
    profile_name: *const c_char,
    results_json: *const c_char,
) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_name_str = match c_str_to_rust(profile_name, "profile_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let results_json_str = match c_str_to_rust(results_json, "results_json") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let results: Vec<lh_api::models::test_result::TestResultDto> = match serde_json::from_str(&results_json_str) {
        Ok(r) => r,
        Err(e) => return error_response(&format!("Invalid results JSON: {}", e)),
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .profile_api()
        .update_test_streaks(&username_str, &profile_name_str, results))
}

/// Update card streaks based on repeat session results
#[no_mangle]
pub unsafe extern "C" fn update_repeat_streaks(
    username: *const c_char,
    profile_name: *const c_char,
    results_json: *const c_char,
) -> *const c_char {
    let username_str = match c_str_to_rust(username, "username") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_name_str = match c_str_to_rust(profile_name, "profile_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let results_json_str = match c_str_to_rust(results_json, "results_json") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let results: Vec<lh_api::models::test_result::TestResultDto> = match serde_json::from_str(&results_json_str) {
        Ok(r) => r,
        Err(e) => return error_response(&format!("Invalid results JSON: {}", e)),
    };

    ffi_call!(APP_API
        .as_ref()
        .unwrap()
        .profile_api()
        .update_repeat_streaks(&username_str, &profile_name_str, results))
}
