// Device management commands

use crate::api::types::DeviceFile;
use crate::audio::cache::AudioCache;
use crate::commands::{auth_commands::AuthState, AppState};
use crate::db::types::{Device, DeviceStatus, InsertNote};
use chrono::Utc;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindDeviceRequest {
    pub id: String,
    pub name: String,
}

/// List all devices
#[tauri::command]
pub async fn list_devices(state: State<'_, AppState>) -> Result<Vec<Device>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_devices()
        .map_err(|e| format!("Failed to list devices: {}", e))
}

/// Get a device by ID
#[tauri::command]
pub async fn get_device(
    device_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Device>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.get_device(&device_id)
        .map_err(|e| format!("Failed to get device: {}", e))
}

/// Bind a new device
#[tauri::command]
pub async fn bind_device(
    request: BindDeviceRequest,
    state: State<'_, AppState>,
) -> Result<Device, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    // Check if device already exists
    if let Some(_existing) = db
        .get_device(&request.id)
        .map_err(|e| format!("Failed to check device existence: {}", e))?
    {
        return Err(format!("Device with ID {} is already bound", request.id));
    }

    let now = Utc::now();
    let device = Device {
        id: request.id,
        name: request.name,
        status: DeviceStatus::Disconnected,
        last_sync: None,
        created_at: now,
    };

    db.insert_device(&device)
        .map_err(|e| format!("Failed to bind device: {}", e))
}

/// Unbind a device
#[tauri::command]
pub async fn unbind_device(device_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_device(&device_id)
        .map_err(|e| format!("Failed to unbind device: {}", e))
}

/// Update device status
#[tauri::command]
pub async fn update_device_status(
    device_id: String,
    status: String,
    state: State<'_, AppState>,
) -> Result<Device, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let device_status =
        DeviceStatus::from_str(&status).map_err(|e| format!("Invalid device status: {}", e))?;

    db.update_device_status(&device_id, device_status)
        .map_err(|e| format!("Failed to update device status: {}", e))
}

/// Update device last sync time
#[tauri::command]
pub async fn update_device_last_sync(
    device_id: String,
    state: State<'_, AppState>,
) -> Result<Device, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.update_device_last_sync(&device_id)
        .map_err(|e| format!("Failed to update device last sync: {}", e))
}

// ============================================================================
// Device File Management Commands
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDownloadProgress {
    pub file_id: String,
    pub downloaded: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSyncProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
}

/// List all files stored on a connected HiDoc device
///
/// # Arguments
/// * `device_id` - The unique identifier of the device
/// * `auth_state` - Authentication state containing API client
///
/// # Returns
/// * `Result<Vec<DeviceFile>>` - List of files with metadata
///
/// # Errors
/// * Returns error if not authenticated
/// * Returns error if device is not found or not connected
#[tauri::command]
pub async fn list_device_files(
    device_id: String,
    auth_state: State<'_, AuthState>,
) -> Result<Vec<DeviceFile>, String> {
    info!("Listing files for device: {}", device_id);

    let api_client = auth_state.api_client.read().await;

    api_client.list_device_files(device_id).await.map_err(|e| {
        error!("Failed to list device files: {}", e);
        format!("Failed to list device files: {}", e)
    })
}

