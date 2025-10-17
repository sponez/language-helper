//! Launch modal component showing "Assistant is starting, please wait...".

use std::rc::Rc;

use iced::widget::{button, column, container, text, Container};
use iced::{Alignment, Background, Color, Element, Length};

use crate::app_state::AppState;
use crate::routers::assistant_settings::message::LaunchModalMessage;

/// Status of the assistant launch process
#[derive(Debug, Clone, PartialEq)]
pub enum LaunchStatus {
    Idle,
    CheckingServer,
    StartingServer,
    CheckingModels,
    PromptingPull,
    PullingModel,
    LaunchingModel,
    Complete,
    Error,
}

/// Renders the launch modal overlay.
///
/// This modal displays during the assistant launch process with:
/// - Semi-transparent background overlay
/// - Modal card with progress message
/// - Appropriate buttons based on launch status
///
/// # Arguments
///
/// * `app_state` - Application state for i18n
/// * `launch_status` - Current status of launch process
/// * `launch_progress_message` - Progress message to display
/// * `launch_error_message` - Optional error message
///
/// # Returns
///
/// A full-screen overlay container with the modal
pub fn launch_modal<'a>(
    app_state: &Rc<AppState>,
    launch_status: &LaunchStatus,
    launch_progress_message: &str,
    launch_error_message: &Option<String>,
) -> Element<'a, LaunchModalMessage> {
    let i18n = app_state.i18n();

    // Build modal content based on status (all content centered)
    let mut modal_content = column![]
        .spacing(20)
        .padding(30)
        .align_x(Alignment::Center)
        .width(Length::Fill);

    // Progress message (dynamic content - use shaping)
    let progress_text = text(launch_progress_message.to_string())
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    modal_content = modal_content.push(progress_text);

    // Error message if present (dynamic content - use shaping)
    if let Some(error_msg) = launch_error_message {
        let error_text = text(error_msg.clone())
            .size(14)
            .color(Color::from_rgb(0.8, 0.0, 0.0))
            .shaping(iced::widget::text::Shaping::Advanced);

        modal_content = modal_content.push(error_text);
    }

    // Buttons based on status
    // Only show buttons for specific states where user action is possible
    match launch_status {
        LaunchStatus::PromptingPull => {
            // Show confirm and cancel buttons for model download confirmation
            let download_text = text(i18n.get("assistant-settings-download", None))
                .size(14)
                .shaping(iced::widget::text::Shaping::Advanced);
            let confirm_btn = button(download_text)
                .on_press(LaunchModalMessage::ConfirmPull)
                .padding(10)
                .width(Length::Fixed(120.0));

            let cancel_text = text(i18n.get("assistant-settings-cancel", None))
                .size(14)
                .shaping(iced::widget::text::Shaping::Advanced);
            let cancel_btn = button(cancel_text)
                .on_press(LaunchModalMessage::CancelLaunch)
                .padding(10)
                .width(Length::Fixed(120.0));

            let button_row = iced::widget::row![confirm_btn, cancel_btn]
                .spacing(15)
                .align_y(Alignment::Center);

            modal_content = modal_content.push(button_row);
        }
        LaunchStatus::Complete => {
            // Show close button after successful completion
            let close_text = text(i18n.get("assistant-settings-close", None))
                .size(14)
                .shaping(iced::widget::text::Shaping::Advanced);
            let close_btn = button(close_text)
                .on_press(LaunchModalMessage::CloseModal)
                .padding(10)
                .width(Length::Fixed(120.0));

            let button_row = iced::widget::row![close_btn]
                .spacing(15)
                .align_y(Alignment::Center);

            modal_content = modal_content.push(button_row);
        }
        LaunchStatus::Error => {
            // Show close button after error
            let close_text = text(i18n.get("assistant-settings-close", None))
                .size(14)
                .shaping(iced::widget::text::Shaping::Advanced);
            let close_btn = button(close_text)
                .on_press(LaunchModalMessage::CloseModal)
                .padding(10)
                .width(Length::Fixed(120.0));

            let button_row = iced::widget::row![close_btn]
                .spacing(15)
                .align_y(Alignment::Center);

            modal_content = modal_content.push(button_row);
        }
        _ => {
            // No buttons for in-progress operations (CheckingServer, StartingServer,
            // CheckingModels, PullingModel, LaunchingModel)
            // User cannot cancel these operations
        }
    }

    // Style modal card
    let modal_card = container(modal_content)
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            border: iced::Border {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 2.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        })
        .padding(20)
        .width(Length::Fixed(500.0));

    // Create overlay with semi-transparent background
    let overlay = container(
        Container::new(modal_card)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
        ..Default::default()
    });

    overlay.into()
}
