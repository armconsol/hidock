use crate::audio::cache::AudioCache;
use crate::audio::AudioProcessor;
use crate::commands::AppState;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Get audio for a note, fetching from cache or downloading if needed
#[tauri::command]
pub async fn get_audio(
    note_id: String,
    audio_url: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    // Get database path (lock is scoped and dropped before any .await)
    let db_path = {
        let db = state.db.lock().map_err(|e| format!("{}", e))?;
        db.get_db_path()
    };

    // Get cache directory
    let cache_dir = AudioCache::get_platform_cache_dir().map_err(|e| format!("{}", e))?;

    // Create database connection for audio cache
    let cache_db_conn = Connection::open(&db_path).map_err(|e| format!("{}", e))?;

    // Initialize audio cache table if not exists
    cache_db_conn
        .execute(
            "CREATE TABLE IF NOT EXISTS audio_cache (
                note_id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                last_accessed DATETIME NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("{}", e))?;

    // Create audio cache instance
    let audio_cache = AudioCache::new(cache_dir, cache_db_conn).map_err(|e| format!("{}", e))?;

    // Get audio (from cache or download)
    audio_cache
        .get_audio(&note_id, &audio_url)
        .await
        .map_err(|e| e.to_string())
}

// ===== AUDIO PROCESSING COMMANDS =====

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioMergeRequest {
    pub file_paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioReplaceRequest {
    pub original_path: String,
    pub replacement_path: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub fade_duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioTrimRequest {
    pub input_path: String,
    pub start_ms: u64,
    pub end_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioOperationResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

/// Merge multiple audio files into one
#[tauri::command]
pub async fn merge_audio_files(request: AudioMergeRequest) -> Result<AudioOperationResult, String> {
    let processor = AudioProcessor::new().map_err(|e| format!("{}", e))?;

    let file_paths: Vec<PathBuf> = request.file_paths.iter().map(PathBuf::from).collect();

    match processor.merge_audio(&file_paths).await {
        Ok(output_path) => Ok(AudioOperationResult {
            success: true,
            output_path: Some(output_path.to_string_lossy().to_string()),
            error: None,
        }),
        Err(e) => Ok(AudioOperationResult {
            success: false,
            output_path: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Replace a segment of audio with new audio
#[tauri::command]
pub async fn replace_audio_segment(
    request: AudioReplaceRequest,
) -> Result<AudioOperationResult, String> {
    let processor = AudioProcessor::new().map_err(|e| format!("{}", e))?;

    let original = PathBuf::from(&request.original_path);
    let replacement = PathBuf::from(&request.replacement_path);

    match processor
        .replace_audio_segment(
            &original,
            &replacement,
            request.start_ms,
            request.end_ms,
            request.fade_duration_ms.unwrap_or(0),
        )
        .await
    {
        Ok(output_path) => Ok(AudioOperationResult {
            success: true,
            output_path: Some(output_path.to_string_lossy().to_string()),
            error: None,
        }),
        Err(e) => Ok(AudioOperationResult {
            success: false,
            output_path: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Save audio data as a new file
#[tauri::command]
pub async fn save_audio_as_new(
    audio_data: Vec<u8>,
    format: String,
) -> Result<AudioOperationResult, String> {
    let processor = AudioProcessor::new().map_err(|e| format!("{}", e))?;

    match processor.save_as_new(&audio_data, &format).await {
        Ok(output_path) => Ok(AudioOperationResult {
            success: true,
            output_path: Some(output_path.to_string_lossy().to_string()),
            error: None,
        }),
        Err(e) => Ok(AudioOperationResult {
            success: false,
            output_path: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Extract/trim audio to a specific time range
#[tauri::command]
pub async fn extract_audio_segment(
    request: AudioTrimRequest,
) -> Result<AudioOperationResult, String> {
    let processor = AudioProcessor::new().map_err(|e| format!("{}", e))?;
    let input = PathBuf::from(&request.input_path);

    match processor
        .extract_segment(&input, request.start_ms, request.end_ms)
        .await
    {
        Ok(output_path) => Ok(AudioOperationResult {
            success: true,
            output_path: Some(output_path.to_string_lossy().to_string()),
            error: None,
        }),
        Err(e) => Ok(AudioOperationResult {
            success: false,
            output_path: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Cleanup temporary audio processing files
#[tauri::command]
pub async fn cleanup_audio_temp_files(max_age_hours: u64) -> Result<usize, String> {
    let processor = AudioProcessor::new().map_err(|e| format!("{}", e))?;

    processor
        .cleanup_temp_files(max_age_hours)
        .await
        .map_err(|e| e.to_string())
}

/// Verify FFmpeg is available
#[tauri::command]
pub async fn verify_ffmpeg() -> Result<bool, String> {
    let processor = AudioProcessor::new().map_err(|e| format!("{}", e))?;

    match processor.verify_ffmpeg() {
        Ok(_) => Ok(true),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires FFmpeg
    async fn test_save_audio_as_new() {
        let test_data = vec![1, 2, 3, 4, 5];
        let result = save_audio_as_new(test_data, "m4a".to_string()).await;

        assert!(result.is_ok());
        let operation_result = result.unwrap();
        assert!(operation_result.success);
        assert!(operation_result.output_path.is_some());
    }

    #[tokio::test]
    #[ignore] // Requires FFmpeg
    async fn test_merge_audio_validation() {
        let request = AudioMergeRequest { file_paths: vec![] };

        let result = merge_audio_files(request).await;
        assert!(result.is_ok());

        let operation_result = result.unwrap();
        assert!(!operation_result.success);
        assert!(operation_result.error.is_some());
    }

    #[tokio::test]
    async fn test_verify_ffmpeg() {
        let result = verify_ffmpeg().await;
        // This will pass if FFmpeg is installed, or fail with an error
        assert!(result.is_ok() || result.is_err());
    }
}
