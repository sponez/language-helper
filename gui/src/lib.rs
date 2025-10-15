//! GUI module for the Language Helper application.
//!
//! This module provides the graphical user interface components for the Language Helper application,
//! built using the Iced framework. It handles user interactions, state management, and rendering
//! of various screens.
//!
//! # Architecture
//!
//! The GUI follows a router stack pattern for hierarchical navigation:
//! - **router**: Core router infrastructure (RouterStack, RouterNode trait)
//! - **routers**: Individual screen routers (self-contained with their own logic)
//!
//! ## Router Stack Pattern
//!
//! The router stack pattern enables scalable, decoupled navigation:
//!
//! ### RouterStack (router.rs)
//! - Manages a stack of active routers
//! - Routes messages to the topmost (current) router
//! - Handles Push (navigate deeper), Pop (go back), Exit events
//! - Contains NO business logic
//!
//! ### Routers (routers/*)
//! - Self-contained screen components
//! - Own their state and business logic
//! - Handle API calls internally
//! - Know ONLY about their immediate child routers
//! - Emit RouterEvents (Push child, Pop, Exit)
//! - Define their own Messages for internal communication
//!
//! ## Navigation Flow
//!
//! ```text
//! [AccountListRouter]
//!     ↓ user selects account
//! [AccountListRouter] → [AccountRouter]
//!     ↓ user clicks settings
//! [AccountListRouter] → [AccountRouter] → [SettingsRouter]
//!     ↓ back button (Pop event)
//! [AccountListRouter] → [AccountRouter]
//! ```
//!
//! ## Adding a New Router
//!
//! To add a new router:
//! 1. Create a new router module in `routers/` with:
//!    - Router struct (private fields)
//!    - `Message` enum (internal messages)
//!    - `new()`, `update()`, `view()` methods
//! 2. Implement RouterNode trait for the router in `router.rs`
//! 3. Add variant to global `Message` enum in `router.rs`
//! 4. Parent router pushes child router: `Some(RouterEvent::Push(Box::new(ChildRouter::new(...))))`
//!
//! # Example
//!
//! ```no_run
//! use gui::router::{RouterStack, RouterNode};
//! use gui::routers::user_list_router::UserListRouter;
//! use lh_api::app_api::AppApi;
//! use std::sync::Arc;
//!
//! fn initialize_gui(api: Box<dyn AppApi>) -> RouterStack {
//!     let api_arc = Arc::from(api);
//!     let root_router: Box<dyn RouterNode> = Box::new(UserListRouter::new(api_arc));
//!     RouterStack::new(root_router)
//! }
//! ```

pub mod app_state;
pub mod i18n;
pub mod languages;
pub mod mappers;
pub mod models;
pub mod router;
pub mod routers;
pub mod runtime_util;
