//! Language Helper Application
//!
//! This is the main entry point for the Language Helper application.
//! It sets up the dependency injection, initializes all layers, and runs the GUI.

use std::rc::Rc;

use iced::{window, Element, Task};

use lh_core::api_impl::{AppApiImpl, AppSettingsApiImpl, UsersApiImpl};
use lh_core::repositories::adapters::{
    AppSettingsRepositoryAdapter, ProfileRepositoryAdapter, UserRepositoryAdapter,
    UserSettingsRepositoryAdapter,
};
use lh_core::services::app_settings_service::AppSettingsService;
use lh_core::services::profile_service::ProfileService;
use lh_core::services::user_service::UserService;
use lh_core::services::user_settings_service::UserSettingsService;
use lh_persistence::{
    SqliteAppSettingsRepository, SqliteProfileRepository, SqliteUserRepository,
    SqliteUserSettingsRepository,
};

use gui::router::{Message, RouterNode, RouterStack};
use gui::routers::user_list_router::UserListRouter;

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
        let root_router: Box<dyn RouterNode> = Box::new(UserListRouter::new(app_api_rc));
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
            window::get_latest().and_then(|id| window::close(id))
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

    fn theme(&self) -> iced::Theme {
        self.router_stack.theme()
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

    // 1. Create a shared database connection
    // UserRepository opens the database and ensures parent directory exists
    let user_persistence = SqliteUserRepository::new(&config.database_path)
        .expect("Failed to initialize user database");

    // Get a reference to the connection for other repositories
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};

    // Open the same database for all other repositories
    let conn = Connection::open(&config.database_path).expect("Failed to open database connection");
    let shared_conn = Arc::new(Mutex::new(conn));

    // Create other persistence repositories sharing the connection
    let app_settings_persistence = SqliteAppSettingsRepository::new(shared_conn.clone())
        .expect("Failed to initialize app settings repository");
    let user_settings_persistence = SqliteUserSettingsRepository::new(shared_conn.clone())
        .expect("Failed to initialize user settings repository");
    let profile_persistence = SqliteProfileRepository::new(shared_conn.clone())
        .expect("Failed to initialize profile repository");

    // 2. Wrap the persistence repositories with adapters
    let user_repository = UserRepositoryAdapter::new(user_persistence);
    let app_settings_repository = AppSettingsRepositoryAdapter::new(app_settings_persistence);
    let user_settings_repository = UserSettingsRepositoryAdapter::new(user_settings_persistence);
    let profile_repository = ProfileRepositoryAdapter::new(profile_persistence);

    // 3. Create the services (core layer)
    let user_service = UserService::new(user_repository);
    let app_settings_service = AppSettingsService::new(app_settings_repository);

    // For UserSettingsService, we need to share the same adapted repositories
    // Create additional repository instances
    let app_settings_persistence2 = SqliteAppSettingsRepository::new(shared_conn.clone())
        .expect("Failed to initialize app settings repository for user settings service");
    let user_persistence2 = SqliteUserRepository::new(&config.database_path)
        .expect("Failed to initialize user repository for user settings service");

    let user_settings_service = UserSettingsService::new(
        user_settings_repository,
        AppSettingsRepositoryAdapter::new(app_settings_persistence2),
        UserRepositoryAdapter::new(user_persistence2),
    );

    // For ProfileService
    let user_persistence3 = SqliteUserRepository::new(&config.database_path)
        .expect("Failed to initialize user repository for profile service");

    let profile_service = ProfileService::new(
        profile_repository,
        UserRepositoryAdapter::new(user_persistence3),
    );

    // 4. Create the API implementations (bridge between core and API)
    let users_api = UsersApiImpl::new(user_service, user_settings_service, profile_service);
    let app_settings_api = AppSettingsApiImpl::new(app_settings_service);
    let app_api = AppApiImpl::new(users_api, app_settings_api);

    // 5. Box the AppApi for trait object usage
    let app_api_boxed: Box<dyn lh_api::app_api::AppApi> = Box::new(app_api);

    // 6. Load embedded fonts
    let fonts = vec![
        // Noto Sans for Latin/Cyrillic
        include_bytes!("../../gui/assets/fonts/NotoSans-Regular.ttf").as_slice(),
        // Noto Sans Arabic
        include_bytes!("../../gui/assets/fonts/NotoSansArabic-Regular.ttf").as_slice(),
        // Noto Sans Devanagari (Hindi)
        include_bytes!("../../gui/assets/fonts/NotoSansDevanagari-Regular.ttf").as_slice(),
        // Noto Sans Bengali
        include_bytes!("../../gui/assets/fonts/NotoSansBengali-Regular.ttf").as_slice(),
        // Noto Sans SC (Chinese Simplified)
        include_bytes!("../../gui/assets/fonts/NotoSansSC-Regular.otf").as_slice(),
        // Noto Sans JP (Japanese)
        include_bytes!("../../gui/assets/fonts/NotoSansJP-Regular.otf").as_slice(),
        // Noto Sans KR (Korean)
        include_bytes!("../../gui/assets/fonts/NotoSansKR-Regular.otf").as_slice(),
        // Noto Sans Thai
        include_bytes!("../../gui/assets/fonts/NotoSansThai-Regular.ttf").as_slice(),
    ];

    // 7. Run the iced application with the injected dependencies and fonts
    iced::application(
        "Language Helper",
        LanguageHelperApp::update,
        LanguageHelperApp::view,
    )
    .theme(LanguageHelperApp::theme)
    .font(fonts[0]) // Noto Sans (default)
    .font(fonts[1]) // Arabic
    .font(fonts[2]) // Devanagari
    .font(fonts[3]) // Bengali
    .font(fonts[4]) // Chinese Simplified (SC)
    .font(fonts[5]) // Japanese (JP)
    .font(fonts[6]) // Korean (KR)
    .font(fonts[7]) // Thai
    .run_with(|| LanguageHelperApp::new(app_api_boxed))
}
