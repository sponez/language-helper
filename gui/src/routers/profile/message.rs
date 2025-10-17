//! Message types for the profile router.

use crate::states::{AssistantState, CardState};

use super::elements::{
    ai_button::AiButtonMessage, cards_button::CardsButtonMessage,
    settings_button::SettingsButtonMessage,
};

/// Messages handled by the profile router
#[derive(Debug, Clone)]
pub enum Message {
    /// Back button pressed
    BackButton,
    /// Cards button pressed
    CardsButton(CardsButtonMessage),
    /// AI button pressed
    AiButton(AiButtonMessage),
    /// Settings button pressed
    SettingsButton(SettingsButtonMessage),

    /// Card state loaded from API
    CardStateLoaded(Result<CardState, String>),
    /// Assistant state loaded from API (None if not configured/running)
    AssistantStateLoaded(Option<AssistantState>),
}
