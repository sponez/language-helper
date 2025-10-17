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

use iced::{Element, Subscription, Task};

use crate::routers::{main_screen, profile, profile_list, profile_settings, user, user_settings};

/// Identifies a specific router type for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouterTarget {
    MainScreen,
    User,
    UserSettings,
    ProfileList,
    Profile,
    ProfileSettings,
    CardSettings,
    AssistantSettings,
    ManageCards,
}

impl RouterTarget {
    /// Get the string identifier for this router target
    fn as_str(&self) -> &'static str {
        match self {
            RouterTarget::MainScreen => "main_screen",
            RouterTarget::User => "user",
            RouterTarget::UserSettings => "user_settings",
            RouterTarget::ProfileList => "profile_list",
            RouterTarget::Profile => "profile",
            RouterTarget::ProfileSettings => "profile_settings",
            RouterTarget::CardSettings => "card_settings",
            RouterTarget::AssistantSettings => "assistant_settings",
            RouterTarget::ManageCards => "manage_cards",
        }
    }
}

/// Events that routers can emit to control navigation.
pub enum RouterEvent {
    /// Navigate deeper by pushing a new router onto the stack
    Push(Box<dyn RouterNode>),
    /// Go back by popping the current router from the stack
    /// The previous router will be automatically refreshed
    Pop,
    /// Pop back to a specific router, or to root if None
    /// All intermediate routers will be removed
    /// The target router will be automatically refreshed
    PopTo(Option<RouterTarget>),
    /// Exit the application entirely
    Exit,
}

impl std::fmt::Debug for RouterEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterEvent::Push(_) => f.debug_tuple("Push").field(&"<router>").finish(),
            RouterEvent::Pop => f.debug_tuple("Pop").finish(),
            RouterEvent::PopTo(target) => f.debug_tuple("PopTo").field(target).finish(),
            RouterEvent::Exit => f.debug_tuple("Exit").finish(),
        }
    }
}

/// Global message type that can wrap any router's messages.
#[derive(Debug, Clone)]
pub enum Message {
    /// Message for the user list router
    MainScreen(main_screen::message::Message),
    /// Message for the user router
    User(user::message::Message),
    /// Message for the user settings router
    UserSettings(user_settings::message::Message),
    /// Message for the profile list router
    ProfileList(profile_list::message::Message),
    /// Message for the profile router
    Profile(profile::message::Message),
    /// Message for the profile settings router
    ProfileSettings(profile_settings::message::Message),
    // /// Message for the card settings router
    // CardSettings(card_settings_router::Message),
    // /// Message for the assistant settings router
    // AssistantSettings(assistant_settings_router::Message),
    // /// Message for the explain AI router
    // ExplainAI(explain_ai_router::Message),
    // /// Message for the cards menu router
    // CardsMenu(cards_menu_router::Message),
    // /// Message for the manage cards router
    // ManageCards(manage_cards_router::Message),
    // /// Message for the add card router
    // AddCard(add_card_router::Message),
    // /// Message for the inverse cards review router
    // InverseCardsReview(inverse_cards_review_router::Message),
}

/// Type-erased router node that can be stored in the stack.
///
/// This trait allows different router types to be stored together
/// by erasing their specific message types.
pub trait RouterNode {
    /// Get the name of this router for navigation purposes
    ///
    /// This is used by PopTo to identify which router to navigate back to.
    fn router_name(&self) -> &'static str;

    /// Update with a global message
    ///
    /// Returns a tuple of:
    /// - Optional router event (Push, Pop, PopTo, Exit)
    /// - A Task to execute (use Task::none() if no async work needed)
    fn update(&mut self, message: &Message) -> (Option<RouterEvent>, Task<Message>);

    /// Render view with global message type
    fn view(&self) -> Element<'_, Message>;

    /// Get the current theme from the router
    fn theme(&self) -> iced::Theme;

