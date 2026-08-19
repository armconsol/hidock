use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use crate::sync::CalendarSync;

/// State for calendar sync worker
pub struct CalendarSyncState {
    pub worker: Arc<Mutex<Option<CalendarSync>>>,
}

impl CalendarSyncState {
    pub fn new() -> Self {
        Self {
            worker: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for CalendarSyncState {
    fn default() -> Self {
        Self::new()
    }
}

/// Response for calendar sync status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSyncStatus {
    pub is_running: bool,
    pub last_sync: Option<String>,
    pub has_token: bool,
}

/// Start the calendar sync worker
#[tauri::command]
pub async fn start_calendar_sync(
    google_token: Option<String>,
    calendar_id: Option<String>,
    state: State<'_, CalendarSyncState>,
) -> Result<String, String> {
    let worker_guard = state.worker.lock().await;

    if let Some(ref worker) = *worker_guard {
        if worker.is_running().await {
            return Err("Calendar sync worker is already running".to_string());
        }
    }

    // Worker should be initialized at app startup
    if let Some(ref worker) = *worker_guard {
        // Set Google token if provided
        if let Some(token) = google_token {
            worker.set_google_token(token).await;
        }

        // Set calendar ID if provided
        if let Some(id) = calendar_id {
            worker.set_calendar_id(id).await;
        }

        worker
            .start_sync_loop()
            .await
            .map_err(|e| format!("Failed to start calendar sync: {}", e))?;

        Ok("Calendar sync worker started".to_string())
    } else {
        Err("Calendar sync worker not initialized".to_string())
    }
}

/// Stop the calendar sync worker
#[tauri::command]
pub async fn stop_calendar_sync(state: State<'_, CalendarSyncState>) -> Result<String, String> {
    let worker_guard = state.worker.lock().await;

    if let Some(ref worker) = *worker_guard {
        worker.stop().await;
        Ok("Calendar sync worker stopped".to_string())
    } else {
        Err("Calendar sync worker not initialized".to_string())
    }
}

/// Get calendar sync status
#[tauri::command]
pub async fn get_calendar_sync_status(
    state: State<'_, CalendarSyncState>,
) -> Result<CalendarSyncStatus, String> {
    let worker_guard = state.worker.lock().await;

    if let Some(ref worker) = *worker_guard {
        let is_running = worker.is_running().await;
        let last_sync = worker.get_last_sync().await.map(|dt| dt.to_rfc3339());

        // Check if token is set (without exposing the token value)
        let has_token = {
            // This is a simplified check - in production you'd want a proper method
            // For now we'll assume if it's running, it has a token
            is_running
        };

        Ok(CalendarSyncStatus {
            is_running,
            last_sync,
            has_token,
        })
    } else {
        Err("Calendar sync worker not initialized".to_string())
    }
}

/// Set Google OAuth token for calendar sync
#[tauri::command]
pub async fn set_calendar_google_token(
    token: String,
    state: State<'_, CalendarSyncState>,
) -> Result<String, String> {
    let worker_guard = state.worker.lock().await;

    if let Some(ref worker) = *worker_guard {
        worker.set_google_token(token).await;
        Ok("Google token set successfully".to_string())
    } else {
        Err("Calendar sync worker not initialized".to_string())
    }
}

/// Clear Google OAuth token for calendar sync
#[tauri::command]
pub async fn clear_calendar_google_token(
    state: State<'_, CalendarSyncState>,
) -> Result<String, String> {
    let worker_guard = state.worker.lock().await;

    if let Some(ref worker) = *worker_guard {
        worker.clear_google_token().await;
        Ok("Google token cleared successfully".to_string())
    } else {
        Err("Calendar sync worker not initialized".to_string())
    }
}

/// Set the calendar ID to sync
#[tauri::command]
pub async fn set_calendar_id(
    calendar_id: String,
    state: State<'_, CalendarSyncState>,
) -> Result<String, String> {
    let worker_guard = state.worker.lock().await;

    if let Some(ref worker) = *worker_guard {
        worker.set_calendar_id(calendar_id.clone()).await;
        Ok(format!("Calendar ID set to: {}", calendar_id))
    } else {
        Err("Calendar sync worker not initialized".to_string())
    }
}

/// Manually trigger a sync now
#[tauri::command]
pub async fn sync_calendar_now(state: State<'_, CalendarSyncState>) -> Result<usize, String> {
    let worker_guard = state.worker.lock().await;

    if let Some(ref worker) = *worker_guard {
        worker
            .sync_now()
            .await
            .map_err(|e| format!("Failed to sync calendar: {}", e))
    } else {
        Err("Calendar sync worker not initialized".to_string())
    }
}

/// Get last sync timestamp
#[tauri::command]
pub async fn get_calendar_last_sync(
    state: State<'_, CalendarSyncState>,
) -> Result<Option<String>, String> {
    let worker_guard = state.worker.lock().await;

    if let Some(ref worker) = *worker_guard {
        Ok(worker.get_last_sync().await.map(|dt| dt.to_rfc3339()))
    } else {
        Err("Calendar sync worker not initialized".to_string())
    }
}

/// Notify HiNotes calendar of recording status for a specific event
///
/// This command notifies the HiNotes API that a recording has started or stopped
/// for a specific calendar event. The server typically updates the Google Calendar
/// event description with "Recording in progress..." while active, and may add a
/// transcription link when the recording is finished.
///
/// # Arguments
/// * `event_id` - Google Calendar event ID
/// * `is_recording` - True if recording started, False if recording stopped
///
/// # Returns
/// Success message or error
#[tauri::command]
pub async fn notify_calendar_recording(
    event_id: String,
    is_recording: bool,
    state: State<'_, CalendarSyncState>,
) -> Result<String, String> {
    let worker_guard = state.worker.lock().await;

    if let Some(ref worker) = *worker_guard {
        worker
            .notify_recording_status(&event_id, is_recording)
            .await
            .map_err(|e| format!("Failed to notify recording status: {}", e))?;

        let status_msg = if is_recording {
            "started"
        } else {
            "stopped"
        };

        Ok(format!(
            "Successfully notified that recording {} for event {}",
            status_msg, event_id
        ))
    } else {
        Err("Calendar sync worker not initialized".to_string())
    }
}
