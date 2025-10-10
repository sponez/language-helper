//! Repository adapters for mapping persistence errors to core errors.
//!
//! This module provides adapter traits and implementations that bridge
//! the persistence layer and core layer, handling error mapping.

pub mod app_settings_repository_adapter;
pub mod profile_repository_adapter;
pub mod user_repository_adapter;
pub mod user_settings_repository_adapter;

pub use app_settings_repository_adapter::{
    AppSettingsRepositoryAdapter, PersistenceAppSettingsRepository,
};
pub use profile_repository_adapter::{PersistenceProfileRepository, ProfileRepositoryAdapter};
pub use user_repository_adapter::{PersistenceUserRepository, UserRepositoryAdapter};
pub use user_settings_repository_adapter::{
    PersistenceUserSettingsRepository, UserSettingsRepositoryAdapter,
};
