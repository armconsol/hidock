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

use api::client::HiNotesClient;
use auth::oauth::OAuth2Handler;
use commands::{
    auth_commands::AuthState, calendar_sync_commands::CalendarSyncState,
    speaker_commands::SpeakerState, AppState, FFmpegState,
};
use db::Database;
use std::sync::{Arc, Mutex};
use sync::CalendarSync;
use tokio::sync::{Mutex as TokioMutex, RwLock};
use translation::cache::TranslationCache;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize environment variables from .env file if present
    dotenv::dotenv().ok();

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

    let db_arc = Arc::new(TokioMutex::new(db));
    let app_state = AppState {
        db: Mutex::new(
            Database::new(&db_path).expect("Failed to initialize database for AppState"),
        ),
    };
    let ffmpeg_state = FFmpegState::new();
    let speaker_state = SpeakerState::default();

    // Initialize translation cache
    let translation_cache = TranslationCache::new(db_path.to_str().expect("Invalid DB path"))
        .expect("Failed to initialize translation cache");
    let translation_cache_state = Arc::new(TokioMutex::new(translation_cache));

    // Initialize API client and OAuth handler
    // Uses HINOTES_API_URL environment variable or defaults to production
    let api_client = Arc::new(HiNotesClient::new());
    let oauth_handler = OAuth2Handler::from_env()
        .expect("Failed to initialize OAuth2Handler - ensure GOOGLE_CLIENT_ID is set");

    let auth_state = AuthState {
        api_client: Arc::new(RwLock::new(HiNotesClient::new())),
        oauth_handler: Arc::new(oauth_handler),
    };

    // Initialize calendar sync worker
    let calendar_sync = CalendarSync::new(api_client.clone(), db_arc.clone());
    let calendar_sync_state = CalendarSyncState {
        worker: Arc::new(TokioMutex::new(Some(calendar_sync))),
    };

    tauri::Builder::default()
        .manage(app_state)
        .manage(ffmpeg_state)
        .manage(speaker_state)
        .manage(translation_cache_state)
        .manage(auth_state)
        .manage(calendar_sync_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            // Authentication commands
            commands::auth_commands::authenticate_with_credentials,
            commands::auth_commands::authenticate_google,
            commands::auth_commands::authenticate_apple,
            // User profile commands
            commands::user_commands::get_user_profile,
            commands::user_commands::update_user_name,
            commands::user_commands::update_user_region,
            commands::user_commands::upload_user_avatar,
            // Password and security commands
            commands::user_commands::change_password,
            commands::user_commands::delete_user_account,
            commands::user_commands::send_email_verification,
            commands::user_commands::verify_email_code,
            commands::user_commands::send_password_reset,
            commands::user_commands::verify_reset_code,
            commands::user_commands::save_new_password,
            // Settings commands
            commands::settings_commands::get_user_setting,
            commands::settings_commands::set_user_setting,
            commands::settings_commands::list_user_settings,
            commands::settings_commands::get_ai_engines,
            commands::settings_commands::sync_settings_with_cloud,
            commands::settings_commands::auto_sync_settings,
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
            // Calendar sync commands
            commands::start_calendar_sync,
            commands::stop_calendar_sync,
            commands::get_calendar_sync_status,
            commands::set_calendar_google_token,
            commands::clear_calendar_google_token,
            commands::set_calendar_id,
            commands::sync_calendar_now,
            commands::get_calendar_last_sync,
            commands::notify_calendar_recording,
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
            // Whisper notes commands
            commands::create_whisper_note,
            commands::list_whisper_notes,
            commands::get_whisper_note,
            commands::delete_whisper_note,
            commands::convert_whisper_to_note,
            commands::convert_whisper_to_todo,
            commands::extract_calendar_from_whisper,
            commands::count_whisper_notes,
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
            // Device file commands
            commands::list_device_files,
            commands::download_device_file,
            commands::upload_device_file,
            commands::sync_all_device_files,
            // USB commands
            commands::usb_init,
            commands::usb_scan_devices,
            commands::usb_is_device_connected,
            commands::usb_scan_mass_storage,
            commands::usb_import_audio_file,
            commands::usb_delete_audio_file,
            // Translation commands
            commands::translate_text,
            commands::get_supported_languages,
            commands::set_target_language,
            commands::get_target_language,
            commands::clear_translation_cache,
            commands::get_cache_stats,
            commands::start_translation_session,
            commands::end_translation_session,
            commands::get_active_translation_session,
            commands::get_translation_segments,
            commands::list_translation_sessions,
            // Referral commands
            commands::create_referral_code,
            commands::get_referral_stats,
            commands::track_referral_usage,
            commands::get_user_referral_codes,
            commands::validate_referral_code,
            commands::deactivate_referral_code,
            // Rewards commands
            commands::list_rewards,
            commands::redeem_reward,
            commands::request_payout,
            commands::get_reward_history,
            commands::expire_rewards,
            commands::add_reward
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
