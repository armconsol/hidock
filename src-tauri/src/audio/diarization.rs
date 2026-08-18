// Speaker diarization analysis for HiNotes audio recordings

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a time segment where a specific speaker was talking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeakerSegment {
    pub id: String,
    pub note_id: String,
    pub speaker_id: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

/// Represents a speaker profile with voice characteristics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeakerProfile {
    pub id: String,
    pub name: Option<String>,
    pub voice_signature: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Statistics about a speaker's participation in a recording
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeakerStatistics {
    pub speaker_id: String,
    pub total_talk_time: f64,
    pub turn_count: usize,
    pub average_turn_duration: f64,
    pub percentage_of_total: f64,
}

/// Result of speaker diarization analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiarizationResult {
    pub note_id: String,
    pub segments: Vec<SpeakerSegment>,
    pub speakers: Vec<SpeakerProfile>,
    pub statistics: Vec<SpeakerStatistics>,
    pub total_duration: f64,
}

impl SpeakerSegment {
    /// Get the duration of this segment in seconds
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }

    /// Check if this segment overlaps with another
    pub fn overlaps_with(&self, other: &SpeakerSegment) -> bool {
        self.start_time < other.end_time && other.start_time < self.end_time
    }
}

impl SpeakerStatistics {
    /// Calculate statistics for a speaker from their segments
    pub fn from_segments(
        speaker_id: String,
        segments: &[SpeakerSegment],
        total_duration: f64,
    ) -> Self {
        let total_talk_time: f64 = segments.iter().map(|s| s.duration()).sum();
        let turn_count = segments.len();
        let average_turn_duration = if turn_count > 0 {
            total_talk_time / turn_count as f64
        } else {
            0.0
        };
        let percentage_of_total = if total_duration > 0.0 {
            (total_talk_time / total_duration) * 100.0
        } else {
            0.0
        };

        Self {
            speaker_id,
            total_talk_time,
            turn_count,
            average_turn_duration,
            percentage_of_total,
        }
    }
}

/// Speaker diarization analyzer
pub struct Diarizer {
    // Placeholder for future audio processing capabilities
}

impl Diarizer {
    pub fn new() -> Self {
        Self {}
    }

    /// Analyze an audio file for speaker diarization (placeholder)
    pub fn analyze_audio(&self, _audio_path: &str) -> Result<DiarizationResult> {
        // This is a placeholder - actual implementation would use ML models
        anyhow::bail!("Audio analysis not yet implemented - requires ML model integration")
    }
}

impl Default for Diarizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_segment(
        note_id: &str,
        speaker_id: &str,
        start: f64,
        end: f64,
        confidence: f64,
    ) -> SpeakerSegment {
        SpeakerSegment {
            id: uuid::Uuid::new_v4().to_string(),
            note_id: note_id.to_string(),
            speaker_id: speaker_id.to_string(),
            start_time: start,
            end_time: end,
            confidence,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_speaker_segment_duration() {
        let segment = create_test_segment("note-1", "speaker-1", 0.0, 10.5, 0.95);
        assert_eq!(segment.duration(), 10.5);
    }

    #[test]
    fn test_speaker_segment_overlaps() {
        let seg1 = create_test_segment("note-1", "speaker-1", 0.0, 10.0, 0.95);
        let seg2 = create_test_segment("note-1", "speaker-2", 5.0, 15.0, 0.90);
        let seg3 = create_test_segment("note-1", "speaker-1", 20.0, 30.0, 0.92);

        assert!(seg1.overlaps_with(&seg2));
        assert!(seg2.overlaps_with(&seg1));
        assert!(!seg1.overlaps_with(&seg3));
        assert!(!seg3.overlaps_with(&seg1));
    }

    #[test]
    fn test_speaker_segment_no_overlap_adjacent() {
        let seg1 = create_test_segment("note-1", "speaker-1", 0.0, 10.0, 0.95);
        let seg2 = create_test_segment("note-1", "speaker-2", 10.0, 20.0, 0.90);

        // Adjacent segments should not overlap
        assert!(!seg1.overlaps_with(&seg2));
    }

    #[test]
    fn test_speaker_statistics_from_single_segment() {
        let segments = vec![create_test_segment("note-1", "speaker-1", 0.0, 10.0, 0.95)];
        let stats = SpeakerStatistics::from_segments("speaker-1".to_string(), &segments, 20.0);

        assert_eq!(stats.speaker_id, "speaker-1");
        assert_eq!(stats.total_talk_time, 10.0);
        assert_eq!(stats.turn_count, 1);
        assert_eq!(stats.average_turn_duration, 10.0);
        assert_eq!(stats.percentage_of_total, 50.0);
    }

    #[test]
    fn test_speaker_statistics_from_multiple_segments() {
        let segments = vec![
            create_test_segment("note-1", "speaker-1", 0.0, 10.0, 0.95),
            create_test_segment("note-1", "speaker-1", 20.0, 25.0, 0.92),
            create_test_segment("note-1", "speaker-1", 30.0, 35.0, 0.90),
        ];
        let stats = SpeakerStatistics::from_segments("speaker-1".to_string(), &segments, 60.0);

        assert_eq!(stats.speaker_id, "speaker-1");
        assert_eq!(stats.total_talk_time, 20.0); // 10 + 5 + 5
        assert_eq!(stats.turn_count, 3);
        assert_eq!(stats.average_turn_duration, 20.0 / 3.0);
        assert!((stats.percentage_of_total - 33.333333).abs() < 0.001);
    }

    #[test]
    fn test_speaker_statistics_empty_segments() {
        let segments: Vec<SpeakerSegment> = vec![];
        let stats = SpeakerStatistics::from_segments("speaker-1".to_string(), &segments, 60.0);

        assert_eq!(stats.speaker_id, "speaker-1");
        assert_eq!(stats.total_talk_time, 0.0);
        assert_eq!(stats.turn_count, 0);
        assert_eq!(stats.average_turn_duration, 0.0);
        assert_eq!(stats.percentage_of_total, 0.0);
    }

    #[test]
    fn test_speaker_statistics_zero_duration() {
        let segments = vec![create_test_segment("note-1", "speaker-1", 0.0, 10.0, 0.95)];
        let stats = SpeakerStatistics::from_segments("speaker-1".to_string(), &segments, 0.0);

        assert_eq!(stats.percentage_of_total, 0.0);
    }

    #[test]
    fn test_diarizer_creation() {
        let diarizer = Diarizer::new();
        assert!(std::mem::size_of_val(&diarizer) >= 0);
    }

    #[test]
    fn test_diarizer_default() {
        let diarizer = Diarizer::default();
        assert!(std::mem::size_of_val(&diarizer) >= 0);
    }

    #[test]
    fn test_diarizer_analyze_audio_not_implemented() {
        let diarizer = Diarizer::new();
        let result = diarizer.analyze_audio("/path/to/audio.wav");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not yet implemented"));
    }
}