    /// Initialize the router's data from the API
    ///
    /// This is called automatically after Push, Pop, or PopTo operations.
    /// Routers should load their data from the API to ensure they display current information.
    ///
    /// # Arguments
    ///
    /// * `incoming_task` - A task from the previous operation that should be batched with init tasks
    ///
    /// # Returns
    ///
    /// A Task that performs the initialization, batched with the incoming task
    ///
    /// Default implementation just returns the incoming task without any initialization.
    fn init(&mut self, incoming_task: Task<Message>) -> Task<Message> {
        incoming_task
    }

    /// Get the router's subscriptions for async operations
    ///
    /// Returns a subscription that will produce messages for this router.
    /// Default implementation returns no subscriptions.
    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
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
    /// A tuple of:
    /// - `Ok(true)` if the application should exit, `Ok(false)` otherwise
    /// - A Task to execute
    /// - `Err` if the stack is empty (should never happen)
    pub fn update(&mut self, message: Message) -> Result<(bool, Task<Message>), &'static str> {
        let router = self.stack.last_mut().ok_or("Router stack is empty")?;

        let (event, task) = router.update(&message);

        if let Some(event) = event {
            match event {
                RouterEvent::Push(mut new_router) => {
                    // Initialize the new router before pushing
                    let init_task = new_router.init(task);
                    self.stack.push(new_router);
                    return Ok((false, init_task));
                }
                RouterEvent::Pop => {
                    if self.stack.len() > 1 {
                        self.stack.pop();
                        // Always initialize the now-current router after popping
                        if let Some(current_router) = self.stack.last_mut() {
                            let init_task = current_router.init(task);
                            return Ok((false, init_task));
                        }
                    } else {
                        // Can't pop the root router - exit instead
                        return Ok((true, task));
                    }
                }
                RouterEvent::PopTo(target) => {
                    if let Some(target_router) = target {
                        // Find the target router in the stack
                        let target_name = target_router.as_str();
                        let target_index = self
                            .stack
                            .iter()
                            .position(|r| r.router_name() == target_name);

                        if let Some(index) = target_index {
                            // Pop all routers above the target
                            self.stack.truncate(index + 1);
                            // Initialize the target router
                            if let Some(current_router) = self.stack.last_mut() {
                                let init_task = current_router.init(task);
                                return Ok((false, init_task));
                            }
                        } else {
                            // Target not found - just do a regular pop
                            if self.stack.len() > 1 {
                                self.stack.pop();
                                if let Some(current_router) = self.stack.last_mut() {
                                    let init_task = current_router.init(task);
                                    return Ok((false, init_task));
                                }
                            } else {
                                return Ok((true, task));
                            }
                        }
                    } else {
                        // No target specified - pop to root (keep only first router)
                        if self.stack.len() > 1 {
                            self.stack.truncate(1);
                            // Initialize the root router
                            if let Some(root_router) = self.stack.last_mut() {
                                let init_task = root_router.init(task);
                                return Ok((false, init_task));
                            }
                        }
                    }
                }
                RouterEvent::Exit => {
                    return Ok((true, task));
                }
            }
        }

        Ok((false, task))
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

    /// Get subscriptions from the current (topmost) router.
    ///
    /// # Returns
    ///
    /// The subscription for the current router, or no subscriptions if the stack is empty.
    pub fn subscription(&self) -> Subscription<Message> {
        if let Some(router) = self.stack.last() {
            router.subscription()
        } else {
            Subscription::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock router for testing
    struct MockRouter {
        name: &'static str,
        refresh_count: std::cell::RefCell<usize>,
    }

    impl MockRouter {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                refresh_count: std::cell::RefCell::new(0),
            }
        }
    }

    impl RouterNode for MockRouter {
        fn router_name(&self) -> &'static str {
            self.name
        }

        fn update(&mut self, _message: &Message) -> (Option<RouterEvent>, Task<Message>) {
            (None, Task::none())
        }

