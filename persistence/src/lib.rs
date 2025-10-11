//! Persistence layer for Language Helper.
//!
//! This crate provides concrete implementations of the repository traits
//! defined in the core layer, using SQLite as the backend.
//!
//! # Architecture
//!
//! The persistence layer follows a clean architecture approach:
//!
//! - **models**: Persistence-specific models with database-friendly types
//! - **mappers**: Conversion functions between persistence entities and core models
//! - **repositories**: SQLite implementations of repository traits
//! - **errors**: Persistence-specific error types (independent of core)
//!
//! # Design Decisions
//!
//! - Timestamps are stored as Unix timestamps (i64) for efficiency
//! - Persistence models are separate from domain models
//! - Persistence errors are separate from core errors
//! - Mapping functions convert between layers
//!
//! # Error Handling
//!
//! The persistence layer uses `PersistenceError` for all operations.
//! When implementing repository traits (which require `CoreError`),
//! a mapping layer in the core converts `PersistenceError` to `CoreError`.

pub mod errors;
pub mod mappers;
pub mod models;
pub mod repositories;

pub use errors::PersistenceError;
pub use models::{AppSettingsEntity, ProfileEntity, UserEntity, UserSettingsEntity};
pub use repositories::{
    SqliteAppSettingsRepository, SqliteProfileDbRepository, SqliteProfileRepository,
    SqliteUserRepository, SqliteUserSettingsRepository,
};
