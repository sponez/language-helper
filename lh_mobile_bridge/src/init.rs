//! Application initialization FFI functions.
//!
//! This module handles the one-time initialization of the Language Helper app,
//! setting up the database, services, and API layer.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};

use lh_core::api_impl::{
    AiAssistantApiImpl, AppApiImpl, AppSettingsApiImpl, ProfilesApiImpl,
    SystemRequirementsApiImpl, UsersApiImpl,
};
use lh_core::repositories::adapters::{
    AppSettingsRepositoryAdapter, ProfileDbRepositoryAdapter, ProfileRepositoryAdapter,
    UserRepositoryAdapter, UserSettingsRepositoryAdapter,
};
use lh_core::services::app_settings_service::AppSettingsService;
use lh_core::services::learning_service::LearningService;
use lh_core::services::profile_service::ProfileService;
use lh_core::services::user_profiles_service::UserProfilesService;
use lh_core::services::user_service::UserService;
use lh_core::services::user_settings_service::UserSettingsService;
use lh_persistence::{
    SqliteAppSettingsRepository, SqliteProfileDbRepository, SqliteProfileRepository,
    SqliteUserRepository, SqliteUserSettingsRepository,
};
use rusqlite::Connection;
use tokio::runtime::Runtime;

use crate::common::{APP_API, INIT, RUNTIME};

/// Initialize the Language Helper application.
///
/// This must be called once before using any other FFI functions.
///
/// # Arguments
///
/// * `db_path` - Null-terminated C string containing the path to the main database file
///
/// # Returns
///
/// Returns `true` on success, `false` on failure.
///
/// # Safety
///
/// This function is unsafe because it:
/// - Dereferences a raw pointer (`db_path`)
/// - Uses mutable static variables
/// - Must be called only once
///
/// # Examples
///
/// From Dart:
/// ```dart
/// final dbPath = '/data/user/0/com.example.app/databases/Language Helper 2/main.db'.toNativeUtf8();
/// final success = initApp(dbPath.cast());
/// ```
#[no_mangle]
pub unsafe extern "C" fn init_app(db_path: *const c_char) -> bool {
    let mut success = false;

    INIT.call_once(|| {
        // Convert C string to Rust string
        let c_str = match CStr::from_ptr(db_path).to_str() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to convert db_path: {}", e);
                return;
            }
        };

        // Initialize Tokio runtime
        let runtime = match Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("Failed to create Tokio runtime: {}", e);
                return;
            }
        };

        // Initialize database and API (same as desktop main.rs)
        let api = match initialize_app_api(c_str) {
            Ok(api) => api,
            Err(e) => {
                eprintln!("Failed to initialize app API: {}", e);
                return;
            }
        };

        RUNTIME = Some(runtime);
        APP_API = Some(Arc::new(api));
        success = true;
    });

    success
}

/// Initialize the AppApi with all dependencies (mirrors desktop main.rs)
fn initialize_app_api(
    db_path: &str,
) -> Result<
    AppApiImpl<
        UserRepositoryAdapter<SqliteUserRepository>,
        AppSettingsRepositoryAdapter<SqliteAppSettingsRepository>,
        UserSettingsRepositoryAdapter<SqliteUserSettingsRepository>,
        ProfileRepositoryAdapter<SqliteProfileRepository>,
        ProfileDbRepositoryAdapter<SqliteProfileDbRepository>,
    >,
    String,
> {
    // 1. Create persistence layer
    let user_persistence = SqliteUserRepository::new(db_path)
        .map_err(|e| format!("Failed to initialize user database: {}", e))?;

    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database connection: {}", e))?;
    let shared_conn = Arc::new(Mutex::new(conn));

    let app_settings_persistence = SqliteAppSettingsRepository::new(shared_conn.clone())
        .map_err(|e| format!("Failed to initialize app settings repository: {}", e))?;
    let user_settings_persistence = SqliteUserSettingsRepository::new(shared_conn.clone())
        .map_err(|e| format!("Failed to initialize user settings repository: {}", e))?;
    let profile_persistence = SqliteProfileRepository::new(shared_conn.clone())
        .map_err(|e| format!("Failed to initialize profile repository: {}", e))?;

    // 2. Wrap with adapters
    let user_repository = UserRepositoryAdapter::new(user_persistence);
    let app_settings_repository = AppSettingsRepositoryAdapter::new(app_settings_persistence);
    let user_settings_repository = UserSettingsRepositoryAdapter::new(user_settings_persistence);
    let profile_repository = ProfileRepositoryAdapter::new(profile_persistence);

    // 3. Create services
    let user_service = UserService::new(user_repository);
    let app_settings_service = AppSettingsService::new(app_settings_repository);

    let app_settings_persistence2 = SqliteAppSettingsRepository::new(shared_conn.clone())
        .map_err(|e| format!("Failed to initialize app settings repository 2: {}", e))?;
    let user_persistence2 = SqliteUserRepository::new(db_path)
        .map_err(|e| format!("Failed to initialize user repository 2: {}", e))?;

    let user_settings_service = UserSettingsService::new(
        user_settings_repository,
        AppSettingsRepositoryAdapter::new(app_settings_persistence2),
        UserRepositoryAdapter::new(user_persistence2),
    );

    let user_persistence3 = SqliteUserRepository::new(db_path)
        .map_err(|e| format!("Failed to initialize user repository 3: {}", e))?;

    let profile_metadata_service = UserProfilesService::new(
        profile_repository,
        UserRepositoryAdapter::new(user_persistence3),
    );

    // Extract data_dir from db_path
    let data_dir = std::path::Path::new(db_path)
        .parent()
        .ok_or("Invalid database path")?
        .to_str()
        .ok_or("Invalid UTF-8 in path")?;

    let profile_db_persistence = SqliteProfileDbRepository::new();
    let profile_db_repository = ProfileDbRepositoryAdapter::new(profile_db_persistence);
    let profile_db_service = ProfileService::new(profile_db_repository, data_dir);

    let profile_db_persistence2 = SqliteProfileDbRepository::new();
    let profile_db_repository2 = ProfileDbRepositoryAdapter::new(profile_db_persistence2);
    let learning_service = LearningService::new(profile_db_repository2);

    // 4. Create API implementations
    let users_api = UsersApiImpl::new(user_service, user_settings_service, profile_metadata_service);
    let profiles_api = ProfilesApiImpl::new(profile_db_service, learning_service);
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

    Ok(app_api)
}