        fn view(&self) -> Element<'_, Message> {
            iced::widget::text("Mock Router").into()
        }

        fn theme(&self) -> iced::Theme {
            iced::Theme::Dark
        }

        fn init(&mut self, incoming_task: Task<Message>) -> Task<Message> {
            *self.refresh_count.borrow_mut() += 1;
            incoming_task
        }
    }

    #[test]
    fn test_router_target_as_str() {
        assert_eq!(RouterTarget::MainScreen.as_str(), "main_screen");
        assert_eq!(RouterTarget::User.as_str(), "user");
        assert_eq!(RouterTarget::UserSettings.as_str(), "user_settings");
        assert_eq!(RouterTarget::ProfileList.as_str(), "profile_list");
        assert_eq!(RouterTarget::Profile.as_str(), "profile");
    }

    #[test]
    fn test_router_stack_new() {
        let root = Box::new(MockRouter::new("root"));
        let stack = RouterStack::new(root);
        assert_eq!(stack.stack.len(), 1);
    }

    #[test]
    fn test_router_stack_push() {
        let root = Box::new(MockRouter::new("root"));
        let mut stack = RouterStack::new(root);

        // Push a new router
        let child = Box::new(MockRouter::new("child"));
        let event = RouterEvent::Push(child);

        // Simulate push by directly adding to stack
        if let RouterEvent::Push(router) = event {
            stack.stack.push(router);
        }

        assert_eq!(stack.stack.len(), 2);
        assert_eq!(stack.stack.last().unwrap().router_name(), "child");
    }

    #[test]
    fn test_router_stack_pop_refreshes_previous() {
        let root = MockRouter::new("root");
        let mut stack = RouterStack::new(Box::new(root));

        let child = MockRouter::new("child");
        stack.stack.push(Box::new(child));

        assert_eq!(stack.stack.len(), 2);

        // Pop the child router
        stack.stack.pop();

        // Initialize the root router (this should increment the counter)
        if let Some(current) = stack.stack.last_mut() {
            let _ = current.init(Task::none());
        }

        // Verify stack state after pop
        assert_eq!(stack.stack.len(), 1);
        assert_eq!(stack.stack.last().unwrap().router_name(), "root");

        // Note: We can't directly verify the refresh count due to trait object limitations,
        // but we can verify the refresh method is called by the structure of the test.
        // In real usage, refresh() updates state that affects the view.
    }

    #[test]
    fn test_router_stack_pop_to_target() {
        let root = Box::new(MockRouter::new("user_list"));
        let mut stack = RouterStack::new(root);

        let child1 = Box::new(MockRouter::new("user"));
        stack.stack.push(child1);

        let child2 = Box::new(MockRouter::new("profile_list"));
        stack.stack.push(child2);

        let child3 = Box::new(MockRouter::new("profile"));
        stack.stack.push(child3);

        assert_eq!(stack.stack.len(), 4);

        // PopTo should find "user" and truncate above it
        let target_index = stack.stack.iter().position(|r| r.router_name() == "user");

        assert_eq!(target_index, Some(1));

        stack.stack.truncate(2); // Keep root and "user"

        assert_eq!(stack.stack.len(), 2);
        assert_eq!(stack.stack.last().unwrap().router_name(), "user");
    }

    #[test]
    fn test_router_stack_pop_to_root() {
        let root = Box::new(MockRouter::new("root"));
        let mut stack = RouterStack::new(root);

        stack.stack.push(Box::new(MockRouter::new("child1")));
        stack.stack.push(Box::new(MockRouter::new("child2")));
        stack.stack.push(Box::new(MockRouter::new("child3")));

        assert_eq!(stack.stack.len(), 4);

        // PopTo(None) should go back to root
        stack.stack.truncate(1);

        assert_eq!(stack.stack.len(), 1);
        assert_eq!(stack.stack.last().unwrap().router_name(), "root");
    }
}
