//! Persistence layer for Language Helper.
//!
//! This crate provides concrete implementations of the repository traits
//! defined in the core layer, using SQLite as the backend.

pub mod sqlite_user_repository;

pub use sqlite_user_repository::SqliteUserRepository;
