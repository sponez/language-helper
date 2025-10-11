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
pub fn model_to_entity(username: &str, profile: &Profile) -> ProfileEntity {
    ProfileEntity::with_fields(
        username,
        profile.target_language.clone(),
        profile.created_at,
        profile.last_activity_at,
    )
}
