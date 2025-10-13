//! Persistence models.
//!
//! This module contains persistence-specific models that represent how data
//! is stored in the database. These models are separate from domain models
//! to maintain clean architecture and allow persistence-specific optimizations.
//!
//! # Design
//!
//! - Persistence models use database-friendly types (e.g., i64 for timestamps)
//! - Mapping functions convert between persistence and domain models
//! - Timestamps are stored as Unix timestamps (seconds since epoch)

pub mod app_settings_entity;
pub mod assistant_settings_entity;
pub mod card_settings_entity;
pub mod profile_entity;
pub mod user_entity;
pub mod user_settings_entity;

pub use app_settings_entity::AppSettingsEntity;
pub use assistant_settings_entity::AssistantSettingsEntity;
pub use card_settings_entity::CardSettingsEntity;
pub use profile_entity::ProfileEntity;
pub use user_entity::UserEntity;
pub use user_settings_entity::UserSettingsEntity;
