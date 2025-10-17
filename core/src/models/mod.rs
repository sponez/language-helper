//! Domain models.
//!
//! This module contains the core business entities and domain logic.

pub mod app_settings;
pub mod assistant_settings;
pub mod card;
pub mod card_settings;
pub mod learning_session;
pub mod profile;
pub mod test_result;
pub mod user;
pub mod user_settings;

pub use app_settings::AppSettings;
pub use assistant_settings::AssistantSettings;
pub use card::{Card, CardType, Meaning, Word};
pub use card_settings::CardSettings;
pub use learning_session::{LearningPhase, LearningSession};
pub use profile::Profile;
pub use test_result::TestResult;
pub use user::User;
pub use user_settings::UserSettings;
