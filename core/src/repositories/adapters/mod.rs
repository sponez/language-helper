//! Repository adapters for mapping between layers.
//!
//! This module contains adapters that wrap persistence layer repositories
//! and map their errors to core layer errors, maintaining clean architecture.
//!
//! # Design
//!
//! Adapters implement the core's repository traits while wrapping persistence
//! layer implementations. This maintains the Dependency Inversion Principle
//! by ensuring the core layer doesn't depend on persistence details.
//!
//! # Error Mapping
//!
//! All adapters map `PersistenceError` to `CoreError`, providing a clean
//! separation between layers while maintaining error semantics.

pub mod user_repository_adapter;

pub use user_repository_adapter::{PersistenceUserRepository, UserRepositoryAdapter};
