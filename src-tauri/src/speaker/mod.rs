// Speaker identification module for multi-speaker audio analysis
//
// This module provides speaker identification functionality including:
// - Audio diarization (speaker segmentation)
// - Speaker profile management
// - Visual color assignment for UI
// - Merge/split operations for speaker segments

pub mod colors;
pub mod diarization;
pub mod profiles;

pub use colors::{SpeakerColorAssigner, SPEAKER_COLORS};
pub use diarization::{DiarizationEngine, DiarizationOptions, DiarizationResult};
pub use profiles::{SpeakerMergeHistory, SpeakerProfileManager};

use serde::{Deserialize, Serialize};

/// Request to merge two speakers into one
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeSpeakersRequest {
    /// The speaker ID to keep
    pub target_speaker_id: String,
    /// The speaker ID(s) to merge into the target
    pub source_speaker_ids: Vec<String>,
    /// The note ID where the merge should occur
    pub note_id: String,
}

/// Request to split a speaker segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitSegmentRequest {
    /// The segment ID to split
    pub segment_id: String,
    /// The time (in seconds) where the split should occur
    pub split_time: f64,
    /// Optional: New speaker ID for the second half
    pub new_speaker_id: Option<String>,
}

/// Response for speaker operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerOperationResponse {
    pub success: bool,
    pub message: String,
    /// Updated segments after the operation
    pub updated_segments: Vec<crate::db::types::SpeakerSegment>,
}
