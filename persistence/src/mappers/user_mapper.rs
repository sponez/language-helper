//! Mapper functions for converting between UserEntity and User model.

use crate::models::UserEntity;
use lh_core::models::user::User;

/// Converts a persistence UserEntity to a core User model.
///
/// # Arguments
///
/// * `entity` - The UserEntity to convert
///
/// # Returns
///
/// A User model instance
pub fn entity_to_model(entity: &UserEntity) -> User {
    User::new_unchecked(entity.username.clone())
}

/// Converts a core User model to a persistence UserEntity.
///
/// # Arguments
///
/// * `user` - The User model to convert
///
/// # Returns
///
/// A UserEntity with current timestamps
pub fn model_to_entity(user: &User) -> UserEntity {
    UserEntity::new(user.username.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_to_model() {
        let entity = UserEntity::new("test_user".to_string());
        let user = entity_to_model(&entity);

        assert_eq!(user.username, entity.username);
    }

    #[test]
    fn test_model_to_entity() {
        let user = User::new_unchecked("test_user".to_string());
        let entity = model_to_entity(&user);

        assert_eq!(entity.username, user.username);
        assert!(entity.created_at > 0);
        assert!(entity.last_used_at > 0);
    }

    #[test]
    fn test_roundtrip() {
        let original_user = User::new_unchecked("roundtrip_test".to_string());
        let entity = model_to_entity(&original_user);
        let converted_user = entity_to_model(&entity);

        assert_eq!(original_user.username, converted_user.username);
    }
}
