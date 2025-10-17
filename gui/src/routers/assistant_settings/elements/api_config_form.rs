//! API configuration form component.

use std::rc::Rc;

use iced::widget::{column, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::app_state::AppState;
use crate::routers::assistant_settings::message::ApiConfigFormMessage;

/// Renders the API configuration form with endpoint, key, and model inputs.
///
/// # Arguments
///
/// * `app_state` - Application state for i18n
/// * `api_endpoint` - Current API endpoint (read-only)
/// * `api_key` - Current API key (secure input)
/// * `api_model_name` - Current API model name
///
/// # Returns
///
/// A column with three rows for endpoint, key, and model inputs
pub fn api_config_form<'a>(
    app_state: &Rc<AppState>,
    api_endpoint: &str,
    api_key: &str,
    api_model_name: &str,
) -> Element<'a, ApiConfigFormMessage> {
    let i18n = app_state.i18n();

    // API Endpoint (read-only)
    let endpoint_label = text(i18n.get("assistant-settings-api-endpoint", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    let display_endpoint = if api_endpoint.is_empty() {
        "https://api.openai.com/v1/responses"
    } else {
        api_endpoint
    };

    let endpoint_input = text_input("https://api.openai.com/v1/responses", display_endpoint)
        // No on_input handler - field is read-only
        .padding(10)
        .width(Length::Fixed(300.0));

    let endpoint_row = row![endpoint_label, endpoint_input]
        .spacing(10)
        .align_y(Alignment::Center);

    // API Key (secure input)
    let key_label = text(i18n.get("assistant-settings-api-key", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    let key_input = text_input("your-api-key", api_key)
        .on_input(ApiConfigFormMessage::ApiKeyChanged)
        .padding(10)
        .width(Length::Fixed(300.0))
        .secure(true);

    let key_row = row![key_label, key_input]
        .spacing(10)
        .align_y(Alignment::Center);

    // API Model Name
    let model_name_label = text(i18n.get("assistant-settings-api-model", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    let model_name_input = text_input("gpt-4", api_model_name)
        .on_input(ApiConfigFormMessage::ApiModelChanged)
        .padding(10)
        .width(Length::Fixed(300.0));

    let model_name_row = row![model_name_label, model_name_input]
        .spacing(10)
        .align_y(Alignment::Center);

    column![endpoint_row, key_row, model_name_row]
        .spacing(20)
        .padding(20)
        .align_x(Alignment::Center)
        .into()
}
