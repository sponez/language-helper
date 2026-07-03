mod commands;
mod error;
mod state;

use lh_bootstrap::{BootstrapBridge, BootstrapConfig};
use std::io;
use tauri::Manager;

fn database_path() -> Result<std::path::PathBuf, io::Error> {
    let executable_path = std::env::current_exe()?;
    let executable_directory = executable_path.parent().ok_or_else(|| {
        io::Error::other(format!(
            "executable path has no parent directory: {}",
            executable_path.display()
        ))
    })?;

    Ok(executable_directory.join("language-helper.db"))
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let bridge = BootstrapBridge::create(BootstrapConfig::new(database_path()?))?;

            app.manage(state::DesktopState::new(bridge));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::users::get_usernames,
            commands::users::create_user,
            commands::users::delete_user,
            commands::profiles::list_language_profiles,
            commands::profiles::create_language_profile,
            commands::profiles::delete_language_profile,
            commands::profiles::get_ai_settings,
            commands::profiles::save_ai_settings,
            commands::profiles::get_pronunciation_settings,
            commands::profiles::save_pronunciation_settings,
            commands::cards::list_cards,
            commands::cards::get_card,
            commands::cards::create_cards,
            commands::cards::update_card,
            commands::cards::delete_cards,
            commands::cards::prepare_inverse_cards,
            commands::cards::save_inverse_cards,
            commands::cards::normalize_card,
            commands::speech::get_card_speech,
            commands::sessions::create_study_session,
            commands::sessions::get_study_session_preferences,
            commands::sessions::apply_study_session_action,
            commands::sessions::assess_pronunciation,
            commands::sessions::finish_study_session,
            commands::sessions::cancel_study_session
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Language Helper");
}
