// Modules
pub mod api;
// pub mod audio; // TODO: Fix audio module compilation errors
pub mod auth;
pub mod commands;
pub mod db;
pub mod sync;

use commands::{AppState, AuthState};
use db::Database;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize database
    let db_path = dirs::data_local_dir()
        .expect("Failed to get data directory")
        .join("hinotes")
        .join("hinotes.db");

    // Create directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create database directory");
    }

    let db = Database::new(&db_path).expect("Failed to initialize database");

    let app_state = AppState { db: Mutex::new(db) };

    // Initialize API client and OAuth handler for authentication
    let api_client = api::client::HiNotesClient::new("https://api.hinotes.app/v1");
    let oauth_handler = auth::oauth::OAuth2Handler::new("hinotes-desktop-client-id");

    let auth_state = AuthState {
        api_client: Arc::new(RwLock::new(api_client)),
        oauth_handler: Arc::new(oauth_handler),
    };

    tauri::Builder::default()
        .manage(app_state)
        .manage(auth_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            // Authentication commands
            commands::authenticate_with_credentials,
            commands::authenticate_google,
            commands::authenticate_apple,
            // Calendar commands
            commands::get_calendar_events,
            commands::get_today_events,
            commands::create_calendar_event,
            commands::update_calendar_event,
            commands::delete_calendar_event,
            // commands::get_audio, // TODO: Fix audio module compilation errors first
            commands::list_templates,
            commands::get_template,
            commands::get_default_template,
            commands::create_template,
            commands::update_template,
            commands::toggle_template_favorite,
            commands::set_template_default,
            commands::delete_template,
            // Audio processing commands - TODO: Fix audio module compilation errors first
            // commands::merge_audio_files,
            // commands::replace_audio_segment,
            // commands::save_audio_as_new,
            // commands::get_audio_duration,
            // commands::trim_audio,
            // commands::convert_audio_format,
            // commands::cleanup_audio_temp_files,
            // commands::verify_ffmpeg,
            // Smart Labels commands
            commands::list_smart_labels,
            commands::get_smart_label,
            commands::create_smart_label,
            commands::update_smart_label,
            commands::delete_smart_label,
            // Vocabulary commands
            commands::list_vocabulary,
            commands::get_vocabulary,
            commands::create_vocabulary,
            commands::delete_vocabulary,
            commands::export_vocabulary,
            commands::import_vocabulary,
            // Notes commands
            commands::list_notes,
            commands::list_notes_by_folder,
            commands::get_note,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::count_notes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
