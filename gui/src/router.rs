//! Router system for navigation stack management.
//!
//! This module provides a hierarchical navigation system where each screen
//! is represented by a router that can push child routers onto a stack.
//! This allows for infinite nesting without coupling screens together.
//!
//! # Architecture
//!
//! - **Router trait**: Common interface all routers must implement
//! - **RouterEvent**: Events routers can emit (Push, Pop, Exit)
//! - **RouterStack**: Manages the stack of active routers
//!
//! # Navigation Flow
//!
//! ```text
//! [RouterA]
//!     ↓ user action triggers Push(RouterB)
//! [RouterA] → [RouterB]
//!     ↓ user action triggers Push(RouterC)
//! [RouterA] → [RouterB] → [RouterC]
//!     ↓ user action triggers Pop
//! [RouterA] → [RouterB]
//!     ↓ user action triggers Pop
//! [RouterA]
//! ```
//!
//! Each router only knows about its immediate children and can push them
//! onto the stack. Routers never need to know about parent or sibling routers.

use iced::Element;

use crate::routers::{user_list_router, user_router};

/// Events that routers can emit to control navigation.
pub enum RouterEvent {
    /// Navigate deeper by pushing a new router onto the stack
    Push(Box<dyn RouterNode>),
    /// Go back by popping the current router from the stack
    Pop,
    /// Exit the application entirely
    Exit,
}

impl std::fmt::Debug for RouterEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterEvent::Push(_) => f.debug_tuple("Push").field(&"<router>").finish(),
            RouterEvent::Pop => f.debug_tuple("Pop").finish(),
            RouterEvent::Exit => f.debug_tuple("Exit").finish(),
        }
    }
}

/// Global message type that can wrap any router's messages.
#[derive(Debug, Clone)]
pub enum Message {
    /// Message for the account list router
    UserList(user_list_router::Message),
    /// Message for the account router
    User(user_router::Message),
}

/// Type-erased router node that can be stored in the stack.
///
/// This trait allows different router types to be stored together
/// by erasing their specific message types.
pub trait RouterNode {
    /// Update with a global message
    fn update(&mut self, message: &Message) -> Option<RouterEvent>;

    /// Render view with global message type
    fn view(&self) -> Element<'_, Message>;

    /// Get the current theme from the router
    fn theme(&self) -> iced::Theme;
}

/// Manages a stack of routers for hierarchical navigation.
///
/// The router stack maintains a vector of active routers and always
/// delegates to the topmost router (the current screen).
pub struct RouterStack {
    /// Stack of active routers (last = top)
    stack: Vec<Box<dyn RouterNode>>,
}

impl RouterStack {
    /// Create a new router stack with an initial root router.
    ///
    /// # Arguments
    ///
    /// * `root_router` - The initial router to display
    pub fn new(root_router: Box<dyn RouterNode>) -> Self {
        Self {
            stack: vec![root_router],
        }
    }

    /// Update the current (topmost) router with a message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to process
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the application should exit
    /// - `Ok(false)` if the update completed normally
    /// - `Err` if the stack is empty (should never happen)
    pub fn update(&mut self, message: Message) -> Result<bool, &'static str> {
        let router = self.stack.last_mut().ok_or("Router stack is empty")?;

        if let Some(event) = router.update(&message) {
            match event {
                RouterEvent::Push(new_router) => {
                    self.stack.push(new_router);
                }
                RouterEvent::Pop => {
                    if self.stack.len() > 1 {
                        self.stack.pop();
                    } else {
                        // Can't pop the root router - exit instead
                        return Ok(true);
                    }
                }
                RouterEvent::Exit => {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Render the view of the current (topmost) router.
    ///
    /// # Returns
    ///
    /// The UI element for the current screen, or an error message if the stack is empty.
    pub fn view(&self) -> Element<'_, Message> {
        if let Some(router) = self.stack.last() {
            router.view()
        } else {
            iced::widget::text("Error: Router stack is empty").into()
        }
    }

    pub fn theme(&self) -> iced::Theme {
        if let Some(router) = self.stack.last() {
            router.theme()
        } else {
            iced::Theme::Dark
        }
    }
}
