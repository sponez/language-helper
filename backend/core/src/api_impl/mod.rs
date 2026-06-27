//! API trait implementations.
//!
//! This module provides concrete implementations of the API layer traits,
//! bridging the API layer with the core business logic layer.

pub mod ai_assistant_api_impl;
pub mod app_api_impl;
pub mod app_settings_api_impl;
pub mod profiles_api_impl;
pub mod system_requirements_api_impl;
pub mod users_api_impl;

pub use ai_assistant_api_impl::AiAssistantApiImpl;
pub use app_api_impl::AppApiImpl;
pub use app_settings_api_impl::AppSettingsApiImpl;
pub use profiles_api_impl::ProfilesApiImpl;
pub use system_requirements_api_impl::SystemRequirementsApiImpl;
pub use users_api_impl::UsersApiImpl;
