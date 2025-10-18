//! API configuration form component.

use std::rc::Rc;

use iced::widget::{column, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::app_state::AppState;
use crate::routers::assistant_settings::message::ApiConfigFormMessage;

/// API provider options for the picklist
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiProvider {
    OpenAI,
    Gemini,
}

impl ApiProvider {
    const ALL: [ApiProvider; 2] = [ApiProvider::OpenAI, ApiProvider::Gemini];

    fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "gemini" => ApiProvider::Gemini,
            _ => ApiProvider::OpenAI, // Default to OpenAI
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            ApiProvider::OpenAI => "openai",
            ApiProvider::Gemini => "gemini",
        }
    }
}

impl std::fmt::Display for ApiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Renders the API configuration form with provider, key, and model inputs.
///
/// # Arguments
///
/// * `app_state` - Application state for i18n
/// * `api_provider` - Current API provider (openai/gemini)
/// * `api_key` - Current API key (secure input)
/// * `api_model_name` - Current API model name
///
/// # Returns
///
/// A column with rows for provider selection, key, and model inputs
pub fn api_config_form<'a>(
    app_state: &Rc<AppState>,
    api_provider: &str,
    api_key: &str,
    api_model_name: &str,
) -> Element<'a, ApiConfigFormMessage> {
    let i18n = app_state.i18n();
    let selected_provider = ApiProvider::parse(api_provider);

    // API Provider picklist
    let provider_label = text(i18n.get("assistant-settings-provider-label", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    let provider_picklist = pick_list(&ApiProvider::ALL[..], Some(selected_provider), |provider| {
        ApiConfigFormMessage::ApiProviderChanged(provider.as_str().to_string())
    })
    .padding(10)
    .width(Length::Fixed(300.0))
    .placeholder(i18n.get("assistant-settings-provider-openai", None));

    let provider_row = row![provider_label, provider_picklist]
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

    // Build form column
    column![provider_row, key_row, model_name_row]
        .spacing(20)
        .padding(20)
        .align_x(Alignment::Center)
        .into()
}
