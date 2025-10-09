//! Language Helper Application
//!
//! This is the main entry point for the Language Helper application.
//! It sets up the dependency injection, initializes all layers, and runs the GUI.

use iced::{Element, Task};

use lh_core::api_impl::{AppApiImpl, UsersApiImpl};
use lh_core::services::user_service::UserService;
use lh_persistence::SqliteUserRepository;

use gui::app_gui;

/// Main iced Application struct
struct LanguageHelperApp {
    state: app_gui::State,
}

impl LanguageHelperApp {
    fn new(app_api: Box<dyn lh_api::app_api::AppApi>) -> (Self, Task<app_gui::Message>) {
        let state = app_gui::State::new(app_api);
        (Self { state }, Task::none())
    }

    fn update(&mut self, message: app_gui::Message) -> Task<app_gui::Message> {
        app_gui::update(&mut self.state, message);
        Task::none()
    }

    fn view(&self) -> Element<'_, app_gui::Message> {
        app_gui::view(&self.state)
    }
}

fn main() -> iced::Result {
    // Initialize the dependency injection chain:
    // Persistence Layer -> Core Layer -> API Layer -> GUI Layer

    // 1. Create the repository (persistence layer)
    let repository = SqliteUserRepository::new("data/users.db")
        .expect("Failed to initialize database");

    // 2. Create the service (core layer)
    let user_service = UserService::new(repository);

    // 3. Create the API implementations (bridge between core and API)
    let users_api = UsersApiImpl::new(user_service);
    let app_api = AppApiImpl::new(users_api);

    // 4. Box the AppApi for trait object usage
    let app_api_boxed: Box<dyn lh_api::app_api::AppApi> = Box::new(app_api);

    // 5. Run the iced application with the injected dependencies
    iced::application(
        "Language Helper",
        LanguageHelperApp::update,
        LanguageHelperApp::view,
    )
    .theme(|_| iced::Theme::default())
    .run_with(|| LanguageHelperApp::new(app_api_boxed))
}
