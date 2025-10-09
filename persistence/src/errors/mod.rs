//! Persistence layer error types.
//!
//! This module defines error types specific to the persistence layer.
//! These errors represent issues that can occur during database operations
//! and are independent of the core business logic layer.

use thiserror::Error;

/// Errors that can occur in the persistence layer.
///
/// This enum represents all possible errors that can occur during
/// persistence operations (database access, file system operations, etc.).
///
/// # Design
///
/// These errors are persistence-specific and don't depend on core business logic.
/// They will be mapped to `CoreError` by the core layer when needed.
///
/// # Examples
///
/// ```
/// use lh_persistence::errors::PersistenceError;
///
/// let error = PersistenceError::database_error("Connection failed");
/// println!("Error: {}", error);
/// ```
#[derive(Error, Debug)]
pub enum PersistenceError {
    /// Database operation error.
    ///
    /// Used when a database operation fails (connection, query, etc.).
    ///
    /// # Fields
    ///
    /// * `message` - Description of the database error
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    /// I/O error.
    ///
    /// Used when file system operations fail.
    ///
    /// # Fields
    ///
    /// * `message` - Description of the I/O error
    #[error("I/O error: {message}")]
    IoError { message: String },

    /// Data serialization/deserialization error.
    ///
    /// Used when data cannot be properly serialized or deserialized.
    ///
    /// # Fields
    ///
    /// * `message` - Description of the serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Lock acquisition error.
    ///
    /// Used when a database connection lock cannot be acquired.
    ///
    /// # Fields
    ///
    /// * `message` - Description of the lock error
    #[error("Lock error: {message}")]
    LockError { message: String },
}

impl PersistenceError {
    /// Creates a new DatabaseError.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message
    ///
    /// # Returns
    ///
    /// A new `PersistenceError::DatabaseError` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_persistence::errors::PersistenceError;
    ///
    /// let error = PersistenceError::database_error("Failed to connect");
    /// ```
    pub fn database_error(message: impl Into<String>) -> Self {
        PersistenceError::DatabaseError {
            message: message.into(),
        }
    }

    /// Creates a new IoError.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message
    ///
    /// # Returns
    ///
    /// A new `PersistenceError::IoError` instance.
    pub fn io_error(message: impl Into<String>) -> Self {
        PersistenceError::IoError {
            message: message.into(),
        }
    }

    /// Creates a new SerializationError.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message
    ///
    /// # Returns
    ///
    /// A new `PersistenceError::SerializationError` instance.
    pub fn serialization_error(message: impl Into<String>) -> Self {
        PersistenceError::SerializationError {
            message: message.into(),
        }
    }

    /// Creates a new LockError.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message
    ///
    /// # Returns
    ///
    /// A new `PersistenceError::LockError` instance.
    pub fn lock_error(message: impl Into<String>) -> Self {
        PersistenceError::LockError {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_error() {
        let error = PersistenceError::database_error("Connection failed");
        match error {
            PersistenceError::DatabaseError { message } => {
                assert_eq!(message, "Connection failed");
            }
            _ => panic!("Expected DatabaseError"),
        }
    }

    #[test]
    fn test_io_error() {
        let error = PersistenceError::io_error("File not found");
        match error {
            PersistenceError::IoError { message } => {
                assert_eq!(message, "File not found");
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_serialization_error() {
        let error = PersistenceError::serialization_error("Invalid format");
        match error {
            PersistenceError::SerializationError { message } => {
                assert_eq!(message, "Invalid format");
            }
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_lock_error() {
        let error = PersistenceError::lock_error("Could not acquire lock");
        match error {
            PersistenceError::LockError { message } => {
                assert_eq!(message, "Could not acquire lock");
            }
            _ => panic!("Expected LockError"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = PersistenceError::database_error("Test error");
        let display = format!("{}", error);
        assert!(display.contains("Database error"));
        assert!(display.contains("Test error"));
    }
}
