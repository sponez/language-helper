mod commands;
mod error;
mod state;

use lh_bootstrap::{BootstrapBridge, BootstrapConfig};
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let database_path = app.path().app_local_data_dir()?.join("language-helper.db");
            let bridge = BootstrapBridge::create(BootstrapConfig::new(database_path))?;

            app.manage(state::DesktopState::new(bridge));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::users::get_usernames,
            commands::users::create_user
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Language Helper");
}
