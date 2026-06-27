//! Repository traits.
//!
//! This module defines repository interfaces for persistence operations.
//! These traits will be implemented by the persistence layer.

pub mod adapters;
pub mod app_settings_repository;
pub mod profile_repository;
pub mod user_profiles_repository;
pub mod user_repository;
pub mod user_settings_repository;

pub use app_settings_repository::AppSettingsRepository;
pub use profile_repository::ProfileRepository;
pub use user_profiles_repository::UserProfilesRepository;
pub use user_repository::UserRepository;
pub use user_settings_repository::UserSettingsRepository;
