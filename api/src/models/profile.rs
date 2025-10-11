//! Profile data transfer objects.
//!
//! This module defines the DTOs for learning profiles.

use serde::{Deserialize, Serialize};

/// Data transfer object for a learning profile.
///
/// This struct represents a profile that can be transferred
/// between layers and serialized for storage or network transmission.
///
/// # Examples
///
/// ```
/// use lh_api::models::profile::ProfileDto;
///
/// let profile = ProfileDto {
///     target_language: "spanish".to_string(),
///     created_at: 1234567890,
///     last_activity: 1234567900,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileDto {
    /// Target language being learned.
    pub target_language: String,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last activity.
    pub last_activity: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_dto_creation() {
        let dto = ProfileDto {
            target_language: "spanish".to_string(),
            created_at: 1000,
            last_activity: 2000,
        };
        assert_eq!(dto.target_language, "spanish");
        assert_eq!(dto.created_at, 1000);
        assert_eq!(dto.last_activity, 2000);
    }

    #[test]
    fn test_profile_dto_clone() {
        let dto = ProfileDto {
            target_language: "french".to_string(),
            created_at: 3000,
            last_activity: 4000,
        };
        let cloned = dto.clone();
        assert_eq!(dto, cloned);
    }

    #[test]
    fn test_profile_dto_serialization() {
        let dto = ProfileDto {
            target_language: "german".to_string(),
            created_at: 5000,
            last_activity: 6000,
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("german"));
        assert!(json.contains("5000"));
        assert!(json.contains("6000"));
    }

    #[test]
    fn test_profile_dto_deserialization() {
        let json = r#"{
            "target_language": "italian",
            "created_at": 7000,
            "last_activity": 8000
        }"#;
        let dto: ProfileDto = serde_json::from_str(json).unwrap();
        assert_eq!(dto.target_language, "italian");
        assert_eq!(dto.created_at, 7000);
        assert_eq!(dto.last_activity, 8000);
    }
}
