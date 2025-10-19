//! FFI Bridge for Language Helper 2 Mobile Apps
//!
//! This crate provides C-compatible FFI functions that expose the Language Helper
//! API to mobile platforms (Flutter/Dart on Android and iOS).
//!
//! # Architecture
//!
//! ```text
//! Flutter (Dart) <-> FFI Bridge (this crate) <-> Core Logic (lh_api, lh_core, lh_persistence)
//! ```
//!
//! The bridge maintains a global `AppApi` instance and provides simple C functions
//! that serialize/deserialize data as JSON strings.
//!
//! # Module Organization
//!
//! - `common` - Shared utilities and macros
//! - `init` - Application initialization
//! - `memory` - Memory management (free_string)
//! - `users_ffi` - Users API wrappers
//! - `app_settings_ffi` - App Settings API wrappers
//! - `profiles_ffi` - Profiles API wrappers (cards, learning, settings)
//! - `ai_assistant_ffi` - AI Assistant API wrappers
//!
//! # Usage Example
//!
//! ```dart
//! // 1. Initialize the app (call once)
//! final dbPath = '/path/to/main.db'.toNativeUtf8();
//! final success = initApp(dbPath.cast());
//! calloc.free(dbPath);
//!
//! // 2. Call API functions
//! final jsonPtr = getUsernames();
//! final jsonStr = jsonPtr.toDartString();
//! freeString(jsonPtr);  // Important: free memory!
//!
//! final usernames = jsonDecode(jsonStr);
//! print(usernames);  // ["user1", "user2", ...]
//! ```

// Common utilities
#[macro_use]
mod common;

// Module declarations
mod init;
mod memory;
mod users_ffi;
mod app_settings_ffi;
mod profiles_ffi;
mod ai_assistant_ffi;

// Re-export all FFI functions
pub use init::*;
pub use memory::*;
pub use users_ffi::*;
pub use app_settings_ffi::*;
pub use profiles_ffi::*;
pub use ai_assistant_ffi::*;
