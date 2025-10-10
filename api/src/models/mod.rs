//! Data transfer objects (DTOs) for API communication.
//!
//! This module contains the data structures used to transfer data between
//! the API layer and its consumers. These DTOs are designed to be serializable
//! and provide a stable interface independent of internal domain models.
//!
//! # Available Models
//!
//! - **user**: User-related data transfer objects
//! - **app_settings**: Application settings data transfer objects
//! - **user_settings**: User-specific settings data transfer objects
//! - **profile**: Learning profile data transfer objects

pub mod app_settings;
pub mod profile;
pub mod user;
pub mod user_settings;