/// Download a file from a connected HiDoc device
///
/// This command:
/// 1. Downloads the audio file from the device via API
/// 2. Saves it to the local audio cache
/// 3. Creates a note entry in the database
/// 4. Returns the local file path
///
/// # Arguments
/// * `device_id` - The unique identifier of the device
/// * `file_id` - The unique identifier of the file to download
/// * `file_name` - Name of the file (for note title)
/// * `create_note` - Whether to create a note entry for the downloaded file
/// * `app_handle` - Tauri app handle for emitting progress events
/// * `auth_state` - Authentication state containing API client
/// * `app_state` - Application state containing database
///
/// # Returns
/// * `Result<String>` - Local file path of the downloaded audio
///
/// # Errors
/// * Returns error if not authenticated
/// * Returns error if device or file is not found
/// * Returns error if download fails
#[tauri::command]
pub async fn download_device_file(
    device_id: String,
    file_id: String,
    file_name: String,
    create_note: bool,
    app_handle: AppHandle,
    auth_state: State<'_, AuthState>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Downloading file {} from device {}", file_id, device_id);

    let api_client = auth_state.api_client.read().await;

    // Download file with progress callback
    let app_handle_clone = app_handle.clone();
    let file_id_clone = file_id.clone();

    let audio_data = api_client
        .download_device_file(
            device_id.clone(),
            file_id.clone(),
            Some(move |downloaded, total| {
                let _ = app_handle_clone.emit(
                    "device-file-download-progress",
                    FileDownloadProgress {
                        file_id: file_id_clone.clone(),
                        downloaded,
                        total,
                    },
                );
            }),
        )
        .await
        .map_err(|e| {
            error!("Failed to download device file: {}", e);
            format!("Failed to download device file: {}", e)
        })?;

    info!("Downloaded {} bytes", audio_data.len());

    // Get cache directory
    let cache_dir = AudioCache::get_platform_cache_dir().map_err(|e| {
        error!("Failed to get cache directory: {}", e);
        format!("Failed to get cache directory: {}", e)
    })?;

    // Generate unique filename based on file_id and timestamp
    let extension = if file_name.ends_with(".mp3") {
        "mp3"
    } else if file_name.ends_with(".wav") {
        "wav"
    } else if file_name.ends_with(".m4a") {
        "m4a"
    } else {
        "audio"
    };

    let local_filename = format!("{}_{}.{}", file_id, Utc::now().timestamp(), extension);
    let local_path = cache_dir.join(&local_filename);

    // Ensure cache directory exists
    tokio::fs::create_dir_all(&cache_dir).await.map_err(|e| {
        error!("Failed to create cache directory: {}", e);
        format!("Failed to create cache directory: {}", e)
    })?;

    // Write audio data to file
    tokio::fs::write(&local_path, &audio_data)
        .await
        .map_err(|e| {
            error!("Failed to write audio file: {}", e);
            format!("Failed to write audio file: {}", e)
        })?;

    info!("Saved audio to: {:?}", local_path);

    // Create note entry if requested
    if create_note {
        let db = app_state.db.lock().map_err(|e| {
            error!("Database lock error: {}", e);
            format!("Database lock error: {}", e)
        })?;

        let note = InsertNote {
            id: uuid::Uuid::new_v4().to_string(),
            title: file_name.clone(),
            content: Some(format!(
                "Imported from device {} at {}",
                device_id,
                Utc::now()
            )),
            folder_id: None,
            audio_url: Some(local_path.to_string_lossy().to_string()),
            duration: None,
            rating: None,
        };

        db.insert_note(&note).map_err(|e| {
            error!("Failed to create note: {}", e);
            format!("Failed to create note: {}", e)
        })?;

        info!("Created note with ID: {}", note.id);
    }

    // Update device last sync time
    let db = app_state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.update_device_last_sync(&device_id)
        .map_err(|e| {
            error!("Failed to update device last sync: {}", e);
            // Don't fail the whole operation for this
        })
        .ok();

    Ok(local_path.to_string_lossy().to_string())
}

/// Upload a local audio file to a connected HiDoc device
///
/// # Arguments
/// * `device_id` - The unique identifier of the device
/// * `file_path` - Path to the local audio file to upload
/// * `auth_state` - Authentication state containing API client
///
/// # Returns
/// * `Result<()>` - Success or error
///
/// # Errors
/// * Returns error if not authenticated
/// * Returns error if file does not exist or cannot be read
/// * Returns error if device is not connected
/// * Returns error if upload fails
#[tauri::command]
pub async fn upload_device_file(
    device_id: String,
    file_path: String,
    auth_state: State<'_, AuthState>,
) -> Result<(), String> {
    info!("Uploading file {} to device {}", file_path, device_id);

    let path = PathBuf::from(file_path);

    if !path.exists() {
        return Err(format!("File does not exist: {:?}", path));
    }

    let api_client = auth_state.api_client.read().await;

    api_client
        .upload_device_file(device_id, path, None::<fn(u64, u64)>)
        .await
        .map_err(|e| {
            error!("Failed to upload device file: {}", e);
            format!("Failed to upload device file: {}", e)
        })
}

