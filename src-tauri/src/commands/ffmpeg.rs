use crate::audio::{AudioInfo, FFmpegWrapper};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

/// FFmpeg state wrapper for Tauri
pub struct FFmpegState {
    wrapper: Mutex<Option<FFmpegWrapper>>,
}

impl FFmpegState {
    pub fn new() -> Self {
        Self {
            wrapper: Mutex::new(None),
        }
    }

    /// Initialize FFmpeg wrapper (lazy initialization)
    fn get_or_init(&self) -> Result<(), String> {
        let mut wrapper = self.wrapper.lock().map_err(|e| e.to_string())?;

        if wrapper.is_none() {
            let ffmpeg = FFmpegWrapper::new().map_err(|e| e.to_string())?;
            *wrapper = Some(ffmpeg);
        }

        Ok(())
    }
}

/// Check if FFmpeg is available and return version info
#[tauri::command]
pub async fn ffmpeg_validate(state: State<'_, FFmpegState>) -> Result<String, String> {
    state.get_or_init()?;

    let wrapper = state.wrapper.lock().map_err(|e| e.to_string())?;
    let wrapper = wrapper.as_ref().ok_or("FFmpeg not initialized")?;

    wrapper.validate().map_err(|e| e.to_string())
}

/// Get FFmpeg binary path
#[tauri::command]
pub async fn ffmpeg_binary_path(state: State<'_, FFmpegState>) -> Result<String, String> {
    state.get_or_init()?;

    let wrapper = state.wrapper.lock().map_err(|e| e.to_string())?;
    let wrapper = wrapper.as_ref().ok_or("FFmpeg not initialized")?;

    Ok(wrapper.binary_path().to_string_lossy().to_string())
}

/// Convert audio file to a different format
#[tauri::command]
pub async fn ffmpeg_convert_audio(
    state: State<'_, FFmpegState>,
    input_path: String,
    output_path: String,
    output_format: String,
) -> Result<(), String> {
    state.get_or_init()?;

    let wrapper = state.wrapper.lock().map_err(|e| e.to_string())?;
    let wrapper = wrapper.as_ref().ok_or("FFmpeg not initialized")?;

    wrapper
        .convert_audio(
            PathBuf::from(input_path),
            PathBuf::from(output_path),
            &output_format,
        )
        .map_err(|e| e.to_string())
}

/// Merge multiple audio files into a single file
#[tauri::command]
pub async fn ffmpeg_merge_audio(
    state: State<'_, FFmpegState>,
    input_paths: Vec<String>,
    output_path: String,
    output_format: String,
) -> Result<(), String> {
    state.get_or_init()?;

    let wrapper = state.wrapper.lock().map_err(|e| e.to_string())?;
    let wrapper = wrapper.as_ref().ok_or("FFmpeg not initialized")?;

    let paths: Vec<PathBuf> = input_paths.into_iter().map(PathBuf::from).collect();

    wrapper
        .merge_audio_files(&paths, PathBuf::from(output_path), &output_format)
        .map_err(|e| e.to_string())
}

/// Extract audio segment from a file
#[tauri::command]
pub async fn ffmpeg_extract_segment(
    state: State<'_, FFmpegState>,
    input_path: String,
    output_path: String,
    start_time: f64,
    duration: f64,
    output_format: String,
) -> Result<(), String> {
    state.get_or_init()?;

    let wrapper = state.wrapper.lock().map_err(|e| e.to_string())?;
    let wrapper = wrapper.as_ref().ok_or("FFmpeg not initialized")?;

    wrapper
        .extract_segment(
            PathBuf::from(input_path),
            PathBuf::from(output_path),
            start_time,
            duration,
            &output_format,
        )
        .map_err(|e| e.to_string())
}

/// Get audio file metadata
#[tauri::command]
pub async fn ffmpeg_get_audio_info(
    state: State<'_, FFmpegState>,
    input_path: String,
) -> Result<AudioInfoResponse, String> {
    state.get_or_init()?;

    let wrapper = state.wrapper.lock().map_err(|e| e.to_string())?;
    let wrapper = wrapper.as_ref().ok_or("FFmpeg not initialized")?;

    let info = wrapper
        .get_audio_info(PathBuf::from(input_path))
        .map_err(|e| e.to_string())?;

    Ok(AudioInfoResponse {
        duration: info.duration,
    })
}

/// Audio info response for frontend
#[derive(serde::Serialize)]
pub struct AudioInfoResponse {
    pub duration: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffmpeg_state_initialization() {
        let state = FFmpegState::new();

        // State should be uninitialized
        let wrapper = state.wrapper.lock().unwrap();
        assert!(wrapper.is_none());
    }

    #[tokio::test]
    async fn test_ffmpeg_validate_command() {
        let state = FFmpegState::new();

        // This will fail if FFmpeg is not installed
        let result = ffmpeg_validate(State::from(&state)).await;

        // We can't assert success because FFmpeg might not be installed
        // But we can verify the function runs without panicking
        match result {
            Ok(version) => {
                assert!(version.starts_with("ffmpeg version"));
            }
            Err(e) => {
                // Expected if FFmpeg is not installed
                assert!(e.contains("not found") || e.contains("FFmpeg"));
            }
        }
    }
}
