//! Application settings API.
//!
//! This module provides the trait definition for global application settings operations.

use crate::{errors::api_error::ApiError, models::app_settings::AppSettingsDto};
use async_trait::async_trait;

/// API for managing global application settings.
///
/// This trait defines the interface for operations on the global application settings.
/// These are singleton settings that apply as defaults for new users.
#[async_trait]
pub trait AppSettingsApi: Send + Sync {
    /// Retrieves the global application settings.
    async fn get_app_settings(&self) -> Result<AppSettingsDto, ApiError>;

    /// Updates the global theme setting.
    async fn update_app_theme(&self, theme: &str) -> Result<(), ApiError>;

    /// Updates the global language setting.
    async fn update_app_language(&self, language: &str) -> Result<(), ApiError>;
}
