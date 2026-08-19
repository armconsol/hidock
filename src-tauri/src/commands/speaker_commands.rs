// Tauri commands for speaker identification and management

use crate::commands::AppState;
use crate::db::types::{Speaker, SpeakerSegment};
use crate::speaker::{
    profiles::SpeakerProfileManager, DiarizationEngine, DiarizationOptions, MergeSpeakersRequest,
    SplitSegmentRequest, SpeakerColorAssigner, SpeakerOperationResponse,
};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

/// State for speaker color assignments (per-session)
pub struct SpeakerState {
    pub color_assigner: Mutex<SpeakerColorAssigner>,
    pub profile_manager: Mutex<SpeakerProfileManager>,
}

impl Default for SpeakerState {
    fn default() -> Self {
        Self {
            color_assigner: Mutex::new(SpeakerColorAssigner::new()),
            profile_manager: Mutex::new(SpeakerProfileManager::new()),
        }
    }
}

/// Get speaker segments for a note
#[tauri::command]
pub async fn get_speakers(note_id: String, state: State<'_, AppState>) -> Result<Vec<SpeakerSegment>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_speaker_segments_for_note(&note_id)
        .map_err(|e| format!("Failed to get speakers: {}", e))
}

/// Get all speaker profiles
#[tauri::command]
pub async fn list_all_speakers(state: State<'_, AppState>) -> Result<Vec<Speaker>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_speakers()
        .map_err(|e| format!("Failed to list speakers: {}", e))
}

/// Get a specific speaker profile
#[tauri::command]
pub async fn get_speaker(speaker_id: String, state: State<'_, AppState>) -> Result<Option<Speaker>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.get_speaker(&speaker_id)
        .map_err(|e| format!("Failed to get speaker: {}", e))
}

/// Update speaker label/name
#[tauri::command]
pub async fn update_speaker_label(
    speaker_id: String,
    new_name: String,
    state: State<'_, AppState>,
    speaker_state: State<'_, SpeakerState>,
) -> Result<Speaker, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let manager = speaker_state
        .profile_manager
        .lock()
        .map_err(|e| format!("Profile manager lock error: {}", e))?;

    manager
        .update_speaker_label(&db, &speaker_id, &new_name)
        .map_err(|e| format!("Failed to update speaker label: {}", e))
}

/// Merge multiple speakers into one
#[tauri::command]
pub async fn merge_speakers(
    request: MergeSpeakersRequest,
    state: State<'_, AppState>,
    speaker_state: State<'_, SpeakerState>,
) -> Result<SpeakerOperationResponse, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let mut manager = speaker_state
        .profile_manager
        .lock()
        .map_err(|e| format!("Profile manager lock error: {}", e))?;

    let mut color_assigner = speaker_state
        .color_assigner
        .lock()
        .map_err(|e| format!("Color assigner lock error: {}", e))?;

    // Perform merge in database
    let updated_segments = manager
        .merge_speakers(
            &db,
            &request.note_id,
            &request.target_speaker_id,
            &request.source_speaker_ids,
        )
        .map_err(|e| format!("Failed to merge speakers: {}", e))?;

    // Update color assignments
    color_assigner.merge_speakers(&request.target_speaker_id, &request.source_speaker_ids);

    Ok(SpeakerOperationResponse {
        success: true,
        message: format!(
            "Successfully merged {} speakers into {}",
            request.source_speaker_ids.len(),
            request.target_speaker_id
        ),
        updated_segments,
    })
}

/// Split a speaker segment into two parts
#[tauri::command]
pub async fn split_segment(
    request: SplitSegmentRequest,
    state: State<'_, AppState>,
    speaker_state: State<'_, SpeakerState>,
) -> Result<SpeakerOperationResponse, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let manager = speaker_state
        .profile_manager
        .lock()
        .map_err(|e| format!("Profile manager lock error: {}", e))?;

    let (first, second) = manager
        .split_segment(&db, &request.segment_id, request.split_time, request.new_speaker_id)
        .map_err(|e| format!("Failed to split segment: {}", e))?;

    Ok(SpeakerOperationResponse {
        success: true,
        message: "Successfully split segment".to_string(),
        updated_segments: vec![first, second],
    })
}