/// Synchronize all unsynced files from a connected HiDoc device
///
/// This command:
/// 1. Lists all files on the device
/// 2. Filters for files that haven't been synced yet
/// 3. Downloads each file sequentially
/// 4. Creates note entries for each downloaded file
/// 5. Emits progress events during the sync
///
/// # Arguments
/// * `device_id` - The unique identifier of the device
/// * `app_handle` - Tauri app handle for emitting progress events
/// * `auth_state` - Authentication state containing API client
/// * `app_state` - Application state containing database
///
/// # Returns
/// * `Result<Vec<String>>` - List of local file paths for downloaded files
///
/// # Errors
/// * Returns error if not authenticated
/// * Returns error if device is not found or not connected
/// * Returns error if any download fails (partial success is not returned)
#[tauri::command]
pub async fn sync_all_device_files(
    device_id: String,
    app_handle: AppHandle,
    auth_state: State<'_, AuthState>,
    app_state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    info!("Syncing all files from device: {}", device_id);

    let api_client = auth_state.api_client.read().await;

    // Get list of all files
    let all_files = api_client
        .list_device_files(device_id.clone())
        .await
        .map_err(|e| {
            error!("Failed to list device files: {}", e);
            format!("Failed to list device files: {}", e)
        })?;

    // Filter for unsynced files
    let unsynced_files: Vec<_> = all_files
        .into_iter()
        .filter(|f| !f.already_synced)
        .collect();

    info!("Found {} unsynced files to download", unsynced_files.len());

    if unsynced_files.is_empty() {
        return Ok(Vec::new());
    }

    let total_files = unsynced_files.len();
    let mut downloaded_paths = Vec::new();

    // Download each file
    for (index, file) in unsynced_files.iter().enumerate() {
        info!(
            "Downloading file {}/{}: {} ({})",
            index + 1,
            total_files,
            file.name,
            file.file_id
        );

        // Emit progress event
        let _ = app_handle.emit(
            "device-file-sync-progress",
            FileSyncProgress {
                current: index + 1,
                total: total_files,
                current_file: file.name.clone(),
            },
        );

        // Download the file
        let app_handle_clone = app_handle.clone();
        let file_id_clone = file.file_id.clone();

        let audio_data = api_client
            .download_device_file(
                device_id.clone(),
                file.file_id.clone(),
                Some(move |downloaded, total| {
                    let _ = app_handle_clone.emit(
                        "device-file-download-progress",
                        FileDownloadProgress {
                            file_id: file_id_clone.clone(),
                            downloaded,
                            total,
                        },
                    );
                }),
            )
            .await
            .map_err(|e| {
                error!("Failed to download file {}: {}", file.name, e);
                format!("Failed to download file {}: {}", file.name, e)
            })?;

        // Get cache directory
        let cache_dir = AudioCache::get_platform_cache_dir().map_err(|e| {
            error!("Failed to get cache directory: {}", e);
            format!("Failed to get cache directory: {}", e)
        })?;

        // Generate unique filename
        let extension = if file.name.ends_with(".mp3") {
            "mp3"
        } else if file.name.ends_with(".wav") {
            "wav"
        } else if file.name.ends_with(".m4a") {
            "m4a"
        } else {
            "audio"
        };

        let local_filename = format!("{}_{}.{}", file.file_id, Utc::now().timestamp(), extension);
        let local_path = cache_dir.join(&local_filename);

        // Ensure cache directory exists
        tokio::fs::create_dir_all(&cache_dir).await.map_err(|e| {
            error!("Failed to create cache directory: {}", e);
            format!("Failed to create cache directory: {}", e)
        })?;

        // Write audio data to file
        tokio::fs::write(&local_path, &audio_data)
            .await
            .map_err(|e| {
                error!("Failed to write audio file: {}", e);
                format!("Failed to write audio file: {}", e)
            })?;

        info!("Saved audio to: {:?}", local_path);

        // Create note entry
        let db = app_state.db.lock().map_err(|e| {
            error!("Database lock error: {}", e);
            format!("Database lock error: {}", e)
        })?;

        let note = InsertNote {
            id: uuid::Uuid::new_v4().to_string(),
            title: file.name.clone(),
            content: Some(format!(
                "Imported from device {} at {}",
                device_id,
                Utc::now()
            )),
            folder_id: None,
            audio_url: Some(local_path.to_string_lossy().to_string()),
            duration: file.duration.map(|d| format!("{:.2}", d)),
            rating: None,
        };

        db.insert_note(&note).map_err(|e| {
            error!("Failed to create note: {}", e);
            format!("Failed to create note: {}", e)
        })?;

        info!("Created note with ID: {}", note.id);

        downloaded_paths.push(local_path.to_string_lossy().to_string());
    }

    // Update device last sync time
    let db = app_state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.update_device_last_sync(&device_id)
        .map_err(|e| {
            error!("Failed to update device last sync: {}", e);
            // Don't fail the whole operation for this
        })
        .ok();

    info!(
        "Successfully synced {} files from device {}",
        downloaded_paths.len(),
        device_id
    );

    Ok(downloaded_paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn test_device_operations() {
        let db = Database::new_in_memory().expect("Failed to create test database");

        // Bind a device
        let device = Device {
            id: "HIDOC-001".to_string(),
            name: "My HiDoc P1".to_string(),
            status: DeviceStatus::Disconnected,
            last_sync: None,
            created_at: Utc::now(),
        };

        db.insert_device(&device).unwrap();

        // Get device
        let retrieved = db.get_device("HIDOC-001").unwrap().unwrap();
        assert_eq!(retrieved.id, "HIDOC-001");
        assert_eq!(retrieved.name, "My HiDoc P1");
        assert_eq!(retrieved.status, DeviceStatus::Disconnected);

        // List devices
        let devices = db.list_devices().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "HIDOC-001");
    }

    #[test]
    fn test_bind_duplicate_device() {
        let db = Database::new_in_memory().expect("Failed to create test database");

        let device = Device {
            id: "HIDOC-001".to_string(),
            name: "My HiDoc P1".to_string(),
            status: DeviceStatus::Disconnected,
            last_sync: None,
            created_at: Utc::now(),
        };

        // First bind should succeed
        db.insert_device(&device).unwrap();

        // Second bind should fail
        let result = db.insert_device(&device);
        assert!(result.is_err());
    }

    #[test]
    fn test_unbind_device() {
        let db = Database::new_in_memory().expect("Failed to create test database");

        // Bind a device
        let device = Device {
            id: "HIDOC-001".to_string(),
            name: "My HiDoc P1".to_string(),
            status: DeviceStatus::Disconnected,
            last_sync: None,
            created_at: Utc::now(),
        };
        db.insert_device(&device).unwrap();

        // Unbind the device
        db.delete_device("HIDOC-001").unwrap();

        // Verify device is gone
        let devices = db.list_devices().unwrap();
        assert_eq!(devices.len(), 0);
    }

    #[test]
    fn test_update_device_status() {
        let db = Database::new_in_memory().expect("Failed to create test database");

        // Bind a device
        let device = Device {
            id: "HIDOC-001".to_string(),
            name: "My HiDoc P1".to_string(),
            status: DeviceStatus::Disconnected,
            last_sync: None,
            created_at: Utc::now(),
        };
        db.insert_device(&device).unwrap();

        // Update status to connected
        db.update_device_status("HIDOC-001", DeviceStatus::Connected)
            .unwrap();

        let updated = db.get_device("HIDOC-001").unwrap().unwrap();
        assert_eq!(updated.status, DeviceStatus::Connected);

        // Update status to disconnected
        db.update_device_status("HIDOC-001", DeviceStatus::Disconnected)
            .unwrap();

        let updated = db.get_device("HIDOC-001").unwrap().unwrap();
        assert_eq!(updated.status, DeviceStatus::Disconnected);
    }

    #[test]
    fn test_update_device_last_sync() {
        let db = Database::new_in_memory().expect("Failed to create test database");

        // Bind a device
        let device = Device {
            id: "HIDOC-001".to_string(),
            name: "My HiDoc P1".to_string(),
            status: DeviceStatus::Disconnected,
            last_sync: None,
            created_at: Utc::now(),
        };
        db.insert_device(&device).unwrap();
        assert!(device.last_sync.is_none());

        // Update last sync
        db.update_device_last_sync("HIDOC-001").unwrap();

        let updated = db.get_device("HIDOC-001").unwrap().unwrap();
        assert!(updated.last_sync.is_some());
    }
}
