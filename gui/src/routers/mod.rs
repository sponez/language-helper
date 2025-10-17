//! Router implementations for different application screens.
//!
//! Each module in this directory represents a self-contained screen/router
//! that handles its own logic and navigation to child screens.

pub mod add_card;
pub mod assistant_settings;
pub mod card_settings;
pub mod cards_menu;
pub mod explain_ai;
pub mod inverse_cards_review;
pub mod learn;
pub mod main_screen;
pub mod manage_cards;
pub mod profile;
pub mod profile_list;
pub mod profile_settings;
pub mod user;
pub mod user_settings;

// Legacy routers (to be deleted after refactoring):
// pub mod add_card_router;
// pub mod assistant_settings_router;
// pub mod inverse_cards_review_router;
// pub mod manage_cards_router;
