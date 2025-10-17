//! Local model system requirements display component.

use std::collections::HashMap;
use std::rc::Rc;

use iced::widget::{button, column, row, text};
use iced::{Alignment, Color, Element};
use lh_api::models::system_requirements::{OllamaStatusDto, SystemCompatibilityDto};

use crate::app_state::AppState;
use crate::routers::assistant_settings::message::LocalModelRequirementsMessage;

/// Renders system requirements for local Ollama models.
///
/// # Arguments
///
/// * `app_state` - Application state for i18n
/// * `selected_model` - Currently selected model name
/// * `model_compatibility` - Compatibility info for all models
/// * `ollama_status` - Ollama installation status
///
/// # Returns
///
/// A column displaying system requirements with color-coded status
pub fn local_model_requirements<'a>(
    app_state: &Rc<AppState>,
    selected_model: &str,
    model_compatibility: &HashMap<String, SystemCompatibilityDto>,
    ollama_status: &Option<OllamaStatusDto>,
) -> Element<'a, LocalModelRequirementsMessage> {
    let i18n = app_state.i18n();

    let compat = model_compatibility.get(selected_model).cloned();

    let mut requirements_column = column![].spacing(10).padding(20).align_x(Alignment::Start);

    if let Some(compat_info) = compat {
        // Title
        let requirements_title = text(i18n.get("assistant-settings-requirements-title", None))
            .size(18)
            .shaping(iced::widget::text::Shaping::Advanced);
        requirements_column = requirements_column.push(requirements_title);

        // Display each requirement detail
        for requirement in &compat_info.requirement_details {
            // GPU requirements are informational, not pass/fail
            let is_gpu = requirement.requirement_type == "GPU";

            let (status_color, requirement_text) = if is_gpu {
                // GPU requirement - show as informational (blue/gray)
                let gpu_text = format!(
                    "ℹ {}: {}",
                    requirement.requirement_type, requirement.required
                );
                (Color::from_rgb(0.4, 0.4, 0.8), gpu_text)
            } else {
                // CPU/RAM - show pass/fail with colors
                let status_symbol = if requirement.is_met { "[OK]" } else { "[X]" };
                let status_color = if requirement.is_met {
                    Color::from_rgb(0.0, 0.8, 0.0) // Green
                } else {
                    Color::from_rgb(0.8, 0.0, 0.0) // Red
                };

                let req_text = format!(
                    "{} {}: {} required, {} available",
                    status_symbol,
                    requirement.requirement_type,
                    requirement.required,
                    requirement.available
                );
                (status_color, req_text)
            };

            // Dynamic content - use shaping
            let requirement_text_widget = text(requirement_text)
                .size(14)
                .color(status_color)
                .shaping(iced::widget::text::Shaping::Advanced);

            let requirement_row = row![requirement_text_widget]
                .spacing(5)
                .align_y(Alignment::Center);

            requirements_column = requirements_column.push(requirement_row);
        }

        // Overall compatibility message
        if !compat_info.is_compatible {
            let incompatible_message = text(i18n.get("assistant-settings-incompatible", None))
                .size(14)
                .shaping(iced::widget::text::Shaping::Advanced);
            requirements_column = requirements_column
                .push(text("").size(10)) // Spacer
                .push(incompatible_message);
        }

        // Ollama status display
        requirements_column = requirements_column.push(text("").size(10)); // Spacer

        let ollama_is_installed = ollama_status
            .as_ref()
            .map(|s| s.is_installed)
            .unwrap_or(false);

        if ollama_is_installed {
            // Show installed message in green (dynamic content - use shaping)
            let ollama_message = ollama_status
                .as_ref()
                .map(|s| s.message.clone())
                .unwrap_or_default();

            let ollama_text_widget = text(ollama_message)
                .size(14)
                .color(Color::from_rgb(0.0, 0.8, 0.0))
                .shaping(iced::widget::text::Shaping::Advanced);

            requirements_column = requirements_column.push(ollama_text_widget);
        } else {
            // Show "not installed" message with clickable link
            let not_installed_prefix =
                text(i18n.get("assistant-settings-ollama-not-installed", None))
                    .size(14)
                    .shaping(iced::widget::text::Shaping::Advanced);

            let link_text = text("ollama.com")
                .size(14)
                .color(Color::from_rgb(0.2, 0.4, 0.8))
                .shaping(iced::widget::text::Shaping::Advanced);

            let link_button = button(link_text)
                .on_press(LocalModelRequirementsMessage::OpenOllamaUrl(
                    "https://ollama.com".to_string(),
                ))
                .style(button::text);

            let ollama_row = row![not_installed_prefix, link_button]
                .spacing(5)
                .align_y(Alignment::Center);

            requirements_column = requirements_column.push(ollama_row);
        }
    } else {
        // No compatibility data available
        let no_data_message = text(i18n.get("assistant-settings-no-data", None))
            .size(14)
            .shaping(iced::widget::text::Shaping::Advanced);
        requirements_column = requirements_column.push(no_data_message);
    }

    requirements_column.into()
}
