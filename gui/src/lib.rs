//! GUI module for the Language Helper application.
//!
//! This module provides the graphical user interface components for the Language Helper application,
//! built using the Iced framework. It handles user interactions, state management, and rendering
//! of various screens.
//!
//! # Architecture
//!
//! The GUI is structured into:
//! - **app_gui**: Main application state and message routing
//! - **frames**: Individual screen components (e.g., account selection)
//!
//! # Example
//!
//! ```no_run
//! use gui::app_gui::State;
//! use lh_api::app_api::AppApi;
//!
//! fn initialize_gui(api: Box<dyn AppApi>) -> State {
//!     State::new(api)
//! }
//! ```

pub mod frames;
pub mod app_gui;
