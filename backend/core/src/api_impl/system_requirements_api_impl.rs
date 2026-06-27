//! SystemRequirementsApi trait implementation.
//!
//! This module provides the concrete implementation of the SystemRequirementsApi trait
//! using the system_checker utilities from the core layer.

use lh_api::apis::system_requirements_api::SystemRequirementsApi;
use lh_api::errors::api_error::ApiError;
use lh_api::models::system_requirements::{
    OllamaStatusDto, RequirementStatusDto, SystemCompatibilityDto,
};

use crate::services::system_checker;

/// Implementation of the SystemRequirementsApi trait.
///
/// This struct uses the system_checker utilities to check hardware compatibility
/// for different AI models.
pub struct SystemRequirementsApiImpl;

impl SystemRequirementsApiImpl {
    /// Creates a new SystemRequirementsApiImpl instance.
    ///
    /// # Returns
    ///
    /// A new `SystemRequirementsApiImpl` instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemRequirementsApiImpl {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to map system_checker::RequirementStatus to RequirementStatusDto
fn map_requirement_status(status: system_checker::RequirementStatus) -> RequirementStatusDto {
    RequirementStatusDto {
        requirement_type: status.requirement_type,
        required: status.required,
        available: status.available,
        is_met: status.is_met,
    }
}

/// Helper function to map system_checker::SystemCompatibility to SystemCompatibilityDto
fn map_compatibility_to_dto(compat: system_checker::SystemCompatibility) -> SystemCompatibilityDto {
    let requirement_details = compat
        .requirement_details
        .into_iter()
        .map(map_requirement_status)
        .collect();

    SystemCompatibilityDto {
        model_name: compat.model_name,
        is_compatible: compat.is_compatible,
        missing_requirements: compat.missing_requirements,
        requirement_details,
    }
}

impl SystemRequirementsApi for SystemRequirementsApiImpl {
    fn check_model_compatibility(
        &self,
        model_name: &str,
    ) -> Result<SystemCompatibilityDto, ApiError> {
        let compatibility = system_checker::check_model_compatibility(model_name);
        Ok(map_compatibility_to_dto(compatibility))
    }

    fn get_compatible_models(&self) -> Result<Vec<String>, ApiError> {
        let compatible = system_checker::get_compatible_models();
        Ok(compatible)
    }

    fn check_multiple_models(
        &self,
        model_names: &[&str],
    ) -> Result<Vec<SystemCompatibilityDto>, ApiError> {
        let results: Vec<SystemCompatibilityDto> = model_names
            .iter()
            .map(|&model| {
                let compat = system_checker::check_model_compatibility(model);
                map_compatibility_to_dto(compat)
            })
            .collect();

        Ok(results)
    }

    fn check_ollama_status(&self) -> Result<OllamaStatusDto, ApiError> {
        let status = system_checker::check_ollama_status();

        Ok(OllamaStatusDto {
            is_installed: status.is_installed,
            version: status.version,
            message: status.message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_api_model_compatibility() {
        let api = SystemRequirementsApiImpl::new();
        let result = api.check_model_compatibility("API");

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.model_name, "API");
        assert!(dto.is_compatible);
        assert!(dto.missing_requirements.is_empty());
    }

    #[test]
    fn test_check_unknown_model() {
        let api = SystemRequirementsApiImpl::new();
        let result = api.check_model_compatibility("UnknownModel");

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.model_name, "UnknownModel");
        assert!(!dto.is_compatible);
        assert!(!dto.missing_requirements.is_empty());
    }

    #[test]
    fn test_get_compatible_models() {
        let api = SystemRequirementsApiImpl::new();
        let result = api.get_compatible_models();

        assert!(result.is_ok());
        let models = result.unwrap();
        // API should always be compatible
        assert!(models.contains(&"API".to_string()));
    }

    #[test]
    fn test_check_multiple_models() {
        let api = SystemRequirementsApiImpl::new();
        let models = ["Weak", "Medium", "API"];
        let result = api.check_multiple_models(&models);

        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 3);

        // Find the API result
        let api_result = results.iter().find(|r| r.model_name == "API").unwrap();
        assert!(api_result.is_compatible);
    }
}
