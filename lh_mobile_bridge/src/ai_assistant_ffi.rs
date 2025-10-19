//! AI Assistant API FFI functions.
//!
//! This module provides FFI wrappers for AI assistant operations.
//! Note: Ollama server management functions are omitted for mobile (desktop-only features).

use std::os::raw::c_char;

use crate::common::{c_str_to_rust, error_response, APP_API, RUNTIME};

/// Explain a phrase or word using AI
///
/// # Arguments
///
/// * `assistant_settings_json` - JSON string of AssistantSettingsDto
/// * `user_language` - The language in which the AI should respond
/// * `profile_language` - The language being learned
/// * `message` - The phrase or word to explain
///
/// # Returns
///
/// JSON string: AI's explanation text or `{"error": "message"}`
#[no_mangle]
pub unsafe extern "C" fn ai_explain(
    assistant_settings_json: *const c_char,
    user_language: *const c_char,
    profile_language: *const c_char,
    message: *const c_char,
) -> *const c_char {
    let settings_json_str = match c_str_to_rust(assistant_settings_json, "assistant_settings_json") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let user_lang_str = match c_str_to_rust(user_language, "user_language") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_lang_str = match c_str_to_rust(profile_language, "profile_language") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let message_str = match c_str_to_rust(message, "message") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let settings: lh_api::models::assistant_settings::AssistantSettingsDto =
        match serde_json::from_str(&settings_json_str) {
            Ok(s) => s,
            Err(e) => return error_response(&format!("Invalid settings JSON: {}", e)),
        };

    let api = match APP_API.as_ref() {
        Some(api) => api,
        None => return error_response("App not initialized"),
    };

    let runtime = match RUNTIME.as_ref() {
        Some(rt) => rt,
        None => return error_response("Runtime not initialized"),
    };

    match runtime.block_on(api.ai_assistant_api().explain(
        settings,
        user_lang_str,
        profile_lang_str,
        message_str,
    )) {
        Ok(explanation) => {
            // Return the explanation text directly as a JSON string
            let result = serde_json::json!(explanation);
            crate::common::success_response(&result)
        }
        Err(e) => error_response(&format!("{}", e)),
    }
}

/// Fill a vocabulary card using AI
///
/// # Arguments
///
/// * `assistant_settings_json` - JSON string of AssistantSettingsDto
/// * `card_name` - The word or phrase to create a card for
/// * `card_type` - The type of card ("Straight" or "Reverse")
/// * `user_language` - The learner's native/interface language
/// * `profile_language` - The target/study language
///
/// # Returns
///
/// JSON string: CardDto or `{"error": "message"}`
#[no_mangle]
pub unsafe extern "C" fn ai_fill_card(
    assistant_settings_json: *const c_char,
    card_name: *const c_char,
    card_type: *const c_char,
    user_language: *const c_char,
    profile_language: *const c_char,
) -> *const c_char {
    let settings_json_str = match c_str_to_rust(assistant_settings_json, "assistant_settings_json") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let card_name_str = match c_str_to_rust(card_name, "card_name") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let card_type_str = match c_str_to_rust(card_type, "card_type") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let user_lang_str = match c_str_to_rust(user_language, "user_language") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_lang_str = match c_str_to_rust(profile_language, "profile_language") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let settings: lh_api::models::assistant_settings::AssistantSettingsDto =
        match serde_json::from_str(&settings_json_str) {
            Ok(s) => s,
            Err(e) => return error_response(&format!("Invalid settings JSON: {}", e)),
        };

    let api = match APP_API.as_ref() {
        Some(api) => api,
        None => return error_response("App not initialized"),
    };

    let runtime = match RUNTIME.as_ref() {
        Some(rt) => rt,
        None => return error_response("Runtime not initialized"),
    };

    match runtime.block_on(api.ai_assistant_api().fill_card(
        settings,
        card_name_str,
        card_type_str,
        user_lang_str,
        profile_lang_str,
    )) {
        Ok(card) => crate::common::success_response(&card),
        Err(e) => error_response(&format!("{}", e)),
    }
}

/// Merge inverse card meanings into existing cards using AI
///
/// # Arguments
///
/// * `assistant_settings_json` - JSON string of AssistantSettingsDto
/// * `new_card_json` - JSON string of the new inverse CardDto
/// * `existing_cards_json` - JSON array of existing CardDto objects
/// * `user_language` - The learner's native/interface language
/// * `profile_language` - The target/study language
///
/// # Returns
///
/// JSON string: Array of updated CardDto objects or `{"error": "message"}`
#[no_mangle]
pub unsafe extern "C" fn ai_merge_inverse_cards(
    assistant_settings_json: *const c_char,
    new_card_json: *const c_char,
    existing_cards_json: *const c_char,
    user_language: *const c_char,
    profile_language: *const c_char,
) -> *const c_char {
    let settings_json_str = match c_str_to_rust(assistant_settings_json, "assistant_settings_json") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let new_card_json_str = match c_str_to_rust(new_card_json, "new_card_json") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let existing_cards_json_str = match c_str_to_rust(existing_cards_json, "existing_cards_json") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let user_lang_str = match c_str_to_rust(user_language, "user_language") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let profile_lang_str = match c_str_to_rust(profile_language, "profile_language") {
        Ok(s) => s,
        Err(err_ptr) => return err_ptr,
    };

    let settings: lh_api::models::assistant_settings::AssistantSettingsDto =
        match serde_json::from_str(&settings_json_str) {
            Ok(s) => s,
            Err(e) => return error_response(&format!("Invalid settings JSON: {}", e)),
        };

    let new_card: lh_api::models::card::CardDto = match serde_json::from_str(&new_card_json_str) {
        Ok(c) => c,
        Err(e) => return error_response(&format!("Invalid new_card JSON: {}", e)),
    };

    let existing_cards: Vec<lh_api::models::card::CardDto> = match serde_json::from_str(&existing_cards_json_str) {
        Ok(c) => c,
        Err(e) => return error_response(&format!("Invalid existing_cards JSON: {}", e)),
    };

    let api = match APP_API.as_ref() {
        Some(api) => api,
        None => return error_response("App not initialized"),
    };

    let runtime = match RUNTIME.as_ref() {
        Some(rt) => rt,
        None => return error_response("Runtime not initialized"),
    };

    match runtime.block_on(api.ai_assistant_api().merge_inverse_cards(
        settings,
        new_card,
        existing_cards,
        user_lang_str,
        profile_lang_str,
    )) {
        Ok(updated_cards) => crate::common::success_response(&updated_cards),
        Err(e) => error_response(&format!("{}", e)),
    }
}
