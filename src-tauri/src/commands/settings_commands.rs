// Settings sync commands for bidirectional cloud sync

use crate::api::client::HiNotesClient;
use crate::api::types::AIEngine;
use crate::commands::AppState;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Setting keys that should be synced with cloud
const SYNCED_SETTINGS: &[&str] = &[
    "theme",
    "language",
    "transcription_engine",
    "auto_translation_enabled",
    "recording_quality",
    "calendar_sync_enabled",
    "notification_preferences",
];

/// Auth state with API client
pub struct SettingsState {
    pub api_client: Arc<RwLock<HiNotesClient>>,
}

/// Response for settings sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettingsResponse {
    pub synced_count: usize,
    pub conflicts_resolved: usize,
    pub errors: Vec<String>,
}

/// Get a single user setting from local database
#[tauri::command]
pub async fn get_user_setting(
    key: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.get_user_setting(&key)
        .map_err(|e| format!("Failed to get setting: {}", e))
}

/// Set a user setting in local database and optionally sync to cloud
#[tauri::command]
pub async fn set_user_setting(
    key: String,
    value: String,
    sync_to_cloud: Option<bool>,
    state: State<'_, AppState>,
    auth_state: State<'_, crate::commands::auth_commands::AuthState>,
) -> Result<(), String> {
    // Store locally
    {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Database lock error: {}", e))?;

        db.set_user_setting(&key, &value)
            .map_err(|e| format!("Failed to set setting: {}", e))?;
    }

    // Sync to cloud if requested and setting is in sync list
    if sync_to_cloud.unwrap_or(true) && SYNCED_SETTINGS.contains(&key.as_str()) {
        let api_client = auth_state.api_client.read().await;

        // Check if authenticated
        if api_client.get_token().await.is_some() {
            // Push to cloud (don't fail local operation if cloud sync fails)
            if let Err(e) = push_setting_to_cloud(&api_client, &key, &value).await {
                log::warn!("Failed to sync setting '{}' to cloud: {}", key, e);
            }
        }
    }

    Ok(())
}

/// List all user settings from local database
#[tauri::command]
pub async fn list_user_settings(
    state: State<'_, AppState>,
) -> Result<HashMap<String, String>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let settings_vec = db
        .list_user_settings()
        .map_err(|e| format!("Failed to list settings: {}", e))?;

    let settings: HashMap<String, String> = settings_vec.into_iter().collect();
    Ok(settings)
}

/// Get list of available AI engines from cloud API
#[tauri::command]
pub async fn get_ai_engines(
    auth_state: State<'_, crate::commands::auth_commands::AuthState>,
) -> Result<Vec<AIEngine>, String> {
    let api_client = auth_state.api_client.read().await;

    // Check if authenticated
    if api_client.get_token().await.is_none() {
        return Err("Not authenticated".to_string());
    }

    api_client
        .list_ai_engines()
        .await
        .map_err(|e| format!("Failed to get AI engines: {}", e))
}

