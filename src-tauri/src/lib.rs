// Modules
pub mod api;
pub mod audio;
pub mod auth;
pub mod commands;
pub mod db;
pub mod referral;
pub mod speaker;
pub mod subscription;
pub mod sync;
pub mod translation;
pub mod usb;

use commands::{speaker_commands::SpeakerState, AppState, FFmpegState};
use db::Database;
use std::sync::Mutex;

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
    let ffmpeg_state = FFmpegState::new();
    let speaker_state = SpeakerState::default();

    tauri::Builder::default()
        .manage(app_state)
        .manage(ffmpeg_state)
        .manage(speaker_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            // FFmpeg commands
            commands::ffmpeg_validate,
            commands::ffmpeg_binary_path,
            commands::ffmpeg_convert_audio,
            commands::ffmpeg_merge_audio,
            commands::ffmpeg_extract_segment,
            commands::ffmpeg_get_audio_info,
            // Calendar commands
            commands::get_calendar_events,
            commands::get_today_events,
            commands::create_calendar_event,
            commands::update_calendar_event,
            commands::delete_calendar_event,
            // Template commands
            commands::list_templates,
            commands::get_template,
            commands::get_default_template,
            commands::create_template,
            commands::update_template,
            commands::toggle_template_favorite,
            commands::set_template_default,
            commands::delete_template,
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
            commands::count_notes,
            // Sharing commands
            commands::create_share_link,
            commands::list_share_links,
            commands::get_shared_note,
            commands::delete_share_link,
            commands::cleanup_expired_shares,
            // Device commands
            commands::list_devices,
            commands::get_device,
            commands::bind_device,
            commands::unbind_device,
            commands::update_device_status,
            commands::update_device_last_sync,
            // Translation commands
            // TODO: Fix Send bounds for translate_text, clear_translation_cache, get_cache_stats
            // commands::translate_text,
            commands::get_supported_languages,
            commands::set_target_language,
            commands::get_target_language,
            // commands::clear_translation_cache,
            // commands::get_cache_stats,
            commands::start_translation_session,
            commands::end_translation_session,
            commands::get_active_translation_session,
            commands::get_translation_segments,
            commands::list_translation_sessions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
