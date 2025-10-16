//! System requirements checker for AI model compatibility.

use std::process::Command;
use sysinfo::System;

/// Model system requirements
pub struct ModelRequirements {
    pub cpu_cores: usize,
    pub ram_gb: f32,
    pub min_gpu: Option<String>, // Minimum GPU requirement description (not checked, informational only)
    pub total_memory_gb: f32,    // Total system + video memory
}

/// Individual requirement status
#[derive(Debug, Clone)]
pub struct RequirementStatus {
    /// Type of requirement (CPU, RAM, VRAM)
    pub requirement_type: String,
    /// Required amount (e.g., "4 cores", "8GB")
    pub required: String,
    /// Available amount (e.g., "8 cores", "16GB")
    pub available: String,
    /// Whether the requirement is met
    pub is_met: bool,
}

/// System compatibility result
#[derive(Debug, Clone)]
pub struct SystemCompatibility {
    pub model_name: String,
    pub is_compatible: bool,
    pub missing_requirements: Vec<String>,
    /// Detailed status for each requirement
    pub requirement_details: Vec<RequirementStatus>,
}

/// Ollama installation status
#[derive(Debug, Clone)]
pub struct OllamaStatus {
    /// Whether Ollama is installed and accessible
    pub is_installed: bool,
    /// Ollama version if installed (e.g., "0.1.23")
    pub version: Option<String>,
    /// Message to display (installation instructions if not installed)
    pub message: String,
}

/// Get the number of CPU cores
pub fn get_cpu_cores() -> usize {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    sys.cpus().len()
}

/// Get total system RAM in GB (rounded up)
pub fn get_total_ram_gb() -> f32 {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let ram_gb = sys.total_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
    ram_gb.ceil()
}

/// Get GPU VRAM in GB (if available)
/// Note: sysinfo doesn't directly support VRAM detection across all platforms
/// This is a simplified implementation - production would need platform-specific code
pub fn get_total_vram_gb() -> Option<f32> {
    // TODO: Implement proper VRAM detection
    // For now, return None as sysinfo doesn't provide cross-platform GPU memory info
    // Would need platform-specific implementations:
    // - Windows: DXGI or WMI
    // - Linux: nvidia-smi, rocm-smi, or /sys/class/drm
    // - macOS: Metal API
    None
}

/// Get model requirements by model name
fn get_model_requirements(model_name: &str) -> Option<ModelRequirements> {
    match model_name {
        "Tiny" => Some(ModelRequirements {
            cpu_cores: 4,
            ram_gb: 8.0,
            min_gpu: None,
            total_memory_gb: 8.0,
        }),
        "Light" => Some(ModelRequirements {
            cpu_cores: 4,
            ram_gb: 16.0,
            min_gpu: None,
            total_memory_gb: 16.0,
        }),
        "Weak" => Some(ModelRequirements {
            cpu_cores: 6,
            ram_gb: 16.0,
            min_gpu: Some("GPU with 8GB VRAM".to_string()),
            total_memory_gb: 24.0, // 16GB RAM + 8GB GPU
        }),
        "Medium" => Some(ModelRequirements {
            cpu_cores: 6,
            ram_gb: 32.0,
            min_gpu: Some("GPU with 12GB VRAM".to_string()),
            total_memory_gb: 44.0, // 32GB RAM + 12GB GPU
        }),
        "Strong" => Some(ModelRequirements {
            cpu_cores: 6,
            ram_gb: 32.0,
            min_gpu: Some("GPU with 16GB VRAM".to_string()),
            total_memory_gb: 48.0, // 32GB RAM + 16GB GPU
        }),
        _ => None,
    }
}

