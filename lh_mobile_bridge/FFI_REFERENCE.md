# FFI Function Reference

Quick reference for all 40 FFI functions in the mobile bridge.

## Initialization

### `init_app(db_path: *const c_char) -> bool`
Initialize the app. Must be called once before any other functions.
- **Args**: Path to main.db file
- **Returns**: `true` on success, `false` on failure

## Memory Management

### `free_string(ptr: *const c_char)`
Free a string allocated by Rust. Call this for every string returned!
- **Args**: Pointer to string
- **Returns**: void

## Users API (8 functions)

### `get_usernames() -> *const c_char`
- **Returns**: JSON array `["user1", "user2"]` or `{"error": "..."}`

### `get_user_by_username(username: *const c_char) -> *const c_char`
- **Args**: Username
- **Returns**: JSON UserDto or error

### `create_user(username: *const c_char, language: *const c_char) -> *const c_char`
- **Args**: Username, language code (e.g., "en-US")
- **Returns**: JSON UserDto or error

### `update_user_theme(username: *const c_char, theme: *const c_char) -> *const c_char`
- **Args**: Username, theme ("Dark" or "Light")
- **Returns**: `{"success": true}` or error

### `update_user_language(username: *const c_char, language: *const c_char) -> *const c_char`
- **Args**: Username, language code
- **Returns**: `{"success": true}` or error

### `delete_user(username: *const c_char) -> *const c_char`
- **Args**: Username
- **Returns**: `{"success": true}` or error

### `create_profile(username: *const c_char, profile_name: *const c_char, target_language: *const c_char) -> *const c_char`
- **Args**: Username, profile name, target language
- **Returns**: JSON ProfileDto or error

### `delete_profile(username: *const c_char, profile_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name
- **Returns**: `{"success": true}` or error

## App Settings API (3 functions)

### `get_app_settings() -> *const c_char`
- **Returns**: JSON `{"theme": "Dark", "language": "en-US"}` or error

### `update_app_theme(theme: *const c_char) -> *const c_char`
- **Args**: Theme ("Dark" or "Light")
- **Returns**: `{"success": true}` or error

### `update_app_language(language: *const c_char) -> *const c_char`
- **Args**: Language code
- **Returns**: `{"success": true}` or error

## Profiles API (22 functions)

### Database Management

#### `create_profile_database(username: *const c_char, profile_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name
- **Returns**: `{"success": true}` or error

#### `delete_profile_database(username: *const c_char, profile_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name
- **Returns**: `{"success": true}` or error

### Card Settings

#### `get_card_settings(username: *const c_char, profile_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name
- **Returns**: JSON CardSettingsDto or error

#### `update_card_settings(username: *const c_char, profile_name: *const c_char, settings_json: *const c_char) -> *const c_char`
- **Args**: Username, profile name, JSON settings
- **Returns**: `{"success": true}` or error

### Assistant Settings

#### `get_assistant_settings(username: *const c_char, profile_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name
- **Returns**: JSON AssistantSettingsDto or error

#### `update_assistant_settings(username: *const c_char, profile_name: *const c_char, settings_json: *const c_char) -> *const c_char`
- **Args**: Username, profile name, JSON settings
- **Returns**: `{"success": true}` or error

#### `clear_assistant_settings(username: *const c_char, profile_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name
- **Returns**: `{"success": true}` or error

### Card Management

#### `save_card(username: *const c_char, profile_name: *const c_char, card_json: *const c_char) -> *const c_char`
- **Args**: Username, profile name, JSON CardDto
- **Returns**: `{"success": true}` or error

#### `get_all_cards(username: *const c_char, profile_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name
- **Returns**: JSON array of CardDto or error

#### `get_unlearned_cards(username: *const c_char, profile_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name
- **Returns**: JSON array of CardDto (unlearned) or error

#### `get_learned_cards(username: *const c_char, profile_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name
- **Returns**: JSON array of CardDto (learned) or error

#### `get_card_by_word_name(username: *const c_char, profile_name: *const c_char, word_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name, word name
- **Returns**: JSON CardDto or error

#### `delete_card(username: *const c_char, profile_name: *const c_char, word_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name, word name
- **Returns**: `{"success": true}` or error

### Learning Sessions

#### `create_learning_session(username: *const c_char, profile_name: *const c_char, start_card_number: usize) -> *const c_char`
- **Args**: Username, profile name, start card (1-indexed)
- **Returns**: JSON LearningSessionDto or error

#### `create_test_session(username: *const c_char, profile_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name
- **Returns**: JSON LearningSessionDto (all unlearned, shuffled) or error

#### `create_repeat_session(username: *const c_char, profile_name: *const c_char) -> *const c_char`
- **Args**: Username, profile name
- **Returns**: JSON LearningSessionDto (all learned, shuffled) or error

#### `check_answer(session_json: *const c_char, user_input: *const c_char) -> *const c_char`
- **Args**: JSON LearningSessionDto, user's answer
- **Returns**: JSON `{"is_correct": bool, "matched_answer": "..."}` or error

#### `process_self_review(username: *const c_char, profile_name: *const c_char, word_name: *const c_char, is_correct: bool) -> *const c_char`
- **Args**: Username, profile name, word name, correct/incorrect
- **Returns**: JSON TestResultDto or error

#### `update_test_streaks(username: *const c_char, profile_name: *const c_char, results_json: *const c_char) -> *const c_char`
- **Args**: Username, profile name, JSON array of TestResultDto
- **Returns**: `{"success": true}` or error

#### `update_repeat_streaks(username: *const c_char, profile_name: *const c_char, results_json: *const c_char) -> *const c_char`
- **Args**: Username, profile name, JSON array of TestResultDto
- **Returns**: `{"success": true}` or error

## AI Assistant API (3 functions)

### `ai_explain(assistant_settings_json: *const c_char, user_language: *const c_char, profile_language: *const c_char, message: *const c_char) -> *const c_char`
- **Args**: JSON AssistantSettingsDto, user language, profile language, message to explain
- **Returns**: JSON string (explanation text) or error

### `ai_fill_card(assistant_settings_json: *const c_char, card_name: *const c_char, card_type: *const c_char, user_language: *const c_char, profile_language: *const c_char) -> *const c_char`
- **Args**: JSON settings, card name, card type ("Straight"/"Reverse"), languages
- **Returns**: JSON CardDto (AI-generated) or error

### `ai_merge_inverse_cards(assistant_settings_json: *const c_char, new_card_json: *const c_char, existing_cards_json: *const c_char, user_language: *const c_char, profile_language: *const c_char) -> *const c_char`
- **Args**: JSON settings, new CardDto, array of existing CardDto, languages
- **Returns**: JSON array of updated CardDto or error

---

## Common Patterns

### Basic Call Pattern
```dart
final inputPtr = input.toNativeUtf8();
final resultPtr = some_ffi_function(inputPtr);
final resultStr = resultPtr.toDartString();
calloc.free(inputPtr);      // Free input
freeString(resultPtr);      // Free result!

final data = jsonDecode(resultStr);
if (data is Map && data.containsKey('error')) {
  throw Exception(data['error']);
}
return data;  // or parse to model
```

### Error Handling
All functions return JSON. Errors have format:
```json
{"error": "Error message here"}
```

Success responses vary:
- Simple operations: `{"success": true}`
- Data queries: Array or object
- Tuples: Object with named fields

### Memory Management Rules
1. **Always call `free_string()`** for every string returned by Rust
2. **Free inputs** you allocate with `toNativeUtf8()` using `calloc.free()`
3. **Never** call `free_string()` twice on the same pointer
4. **Never** use a pointer after freeing it
