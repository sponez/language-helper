//! Mapper functions for converting between ProfileEntity and Profile model.

use crate::models::ProfileEntity;
use lh_core::models::profile::Profile;

/// Converts a persistence ProfileEntity to a core Profile model.
///
/// # Arguments
///
/// * `entity` - The ProfileEntity to convert
///
/// # Returns
///
/// A Profile model instance
pub fn entity_to_model(entity: &ProfileEntity) -> Profile {
    Profile::new_unchecked(
        entity.profile_id.clone(),
        entity.username.clone(),
        entity.target_language.clone(),
        entity.created_at,
        entity.last_activity_at,
    )
}

/// Converts a core Profile model to a persistence ProfileEntity.
///
/// # Arguments
///
/// * `profile` - The Profile model to convert
///
/// # Returns
///
/// A ProfileEntity
pub fn model_to_entity(profile: &Profile) -> ProfileEntity {
    ProfileEntity::with_fields(
        profile.profile_id.clone(),
        profile.username.clone(),
        profile.target_language.clone(),
        profile.created_at,
        profile.last_activity_at,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_to_model() {
        let entity = ProfileEntity::new("test_user".to_string(), "spanish".to_string());
        let profile = entity_to_model(&entity);

        assert_eq!(profile.profile_id, entity.profile_id);
        assert_eq!(profile.username, entity.username);
        assert_eq!(profile.target_language, entity.target_language);
        assert_eq!(profile.created_at, entity.created_at);
        assert_eq!(profile.last_activity_at, entity.last_activity_at);
    }

    #[test]
    fn test_model_to_entity() {
        let profile = Profile::new("test_user".to_string(), "french".to_string()).unwrap();
        let entity = model_to_entity(&profile);

        assert_eq!(entity.profile_id, profile.profile_id);
        assert_eq!(entity.username, profile.username);
        assert_eq!(entity.target_language, profile.target_language);
        assert_eq!(entity.created_at, profile.created_at);
        assert_eq!(entity.last_activity_at, profile.last_activity_at);
    }

    #[test]
    fn test_roundtrip() {
        let original_profile =
            Profile::new("roundtrip_test".to_string(), "german".to_string()).unwrap();
        let entity = model_to_entity(&original_profile);
        let converted_profile = entity_to_model(&entity);

        assert_eq!(original_profile.profile_id, converted_profile.profile_id);
        assert_eq!(original_profile.username, converted_profile.username);
        assert_eq!(
            original_profile.target_language,
            converted_profile.target_language
        );
    }
}
