//! Language Helper Application
//!
//! This is the main entry point for the Language Helper application.
//! It sets up the dependency injection, initializes all layers, and runs the GUI.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use iced::{Element, Subscription, Task};

use lh_core::api_impl::{
    AiAssistantApiImpl, AppApiImpl, AppSettingsApiImpl, ProfilesApiImpl, SystemRequirementsApiImpl,
    UsersApiImpl,
};
use lh_core::repositories::adapters::{
    AppSettingsRepositoryAdapter, ProfileDbRepositoryAdapter, ProfileRepositoryAdapter,
    UserRepositoryAdapter, UserSettingsRepositoryAdapter,
};
use lh_core::services::app_settings_service::AppSettingsService;
use lh_core::services::profile_service::ProfileService;
use lh_core::services::user_profiles_service::UserProfilesService;
use lh_core::services::user_service::UserService;
use lh_core::services::user_settings_service::UserSettingsService;
use lh_persistence::{
    SqliteAppSettingsRepository, SqliteProfileDbRepository, SqliteProfileRepository,
    SqliteUserRepository, SqliteUserSettingsRepository,
};

use gui::app_state::AppState;
use gui::router::{Message, RouterNode, RouterStack};
use gui::routers::main_screen::router::MainScreenRouter;
use gui::runtime_util::block_on;

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
    /// * `app_api_rc` - The application API providing access to business logic (wrapped in Arc for cloning)
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The new `LanguageHelperApp` instance
    /// - An initial task (currently none)
    fn new(app_api_rc: Arc<dyn lh_api::app_api::AppApi>) -> (Self, Task<Message>) {
        // Load initial app settings to create AppState
        let app_settings = block_on(app_api_rc.app_settings_api().get_app_settings())
            .expect("Failed to load app settings");

        // Create global app state
        let app_state = AppState::new(app_settings.theme, app_settings.language);

        let root_router: Box<dyn RouterNode> =
            Box::new(MainScreenRouter::new(app_api_rc, app_state));
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
        match self.router_stack.update(message) {
            Ok((should_exit, task)) => {
                if should_exit {
                    // Exit the application
                    Task::batch(vec![task, iced::exit()])
                } else {
                    task
                }
            }
            Err(e) => {
                eprintln!("Router stack error: {}", e);
                Task::none()
            }
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

    /// Returns the application window title.
    ///
    /// # Returns
    ///
    /// The title string for the application window
    fn title(&self) -> String {
        String::from("Language Helper")
    }

    /// Returns subscriptions for the current router.
    ///
    /// This method delegates to the router stack to get subscriptions from
    /// the currently active router.
    ///
    /// # Returns
    ///
    /// A `Subscription` that produces messages for the current router
    fn subscription(&self) -> Subscription<Message> {
        self.router_stack.subscription()
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

    // For UserProfilesService (profile metadata)
    let user_persistence3 = SqliteUserRepository::new(&config.database_path)
        .expect("Failed to initialize user repository for profile metadata service");

    let profile_metadata_service = UserProfilesService::new(
        profile_repository,
        UserRepositoryAdapter::new(user_persistence3),
    );

    // For ProfileService (profile database management)
    let profile_db_persistence = SqliteProfileDbRepository::new();
    let profile_db_repository = ProfileDbRepositoryAdapter::new(profile_db_persistence);
    let profile_db_service = ProfileService::new(profile_db_repository, &config.data_dir);

    // 4. Create the API implementations (bridge between core and API)
    let users_api = UsersApiImpl::new(
        user_service,
        user_settings_service,
        profile_metadata_service,
    );
    let profiles_api = ProfilesApiImpl::new(profile_db_service);
    let app_settings_api = AppSettingsApiImpl::new(app_settings_service);
    let system_requirements_api = SystemRequirementsApiImpl::new();
    let ai_assistant_api = AiAssistantApiImpl::new();
    let app_api = AppApiImpl::new(
        users_api,
        app_settings_api,
        profiles_api,
        system_requirements_api,
        ai_assistant_api,
    );

    // 5. Wrap the AppApi in Arc for cloning in the boot closure
    let app_api_rc: Arc<dyn lh_api::app_api::AppApi> = Arc::new(app_api);

    // 6. Load embedded fonts
    let fonts = vec![
        // Noto Sans for Latin/Cyrillic
        include_bytes!("assets/fonts/NotoSans-Regular.ttf").as_slice(),
        // Noto Sans Arabic
        include_bytes!("assets/fonts/NotoSansArabic-Regular.ttf").as_slice(),
        // Noto Sans Devanagari (Hindi)
        include_bytes!("assets/fonts/NotoSansDevanagari-Regular.ttf").as_slice(),
        // Noto Sans Bengali
        include_bytes!("assets/fonts/NotoSansBengali-Regular.ttf").as_slice(),
        // Noto Sans SC (Chinese Simplified)
        include_bytes!("assets/fonts/NotoSansSC-Regular.otf").as_slice(),
        // Noto Sans JP (Japanese)
        include_bytes!("assets/fonts/NotoSansJP-Regular.otf").as_slice(),
        // Noto Sans KR (Korean)
        include_bytes!("assets/fonts/NotoSansKR-Regular.otf").as_slice(),
        // Noto Sans Thai
        include_bytes!("assets/fonts/NotoSansThai-Regular.ttf").as_slice(),
    ];

    // 7. Run the iced application with the injected dependencies and fonts
    iced::application(
        move || LanguageHelperApp::new(app_api_rc.clone()),
        LanguageHelperApp::update,
        LanguageHelperApp::view,
    )
    .title(LanguageHelperApp::title)
    .theme(LanguageHelperApp::theme)
    .subscription(LanguageHelperApp::subscription)
    .font(fonts[0]) // Noto Sans (default)
    .font(fonts[1]) // Arabic
    .font(fonts[2]) // Devanagari
    .font(fonts[3]) // Bengali
    .font(fonts[4]) // Chinese Simplified (SC)
    .font(fonts[5]) // Japanese (JP)
    .font(fonts[6]) // Korean (KR)
    .font(fonts[7]) // Thai
    .run()
}
