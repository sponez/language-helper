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

pub mod user_entity;

pub use user_entity::UserEntity;
