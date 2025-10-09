//! GUI module for the Language Helper application.
//!
//! This module provides the graphical user interface components for the Language Helper application,
//! built using the Iced framework. It handles user interactions, state management, and rendering
//! of various screens.
//!
//! # Architecture
//!
//! The GUI follows a clean orchestrator pattern:
//! - **gui_orchestrator**: Main orchestration layer for routing messages and managing screen transitions
//! - **frames**: Individual screen components (self-contained with their own logic)
//!
//! ## Orchestrator Pattern
//!
//! The orchestrator pattern separates concerns cleanly:
//!
//! ### Orchestrator (gui_orchestrator)
//! - Manages which screen is displayed
//! - Routes messages to appropriate frames
//! - Handles screen transitions based on frame events
//! - Contains NO business logic
//!
//! ### Frames (frames/*)
//! - Self-contained UI components
//! - Own their state and business logic
//! - Handle API calls internally
//! - Emit events to orchestrator for screen transitions
//! - Define their own Messages for internal communication
//!
//! ## Adding a New Screen
//!
//! To add a new screen:
//! 1. Create a new frame module in `frames/` with:
//!    - `State` struct (private fields)
//!    - `Message` enum (internal messages)
//!    - `FrameEvent` enum (events for orchestrator)
//!    - `new()`, `update()`, `view()` methods
//! 2. Add variant to `Screen` enum in orchestrator
//! 3. Add variant to orchestrator's `Message` enum
//! 4. Add routing logic in orchestrator's `update()` and `view()`
//!
//! # Example
//!
//! ```no_run
//! use gui::gui_orchestrator::State;
//! use lh_api::app_api::AppApi;
//!
//! fn initialize_gui(api: Box<dyn AppApi>) -> State {
//!     State::new(api)
//! }
//! ```

pub mod frames;
pub mod gui_orchestrator;
