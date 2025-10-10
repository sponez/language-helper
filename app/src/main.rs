//! Language Helper Application
//!
//! This is the main entry point for the Language Helper application.
//! It sets up the dependency injection, initializes all layers, and runs the GUI.

use std::rc::Rc;

use iced::{Element, Task, window};

use lh_core::api_impl::{AppApiImpl, UsersApiImpl};
use lh_core::repositories::adapters::UserRepositoryAdapter;
use lh_core::services::user_service::UserService;
use lh_persistence::SqliteUserRepository;

use gui::router::{RouterStack, RouterNode, Message};
use gui::routers::account_list_router::AccountListRouter;

mod config;
use config::AppConfig;

/// Main iced Application struct.
///
/// This struct wraps the router stack and implements the Iced application lifecycle.
/// It serves as the bridge between the Iced framework and the application's router-based navigation.
struct LanguageHelperApp {
    /// The router stack managing navigation
    router_stack: RouterStack,
}

impl LanguageHelperApp {
    /// Creates a new Language Helper application instance.
    ///
    /// # Arguments
    ///
    /// * `app_api` - The application API providing access to business logic
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The new `LanguageHelperApp` instance
    /// - An initial task (currently none)
    fn new(app_api: Box<dyn lh_api::app_api::AppApi>) -> (Self, Task<Message>) {
        let app_api_rc = Rc::from(app_api);
        let root_router: Box<dyn RouterNode> = Box::new(AccountListRouter::new(app_api_rc));
        let router_stack = RouterStack::new(root_router);

        (Self { router_stack }, Task::none())
    }

    /// Handles application messages and updates state.
    ///
    /// This method processes user interactions and system events, delegating
    /// to the router stack's update function.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to process
    ///
    /// # Returns
    ///
    /// A task to be executed by the Iced runtime. If the application should exit,
    /// returns a task to close the window.
    fn update(&mut self, message: Message) -> Task<Message> {
        let should_exit = self.router_stack.update(message).unwrap_or(false);

        if should_exit {
            // Close the window to exit the application
            window::get_latest()
                .and_then(|id| window::close(id))
        } else {
            Task::none()
        }
    }

    /// Renders the application's current view.
    ///
    /// This method delegates to the router stack to generate the visual representation
    /// of the current screen.
    ///
    /// # Returns
    ///
    /// An `Element` containing the rendered UI
    fn view(&self) -> Element<'_, Message> {
        self.router_stack.view()
    }
}

/// Main entry point for the Language Helper application.
///
/// This function sets up the complete dependency injection chain, initializing
/// all application layers and starting the Iced GUI runtime.
///
/// # Dependency Injection Flow
///
/// The application follows a clean layered architecture with dependency injection:
///
/// 1. **Persistence Layer**: SQLite repository implementation (returns PersistenceError)
/// 2. **Adapter Layer**: Wraps persistence and maps errors to CoreError
/// 3. **Core Layer**: Business logic services (uses UserRepository trait)
/// 4. **API Layer**: API implementations bridging core and GUI
/// 5. **GUI Layer**: User interface and interaction handling
///
/// # Panics
///
/// This function will panic if the database cannot be initialized.
///
/// # Returns
///
/// Returns `iced::Result` which is `Ok(())` on successful application exit,
/// or an error if the Iced runtime encounters a fatal error.
fn main() -> iced::Result {
    // Load configuration
    let config = AppConfig::from_env();

    // Initialize the dependency injection chain:
    // Persistence Layer -> Adapter -> Core Layer -> API Layer -> GUI Layer

    // 1. Create the persistence repository
    let persistence_repository = SqliteUserRepository::new(&config.database_path)
        .expect("Failed to initialize database");

    // 2. Wrap the persistence repository with the adapter
    let repository = UserRepositoryAdapter::new(persistence_repository);

    // 3. Create the service (core layer)
    let user_service = UserService::new(repository);

    // 4. Create the API implementations (bridge between core and API)
    let users_api = UsersApiImpl::new(user_service);
    let app_api = AppApiImpl::new(users_api);

    // 5. Box the AppApi for trait object usage
    let app_api_boxed: Box<dyn lh_api::app_api::AppApi> = Box::new(app_api);

    // 6. Run the iced application with the injected dependencies
    iced::application(
        "Language Helper",
        LanguageHelperApp::update,
        LanguageHelperApp::view,
    )
    .theme(|_| iced::Theme::default())
    .run_with(|| LanguageHelperApp::new(app_api_boxed))
}
