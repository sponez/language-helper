//! Persistence adapters.

pub mod sqlite_user_repository;

pub use sqlite_user_repository::{SqliteUserRepository, SqliteUserRepositoryInitError};
