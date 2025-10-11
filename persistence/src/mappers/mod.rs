//! Mapping functions between persistence entities and core models.
//!
//! This module contains mapper functions that convert between persistence-layer
//! entities and core-layer models. This keeps the persistence entities clean
//! and maintains proper dependency direction (persistence depends on core, not vice versa).

pub mod user_mapper;
pub mod user_settings_mapper;
pub mod profile_mapper;
pub mod app_settings_mapper;