/// Analyze audio file for speaker diarization
#[tauri::command]
pub async fn analyze_audio_for_speakers(
    audio_path: String,
    note_id: String,
    options: Option<DiarizationOptions>,
    state: State<'_, AppState>,
) -> Result<crate::speaker::DiarizationResult, String> {
    let path = PathBuf::from(audio_path);

    if !path.exists() {
        return Err(format!("Audio file not found: {:?}", path));
    }

    let engine = if let Some(opts) = options {
        DiarizationEngine::with_options(opts)
    } else {
        DiarizationEngine::new()
    };

    // Perform diarization
    let result = engine
        .analyze_audio(&path, &note_id)
        .await
        .map_err(|e| format!("Diarization failed: {}", e))?;

    // Store results in database
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let (speakers, segments) = engine.to_database_types(&result);

    // Insert speakers
    for speaker in speakers {
        db.insert_speaker(&speaker)
            .map_err(|e| format!("Failed to insert speaker: {}", e))?;
    }

    // Insert segments
    for segment in segments {
        db.insert_speaker_segment(&segment)
            .map_err(|e| format!("Failed to insert segment: {}", e))?;
    }

    Ok(result)
}

/// Get speaker color assignments for a note
#[tauri::command]
pub async fn get_speaker_colors(
    note_id: String,
    state: State<'_, AppState>,
    speaker_state: State<'_, SpeakerState>,
) -> Result<Vec<crate::speaker::colors::SpeakerColor>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let mut color_assigner = speaker_state
        .color_assigner
        .lock()
        .map_err(|e| format!("Color assigner lock error: {}", e))?;

    // Get all speakers for this note
    let segments = db
        .list_speaker_segments_for_note(&note_id)
        .map_err(|e| format!("Failed to get speaker segments: {}", e))?;

    // Assign colors for each unique speaker
    let mut speaker_ids: Vec<String> = segments
        .iter()
        .map(|s| s.speaker_id.clone())
        .collect();
    speaker_ids.sort();
    speaker_ids.dedup();

    for speaker_id in speaker_ids {
        color_assigner.assign_color(&speaker_id);
    }

    Ok(color_assigner.get_all_assignments())
}

/// Get speaker statistics for a note
#[tauri::command]
pub async fn get_speaker_statistics(
    note_id: String,
    state: State<'_, AppState>,
    speaker_state: State<'_, SpeakerState>,
) -> Result<Vec<crate::speaker::profiles::SpeakerStats>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let manager = speaker_state
        .profile_manager
        .lock()
        .map_err(|e| format!("Profile manager lock error: {}", e))?;

    let stats_map = manager
        .calculate_speaker_statistics(&db, &note_id)
        .map_err(|e| format!("Failed to calculate statistics: {}", e))?;

    Ok(stats_map.into_values().collect())
}

/// Delete a speaker segment
#[tauri::command]
pub async fn delete_speaker_segment(
    segment_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_speaker_segment(&segment_id)
        .map_err(|e| format!("Failed to delete segment: {}", e))
}

/// Delete a speaker profile (and all associated segments)
#[tauri::command]
pub async fn delete_speaker(
    speaker_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_speaker(&speaker_id)
        .map_err(|e| format!("Failed to delete speaker: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn setup_test_state() -> AppState {
        let db = Database::new_in_memory().expect("Failed to create test database");
        AppState {
            db: Mutex::new(db),
        }
    }

    fn setup_speaker_state() -> SpeakerState {
        SpeakerState::default()
    }

    // TODO: Fix Tauri State::new usage in tests (not available in Tauri 2.x)
    // #[tokio::test]
    // async fn test_list_all_speakers() {
    //     let state = setup_test_state();

    //     let result = list_all_speakers(State::new(state)).await;
    //     assert!(result.is_ok());
    //     let speakers = result.unwrap();
    //     assert_eq!(speakers.len(), 0);
    // }

    // #[tokio::test]
    // async fn test_get_speaker_colors() {
    //     let state = setup_test_state();
    //     let speaker_state = setup_speaker_state();

    //     // Create some test data
    //     {
    //         let db = state.db.lock().unwrap();
    //         let speaker = crate::db::types::InsertSpeaker {
    //             id: "speaker-1".to_string(),
    //             name: Some("Test Speaker".to_string()),
    //             voice_signature: None,
    //         };
    //         db.insert_speaker(&speaker).unwrap();

    //         let segment = crate::db::types::InsertSpeakerSegment {
    //             id: "seg-1".to_string(),
    //             note_id: "note-1".to_string(),
    //             speaker_id: "speaker-1".to_string(),
    //             start_time: 0.0,
    //             end_time: 5.0,
    //             confidence: 0.9,
    //         };
    //         db.insert_speaker_segment(&segment).unwrap();
    //     }

    //     let result = get_speaker_colors(
    //         "note-1".to_string(),
    //         State::new(state),
    //         State::new(speaker_state),
    //     )
    //     .await;

    //     assert!(result.is_ok());
    //     let colors = result.unwrap();
    //     assert_eq!(colors.len(), 1);
    //     assert_eq!(colors[0].speaker_id, "speaker-1");
    // }
}
