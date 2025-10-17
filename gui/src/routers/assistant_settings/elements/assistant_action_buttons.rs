//! Assistant action buttons component (Start/Stop/Change/Save).

use std::collections::HashMap;
use std::rc::Rc;

use iced::widget::{button, row, text};
use iced::{Alignment, Element, Length};
use lh_api::models::system_requirements::{OllamaStatusDto, SystemCompatibilityDto};

use crate::app_state::AppState;
use crate::routers::assistant_settings::message::AssistantActionButtonsMessage;

/// Helper to map model strength to Ollama model name
fn get_ollama_model_name(model: &str) -> String {
    match model {
        "Tiny" => "phi4-mini".to_string(),
        "Light" => "phi4".to_string(),
        "Weak" => "gemma2:2b".to_string(),
        "Medium" => "aya:8b".to_string(),
        "Strong" => "gemma2:9b".to_string(),
        _ => model.to_string(),
    }
}

/// Determines which button to show and whether it's enabled.
///
/// # Arguments
///
/// * `selected_model` - Currently selected model
/// * `model_compatibility` - Compatibility info for all models
/// * `ollama_status` - Ollama installation status
/// * `running_models` - List of currently running model names
///
/// # Returns
///
/// Option of (button_message, is_enabled)
fn get_button_state(
    selected_model: &str,
    model_compatibility: &HashMap<String, SystemCompatibilityDto>,
    ollama_status: &Option<OllamaStatusDto>,
    running_models: &[String],
) -> Option<(AssistantActionButtonsMessage, bool)> {
    match selected_model {
        "API" => {
            // For API model, show Save button (always enabled)
            Some((AssistantActionButtonsMessage::SavePressed, true))
        }
        "Tiny" | "Light" | "Weak" | "Medium" | "Strong" => {
            // For local models, check if compatible and Ollama installed
            let is_compatible = model_compatibility
                .get(selected_model)
                .map(|c| c.is_compatible)
                .unwrap_or(false);

            let ollama_installed = ollama_status
                .as_ref()
                .map(|s| s.is_installed)
                .unwrap_or(false);

            // Button is enabled only if system is compatible AND Ollama is installed
            let button_enabled = is_compatible && ollama_installed;

            if running_models.is_empty() {
                // No models running - show Start button
                Some((AssistantActionButtonsMessage::StartPressed, button_enabled))
            } else {
                // Get the expected model name for the selected model
                let expected_model_name = get_ollama_model_name(selected_model);

                // Check if the expected model is running
                let is_selected_running = running_models
                    .iter()
                    .any(|m| m.contains(&expected_model_name));

                if is_selected_running {
                    // Selected model is running - show Stop button
                    Some((AssistantActionButtonsMessage::StopPressed, button_enabled))
                } else {
                    // Different model is running - show Change button
                    Some((AssistantActionButtonsMessage::ChangePressed, button_enabled))
                }
            }
        }
        _ => None,
    }
}

/// Renders the assistant action buttons (Start/Stop/Change/Save) and Back button.
///
/// # Arguments
///
/// * `app_state` - Application state for i18n
/// * `selected_model` - Currently selected model
/// * `model_compatibility` - Compatibility info for all models
/// * `ollama_status` - Ollama installation status
/// * `running_models` - List of currently running model names
///
/// # Returns
///
/// A row with action button and back button
pub fn assistant_action_buttons<'a>(
    app_state: &Rc<AppState>,
    selected_model: &str,
    model_compatibility: &HashMap<String, SystemCompatibilityDto>,
    ollama_status: &Option<OllamaStatusDto>,
    running_models: &[String],
) -> Element<'a, AssistantActionButtonsMessage> {
    let i18n = app_state.i18n();

    if let Some((button_message, is_enabled)) = get_button_state(
        selected_model,
        model_compatibility,
        ollama_status,
        running_models,
    ) {
        // Determine button text based on message type
        let button_text_key = match button_message {
            AssistantActionButtonsMessage::StartPressed => "assistant-settings-start-assistant",
            AssistantActionButtonsMessage::StopPressed => "assistant-settings-stop-assistant",
            AssistantActionButtonsMessage::ChangePressed => "assistant-settings-change-assistant",
            AssistantActionButtonsMessage::SavePressed => "assistant-settings-save-api",
        };

        let button_text = text(i18n.get(button_text_key, None))
            .size(14)
            .shaping(iced::widget::text::Shaping::Advanced);

        let mut action_button = button(button_text).width(Length::Fixed(160.0)).padding(10);

        if is_enabled {
            action_button = action_button.on_press(button_message);
        }

        row![action_button]
            .spacing(15)
            .align_y(Alignment::Center)
            .into()
    } else {
        // No action button
        row![].spacing(15).align_y(Alignment::Center).into()
    }
}