/// Sync all settings bidirectionally with cloud
/// - Pulls all settings from cloud
/// - Compares with local settings
/// - Resolves conflicts (cloud wins by default, or uses timestamps)
/// - Pushes local-only settings to cloud
#[tauri::command]
pub async fn sync_settings_with_cloud(
    state: State<'_, AppState>,
    auth_state: State<'_, crate::commands::auth_commands::AuthState>,
) -> Result<SyncSettingsResponse, String> {
    let api_client = auth_state.api_client.read().await;

    // Check if authenticated
    if api_client.get_token().await.is_none() {
        return Err("Not authenticated".to_string());
    }

    let mut synced_count = 0;
    let mut conflicts_resolved = 0;
    let mut errors = Vec::new();

    // 1. Pull all settings from cloud
    let cloud_settings = match api_client.list_user_settings().await {
        Ok(settings) => settings,
        Err(e) => {
            errors.push(format!("Failed to fetch cloud settings: {}", e));
            return Ok(SyncSettingsResponse {
                synced_count,
                conflicts_resolved,
                errors,
            });
        }
    };

    // 2. Get local settings
    let local_settings = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Database lock error: {}", e))?;

        db.list_user_settings()
            .map_err(|e| format!("Failed to get local settings: {}", e))?
    };

    let local_settings_map: HashMap<String, String> = local_settings.into_iter().collect();

    // 3. Process cloud settings
    for (key, cloud_value) in &cloud_settings {
        // Only sync settings in our sync list
        if !SYNCED_SETTINGS.contains(&key.as_str()) {
            continue;
        }

        match local_settings_map.get(key) {
            Some(local_value) => {
                // Setting exists locally - check for conflicts
                if local_value != cloud_value {
                    // Conflict detected - cloud wins by default
                    log::info!(
                        "Conflict detected for '{}': local='{}', cloud='{}' - using cloud value",
                        key,
                        local_value,
                        cloud_value
                    );

                    let db = state
                        .db
                        .lock()
                        .map_err(|e| format!("Database lock error: {}", e))?;

                    if let Err(e) = db.set_user_setting(key, cloud_value) {
                        errors.push(format!("Failed to update local setting '{}': {}", key, e));
                    } else {
                        conflicts_resolved += 1;
                        synced_count += 1;
                    }
                }
            }
            None => {
                // Setting doesn't exist locally - add it
                let db = state
                    .db
                    .lock()
                    .map_err(|e| format!("Database lock error: {}", e))?;

                if let Err(e) = db.set_user_setting(key, cloud_value) {
                    errors.push(format!("Failed to add local setting '{}': {}", key, e));
                } else {
                    synced_count += 1;
                }
            }
        }
    }

    // 4. Push local-only settings to cloud
    for (key, local_value) in &local_settings_map {
        // Only sync settings in our sync list
        if !SYNCED_SETTINGS.contains(&key.as_str()) {
            continue;
        }

        if !cloud_settings.contains_key(key) {
            // Setting exists locally but not in cloud - push it
            if let Err(e) = push_setting_to_cloud(&api_client, key, local_value).await {
                errors.push(format!("Failed to push setting '{}' to cloud: {}", key, e));
            } else {
                synced_count += 1;
            }
        }
    }

    Ok(SyncSettingsResponse {
        synced_count,
        conflicts_resolved,
        errors,
    })
}

/// Auto sync settings - called on app startup (after login) and after setting changes (debounced)
#[tauri::command]
pub async fn auto_sync_settings(
    state: State<'_, AppState>,
    auth_state: State<'_, crate::commands::auth_commands::AuthState>,
) -> Result<SyncSettingsResponse, String> {
    log::info!("Auto-syncing settings with cloud");
    sync_settings_with_cloud(state, auth_state).await
}

// ===== HELPER FUNCTIONS =====

/// Push a single setting to cloud
async fn push_setting_to_cloud(api_client: &HiNotesClient, key: &str, value: &str) -> Result<()> {
    api_client
        .save_user_setting(key.to_string(), value.to_string())
        .await?;
    Ok(())
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synced_settings_list() {
        // Verify all expected settings are in the sync list
        assert!(SYNCED_SETTINGS.contains(&"theme"));
        assert!(SYNCED_SETTINGS.contains(&"language"));
        assert!(SYNCED_SETTINGS.contains(&"transcription_engine"));
        assert!(SYNCED_SETTINGS.contains(&"auto_translation_enabled"));
        assert!(SYNCED_SETTINGS.contains(&"recording_quality"));
        assert!(SYNCED_SETTINGS.contains(&"calendar_sync_enabled"));
        assert!(SYNCED_SETTINGS.contains(&"notification_preferences"));
    }

    #[test]
    fn test_synced_settings_count() {
        // Ensure we're tracking the expected number of settings
        assert_eq!(SYNCED_SETTINGS.len(), 7);
    }
}