/// Check if a model can run on the current system
pub fn check_model_compatibility(model_name: &str) -> SystemCompatibility {
    // API model always compatible (uses external API)
    if model_name == "API" {
        return SystemCompatibility {
            model_name: model_name.to_string(),
            is_compatible: true,
            missing_requirements: vec![],
            requirement_details: vec![],
        };
    }

    let requirements = match get_model_requirements(model_name) {
        Some(req) => req,
        None => {
            return SystemCompatibility {
                model_name: model_name.to_string(),
                is_compatible: false,
                missing_requirements: vec![format!("Unknown model: {}", model_name)],
                requirement_details: vec![],
            };
        }
    };

    let cpu_cores = get_cpu_cores();
    let ram_gb = get_total_ram_gb();

    let mut missing = Vec::new();
    let mut details = Vec::new();
    let mut is_compatible = true;

    // Check CPU cores
    let cpu_met = cpu_cores >= requirements.cpu_cores;
    if !cpu_met {
        is_compatible = false;
        missing.push(format!(
            "Insufficient CPU cores: {} required, {} available",
            requirements.cpu_cores, cpu_cores
        ));
    }
    details.push(RequirementStatus {
        requirement_type: "CPU".to_string(),
        required: format!("{} cores", requirements.cpu_cores),
        available: format!("{} cores", cpu_cores),
        is_met: cpu_met,
    });

    // Check RAM
    let ram_met = ram_gb >= requirements.ram_gb;
    if !ram_met {
        is_compatible = false;
        missing.push(format!(
            "Insufficient RAM: {:.1}GB required, {:.1}GB available",
            requirements.ram_gb, ram_gb
        ));
    }
    details.push(RequirementStatus {
        requirement_type: "RAM".to_string(),
        required: format!("{:.1}GB", requirements.ram_gb),
        available: format!("{:.1}GB", ram_gb),
        is_met: ram_met,
    });

    // Check GPU requirement (informational only, doesn't block compatibility)
    if let Some(ref min_gpu_desc) = requirements.min_gpu {
        // Add GPU requirement as informational (always marked as met since we don't block)
        details.push(RequirementStatus {
            requirement_type: "GPU".to_string(),
            required: min_gpu_desc.clone(),
            available: "Not verified".to_string(),
            is_met: true, // Don't block compatibility based on GPU detection
        });
    } else {
        // No GPU required
        details.push(RequirementStatus {
            requirement_type: "GPU".to_string(),
            required: "Not required".to_string(),
            available: "N/A".to_string(),
            is_met: true,
        });
    }

    SystemCompatibility {
        model_name: model_name.to_string(),
        is_compatible,
        missing_requirements: missing,
        requirement_details: details,
    }
}

/// Get all available models (not filtered by compatibility)
/// Use check_model_compatibility() to determine which can run
pub fn get_all_models() -> Vec<String> {
    vec![
        "Tiny".to_string(),
        "Light".to_string(),
        "Weak".to_string(),
        "Medium".to_string(),
        "Strong".to_string(),
        "API".to_string(),
    ]
}

/// Get all compatible models for the current system
/// Deprecated: Use get_all_models() and check each with check_model_compatibility()
pub fn get_compatible_models() -> Vec<String> {
    let all_models = get_all_models();
    all_models
        .into_iter()
        .filter(|model| check_model_compatibility(model).is_compatible)
        .collect()
}

/// Check if Ollama is installed and accessible
///
/// Runs "ollama --version" command to verify installation.
///
/// # Returns
///
/// An `OllamaStatus` containing installation status, version (if installed),
/// and a message with installation instructions if not installed.
pub fn check_ollama_status() -> OllamaStatus {
    match Command::new("ollama").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                // Parse version from output
                let version_output = String::from_utf8_lossy(&output.stdout);
                let version_str = version_output.trim();

                OllamaStatus {
                    is_installed: true,
                    version: Some(version_str.to_string()),
                    message: format!("Ollama is installed: {}", version_str),
                }
            } else {
                // Command executed but returned error
                OllamaStatus {
                    is_installed: false,
                    version: None,
                    message: "Ollama is not installed. To install, go to ollama.com".to_string(),
                }
            }
        }
        Err(_) => {
            // Command not found or couldn't execute
            OllamaStatus {
                is_installed: false,
                version: None,
                message: "Ollama is not installed. To install, go to ollama.com".to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cpu_cores() {
        let cores = get_cpu_cores();
        assert!(cores > 0, "Should detect at least one CPU core");
    }

    #[test]
    fn test_get_total_ram_gb() {
        let ram = get_total_ram_gb();
        assert!(ram > 0.0, "Should detect some RAM");
    }

    #[test]
    fn test_api_model_always_compatible() {
        let result = check_model_compatibility("API");
        assert!(result.is_compatible);
        assert!(result.missing_requirements.is_empty());
    }

    #[test]
    fn test_unknown_model() {
        let result = check_model_compatibility("UnknownModel");
        assert!(!result.is_compatible);
        assert!(!result.missing_requirements.is_empty());
    }

    #[test]
    fn test_get_compatible_models_includes_api() {
        let compatible = get_compatible_models();
        assert!(
            compatible.contains(&"API".to_string()),
            "API should always be compatible"
        );
    }
}
