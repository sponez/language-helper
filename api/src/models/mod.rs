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
//! - **card_settings**: Card learning settings data transfer objects
//! - **assistant_settings**: AI assistant settings data transfer objects
//! - **system_requirements**: System requirements and compatibility data transfer objects
//! - **ai_assistant**: AI assistant and running models data transfer objects

pub mod ai_assistant;
pub mod app_settings;
pub mod assistant_settings;
pub mod card_settings;
pub mod profile;
pub mod system_requirements;
pub mod user;
pub mod user_settings;
