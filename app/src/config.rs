//! Application configuration.
//!
//! This module handles loading and managing application configuration.

use std::path::PathBuf;

/// Application configuration.
///
/// This struct holds all configuration parameters for the application.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Path to the SQLite database file
    pub database_path: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("data/users.db"),
        }
    }
}

impl AppConfig {
    /// Creates a new configuration with default values.
    ///
    /// # Returns
    ///
    /// A new `AppConfig` with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use app::config::AppConfig;
    ///
    /// let config = AppConfig::new();
    /// ```
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new configuration from environment variables.
    ///
    /// Environment variables:
    /// - `LH_DATABASE_PATH`: Path to the SQLite database file
    ///
    /// # Returns
    ///
    /// A new `AppConfig` with values from environment or defaults.
    ///
    /// # Examples
    ///
    /// ```
    /// use app::config::AppConfig;
    ///
    /// let config = AppConfig::from_env();
    /// ```
    pub fn from_env() -> Self {
        let database_path = std::env::var("LH_DATABASE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/users.db"));

        Self { database_path }
    }

    /// Sets the database path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the SQLite database file
    ///
    /// # Returns
    ///
    /// The modified configuration (builder pattern).
    ///
    /// # Examples
    ///
    /// ```
    /// use app::config::AppConfig;
    ///
    /// let config = AppConfig::new()
    ///     .with_database_path("custom/path/db.sqlite");
    /// ```
    #[allow(dead_code)]
    pub fn with_database_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.database_path = path.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database_path, PathBuf::from("data/users.db"));
    }

    #[test]
    fn test_new_config() {
        let config = AppConfig::new();
        assert_eq!(config.database_path, PathBuf::from("data/users.db"));
    }

    #[test]
    fn test_with_database_path() {
        let config = AppConfig::new()
            .with_database_path("test/db.sqlite");
        assert_eq!(config.database_path, PathBuf::from("test/db.sqlite"));
    }

    #[test]
    fn test_from_env_with_defaults() {
        // This test assumes LH_DATABASE_PATH is not set
        std::env::remove_var("LH_DATABASE_PATH");
        let config = AppConfig::from_env();
        assert_eq!(config.database_path, PathBuf::from("data/users.db"));
    }

    #[test]
    fn test_from_env_with_custom_path() {
        std::env::set_var("LH_DATABASE_PATH", "custom/test.db");
        let config = AppConfig::from_env();
        assert_eq!(config.database_path, PathBuf::from("custom/test.db"));
        std::env::remove_var("LH_DATABASE_PATH");
    }
}
