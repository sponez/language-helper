//! Repository implementations.
//!
//! This module contains concrete repository implementations using SQLite
//! as the persistence backend.

pub mod sqlite_app_settings_repository;
pub mod sqlite_profile_repository;
pub mod sqlite_user_repository;
pub mod sqlite_user_settings_repository;

pub use sqlite_app_settings_repository::SqliteAppSettingsRepository;
pub use sqlite_profile_repository::SqliteProfileRepository;
pub use sqlite_user_repository::SqliteUserRepository;
pub use sqlite_user_settings_repository::SqliteUserSettingsRepository;
