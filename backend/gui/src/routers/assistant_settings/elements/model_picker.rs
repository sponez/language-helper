//! Model picker component with compatibility indicators.

use std::collections::HashMap;
use std::rc::Rc;

use iced::widget::{pick_list, row, text};
use iced::{Alignment, Element, Length};
use lh_api::models::system_requirements::SystemCompatibilityDto;

use crate::app_state::AppState;
use crate::routers::assistant_settings::message::ModelPickerMessage;

/// Renders a model picker dropdown with compatibility status indicators.
///
/// # Arguments
///
/// * `app_state` - Application state for i18n
/// * `selected_model` - Currently selected model name
/// * `model_compatibility` - Compatibility info for all models
///
/// # Returns
///
/// A row containing label and picker with [OK]/[X] status indicators
pub fn model_picker<'a>(
    app_state: &Rc<AppState>,
    selected_model: &str,
    model_compatibility: &HashMap<String, SystemCompatibilityDto>,
) -> Element<'a, ModelPickerMessage> {
    let i18n = app_state.i18n();

    // Model selection label
    let model_label = text(i18n.get("assistant-settings-model-label", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    // All available models
    let all_models = ["Tiny", "Light", "Weak", "Medium", "Strong", "API"];

    // Create model options with status indicators
    let model_options: Vec<String> = all_models
        .iter()
        .map(|&model| {
            let localized = match model {
                "Tiny" => i18n.get("assistant-settings-tiny", None),
                "Light" => i18n.get("assistant-settings-light", None),
                "Weak" => i18n.get("assistant-settings-weak", None),
                "Medium" => i18n.get("assistant-settings-medium", None),
                "Strong" => i18n.get("assistant-settings-strong", None),
                "API" => i18n.get("assistant-settings-api", None),
                _ => model.to_string(),
            };

            // Add status indicator for non-API models
            if model == "API" {
                localized
            } else {
                let is_compatible = model_compatibility
                    .get(model)
                    .map(|c| c.is_compatible)
                    .unwrap_or(false);

                if is_compatible {
                    format!("{} [OK]", localized)
                } else {
                    format!("{} [X]", localized)
                }
            }
        })
        .collect();

    // Get current model display with status indicator
    let current_model_localized = match selected_model {
        "Tiny" => i18n.get("assistant-settings-tiny", None),
        "Light" => i18n.get("assistant-settings-light", None),
        "Weak" => i18n.get("assistant-settings-weak", None),
        "Medium" => i18n.get("assistant-settings-medium", None),
        "Strong" => i18n.get("assistant-settings-strong", None),
        "API" => i18n.get("assistant-settings-api", None),
        _ => selected_model.to_string(),
    };

    let current_model_display = if selected_model == "API" {
        current_model_localized
    } else {
        let is_current_compatible = model_compatibility
            .get(selected_model)
            .map(|c| c.is_compatible)
            .unwrap_or(false);

        if is_current_compatible {
            format!("{} [OK]", current_model_localized)
        } else {
            format!("{} [X]", current_model_localized)
        }
    };

    // Get localized texts for picker callback (need to be owned for closure)
    let tiny_text = i18n.get("assistant-settings-tiny", None);
    let light_text = i18n.get("assistant-settings-light", None);
    let weak_text = i18n.get("assistant-settings-weak", None);
    let medium_text = i18n.get("assistant-settings-medium", None);
    let strong_text = i18n.get("assistant-settings-strong", None);
    let api_text = i18n.get("assistant-settings-api", None);

    let model_picker = pick_list(
        model_options,
        Some(current_model_display),
        move |selected| {
            // Strip status indicators and match localized text to model name
            let stripped = selected.trim_end_matches(" [OK]").trim_end_matches(" [X]");

            if stripped == tiny_text {
                ModelPickerMessage::ModelSelected("Tiny".to_string())
            } else if stripped == light_text {
                ModelPickerMessage::ModelSelected("Light".to_string())
            } else if stripped == weak_text {
                ModelPickerMessage::ModelSelected("Weak".to_string())
            } else if stripped == medium_text {
                ModelPickerMessage::ModelSelected("Medium".to_string())
            } else if stripped == strong_text {
                ModelPickerMessage::ModelSelected("Strong".to_string())
            } else if stripped == api_text {
                ModelPickerMessage::ModelSelected("API".to_string())
            } else {
                ModelPickerMessage::ModelSelected(selected)
            }
        },
    )
    .width(Length::Fixed(200.0))
    .text_shaping(iced::widget::text::Shaping::Advanced);

    row![model_label, model_picker]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}
