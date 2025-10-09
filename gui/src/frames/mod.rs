//! Frame modules for different screens in the application.
//!
//! This module contains individual frame components that represent different screens
//! or views in the application. Each frame is responsible for:
//! - Managing its own state
//! - Rendering its UI
//! - Defining its specific messages
//! - Emitting events to the orchestrator for screen transitions
//!
//! # Architecture
//!
//! Frames follow the orchestrator pattern where:
//! - **Frames** are self-contained UI components with their own state and logic
//! - **Messages** are internal to each frame for handling user interactions
//! - **Events** are emitted to the orchestrator to signal state transitions
//! - **Orchestrator** coordinates between frames based on events
//!
//! # Available Frames
//!
//! - **account_list_frame**: User account selection and creation screen
//! - **account_frame**: User account management screen

pub mod account_list_frame;
pub mod account_frame;
