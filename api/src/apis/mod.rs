//! API trait definitions.
//!
//! This module contains trait definitions for various API domains.
//! Each domain API provides a specific set of operations.
//!
//! # Available APIs
//!
//! - **user_api**: Operations for managing user accounts
//! - **app_settings_api**: Operations for global application settings
//! - **user_settings_api**: Operations for user-specific settings
//! - **profiles_api**: Operations for learning profiles
//! - **system_requirements_api**: Operations for checking system requirements
//! - **ai_assistant_api**: Operations for managing AI assistants

pub mod ai_assistant_api;
pub mod app_settings_api;
pub mod profiles_api;
pub mod system_requirements_api;
pub mod user_api;
