// Transcription module with speaker labeling and export functionality

use crate::audio::diarization::{DiarizationResult, SpeakerProfile, SpeakerSegment};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a word or phrase in the transcription with timestamp
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranscriptWord {
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f64,
}

/// A complete transcription segment with speaker label
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranscriptSegment {
    pub speaker_id: String,
    pub speaker_label: String,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub words: Vec<TranscriptWord>,
}

/// Complete transcription with speaker information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcription {
    pub note_id: String,
    pub segments: Vec<TranscriptSegment>,
    pub speakers: HashMap<String, SpeakerProfile>,
    pub total_duration: f64,
}

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    PlainText,
    Srt,
    Json,
}

/// Color options for speaker differentiation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeakerColor {
    pub speaker_id: String,
    pub color: String, // hex color like "#FF5733"
}

impl Transcription {
    /// Create a new transcription from diarization result and text
    pub fn new(note_id: String, diarization: DiarizationResult, text: String) -> Result<Self> {
        // Build speaker map
        let mut speakers = HashMap::new();
        for speaker in diarization.speakers {
            speakers.insert(speaker.id.clone(), speaker);
        }

        // Generate segments by mapping diarization segments to text
        let segments = Self::map_text_to_segments(&diarization.segments, &text, &speakers)?;

        Ok(Self {
            note_id,
            segments,
            speakers,
            total_duration: diarization.total_duration,
        })
    }

    /// Map text to speaker segments (simplified implementation)
    fn map_text_to_segments(
        diarization_segments: &[SpeakerSegment],
        text: &str,
        speakers: &HashMap<String, SpeakerProfile>,
    ) -> Result<Vec<TranscriptSegment>> {
        let mut segments = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();

        if words.is_empty() {
            return Ok(segments);
        }

        // Distribute words across segments based on timing
        let mut word_index = 0;
        let words_per_segment = if diarization_segments.is_empty() {
            words.len()
        } else {
            words.len().div_ceil(diarization_segments.len())
        };

        for seg in diarization_segments {
            let speaker_label = speakers
                .get(&seg.speaker_id)
                .and_then(|s| s.name.clone())
                .unwrap_or_else(|| format!("Speaker {}", seg.speaker_id));

            let end_index = std::cmp::min(word_index + words_per_segment, words.len());
            let segment_words = &words[word_index..end_index];
            let segment_text = segment_words.join(" ");

            // Create simple word-level timestamps
            let word_duration = (seg.end_time - seg.start_time) / segment_words.len() as f64;
            let mut transcript_words = Vec::new();

            for (i, word) in segment_words.iter().enumerate() {
                let word_start = seg.start_time + (i as f64 * word_duration);
                let word_end = word_start + word_duration;

                transcript_words.push(TranscriptWord {
                    text: word.to_string(),
                    start_time: word_start,
                    end_time: word_end,
                    confidence: seg.confidence,
                });
            }

            segments.push(TranscriptSegment {
                speaker_id: seg.speaker_id.clone(),
                speaker_label,
                text: segment_text,
                start_time: seg.start_time,
                end_time: seg.end_time,
                words: transcript_words,
            });

            word_index = end_index;
        }

        Ok(segments)
    }

    /// Export transcription to specified format
    pub fn export(&self, format: ExportFormat) -> Result<String> {
        match format {
            ExportFormat::PlainText => self.export_plain_text(),
            ExportFormat::Srt => self.export_srt(),
            ExportFormat::Json => self.export_json(),
        }
    }

    /// Export as plain text with speaker labels and timestamps
    fn export_plain_text(&self) -> Result<String> {
        let mut output = String::new();

        for segment in &self.segments {
            let timestamp = format_timestamp(segment.start_time);
            output.push_str(&format!(
                "[{}] {}: {}\n\n",
                timestamp, segment.speaker_label, segment.text
            ));
        }

        Ok(output)
    }

    /// Export as SRT subtitle format
    fn export_srt(&self) -> Result<String> {
        let mut output = String::new();

        for (index, segment) in self.segments.iter().enumerate() {
            let start = format_srt_timestamp(segment.start_time);
            let end = format_srt_timestamp(segment.end_time);

            output.push_str(&format!(
                "{}\n{} --> {}\n{}: {}\n\n",
                index + 1,
                start,
                end,
                segment.speaker_label,
                segment.text
            ));
        }

        Ok(output)
    }

    /// Export as JSON
    fn export_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize to JSON: {}", e))
    }

    /// Rename a speaker across all segments
    pub fn rename_speaker(&mut self, speaker_id: &str, new_name: String) -> Result<()> {
        // Update speaker profile
        if let Some(profile) = self.speakers.get_mut(speaker_id) {
            profile.name = Some(new_name.clone());
        } else {
            return Err(anyhow!("Speaker not found: {}", speaker_id));
        }

        // Update all segments with this speaker
        for segment in &mut self.segments {
            if segment.speaker_id == speaker_id {
                segment.speaker_label = new_name.clone();
            }
        }

        Ok(())
    }

    /// Generate color-coded HTML transcription
    pub fn export_html_with_colors(&self, color_map: &HashMap<String, String>) -> Result<String> {
        let mut output = String::from("<html><head><style>body { font-family: Arial, sans-serif; line-height: 1.6; }</style></head><body>\n");

        for segment in &self.segments {
            let color = color_map
                .get(&segment.speaker_id)
                .cloned()
                .unwrap_or_else(|| "#000000".to_string());

            let timestamp = format_timestamp(segment.start_time);
            output.push_str(&format!(
                "<p><span style=\"color: {}; font-weight: bold;\">[{}] {}:</span> {}</p>\n",
                color, timestamp, segment.speaker_label, segment.text
            ));
        }

        output.push_str("</body></html>");
        Ok(output)
    }

    /// Get all unique speaker IDs in the transcription
    pub fn get_speaker_ids(&self) -> Vec<String> {
        self.speakers.keys().cloned().collect()
    }

    /// Get speaker statistics (talk time, turn count, etc.)
    pub fn get_speaker_stats(&self) -> HashMap<String, SpeakerStats> {
        let mut stats: HashMap<String, SpeakerStats> = HashMap::new();

        for segment in &self.segments {
            let stat = stats
                .entry(segment.speaker_id.clone())
                .or_insert(SpeakerStats {
                    speaker_id: segment.speaker_id.clone(),
                    speaker_label: segment.speaker_label.clone(),
                    total_talk_time: 0.0,
                    turn_count: 0,
                    word_count: 0,
                });

            stat.total_talk_time += segment.end_time - segment.start_time;
            stat.turn_count += 1;
            stat.word_count += segment.words.len();
        }

        stats
    }
}

/// Statistics about a speaker in the transcription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerStats {
    pub speaker_id: String,
    pub speaker_label: String,
    pub total_talk_time: f64,
    pub turn_count: usize,
    pub word_count: usize,
}

/// Format timestamp as HH:MM:SS
fn format_timestamp(seconds: f64) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

/// Format timestamp for SRT format (HH:MM:SS,mmm)
fn format_srt_timestamp(seconds: f64) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = seconds % 60.0;
    let whole_secs = secs as u32;
    let millis = ((secs - whole_secs as f64) * 1000.0) as u32;
    format!(
        "{:02}:{:02}:{:02},{:03}",
        hours, minutes, whole_secs, millis
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::diarization::{DiarizationResult, SpeakerProfile, SpeakerSegment};
    use chrono::Utc;

    fn create_test_diarization() -> DiarizationResult {
        let speaker1 = SpeakerProfile {
            id: "speaker-1".to_string(),
            name: Some("Alice".to_string()),
            voice_signature: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let speaker2 = SpeakerProfile {
            id: "speaker-2".to_string(),
            name: Some("Bob".to_string()),
            voice_signature: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let segments = vec![
            SpeakerSegment {
                id: "seg-1".to_string(),
                note_id: "note-1".to_string(),
                speaker_id: "speaker-1".to_string(),
                start_time: 0.0,
                end_time: 5.0,
                confidence: 0.95,
                created_at: Utc::now(),
            },
            SpeakerSegment {
                id: "seg-2".to_string(),
                note_id: "note-1".to_string(),
                speaker_id: "speaker-2".to_string(),
                start_time: 5.0,
                end_time: 10.0,
                confidence: 0.92,
                created_at: Utc::now(),
            },
        ];

        DiarizationResult {
            note_id: "note-1".to_string(),
            segments,
            speakers: vec![speaker1, speaker2],
            statistics: vec![],
            total_duration: 10.0,
        }
    }

    #[test]
    fn test_transcription_creation() {
        let diarization = create_test_diarization();
        let text = "Hello world this is a test transcription from multiple speakers";

        let transcription = Transcription::new("note-1".to_string(), diarization, text.to_string());

        assert!(transcription.is_ok());
        let trans = transcription.unwrap();
        assert_eq!(trans.note_id, "note-1");
        assert_eq!(trans.segments.len(), 2);
        assert_eq!(trans.speakers.len(), 2);
    }

    #[test]
    fn test_transcription_speaker_labels() {
        let diarization = create_test_diarization();
        let text = "Hello world this is a test";

        let transcription =
            Transcription::new("note-1".to_string(), diarization, text.to_string()).unwrap();

        assert_eq!(transcription.segments[0].speaker_label, "Alice");
        assert_eq!(transcription.segments[1].speaker_label, "Bob");
    }

    #[test]
    fn test_export_plain_text() {
        let diarization = create_test_diarization();
        let text = "Hello world this is a test";

        let transcription =
            Transcription::new("note-1".to_string(), diarization, text.to_string()).unwrap();

        let output = transcription.export(ExportFormat::PlainText).unwrap();

        assert!(output.contains("Alice:"));
        assert!(output.contains("Bob:"));
        assert!(output.contains("[00:00:00]"));
    }

    #[test]
    fn test_export_srt_format() {
        let diarization = create_test_diarization();
        let text = "Hello world this is a test";

        let transcription =
            Transcription::new("note-1".to_string(), diarization, text.to_string()).unwrap();

        let output = transcription.export(ExportFormat::Srt).unwrap();

        assert!(output.contains("1\n"));
        assert!(output.contains("-->"));
        assert!(output.contains("00:00:00,000"));
        assert!(output.contains("Alice:"));
    }

    #[test]
    fn test_export_json_format() {
        let diarization = create_test_diarization();
        let text = "Hello world";

        let transcription =
            Transcription::new("note-1".to_string(), diarization, text.to_string()).unwrap();

        let output = transcription.export(ExportFormat::Json).unwrap();

        assert!(output.contains("\"note_id\""));
        assert!(output.contains("\"segments\""));
        assert!(output.contains("\"speakers\""));
    }

    #[test]
    fn test_rename_speaker() {
        let diarization = create_test_diarization();
        let text = "Hello world this is a test";

        let mut transcription =
            Transcription::new("note-1".to_string(), diarization, text.to_string()).unwrap();

        let result = transcription.rename_speaker("speaker-1", "John".to_string());

        assert!(result.is_ok());
        assert_eq!(transcription.segments[0].speaker_label, "John");
        assert_eq!(
            transcription.speakers.get("speaker-1").unwrap().name,
            Some("John".to_string())
        );
    }

    #[test]
    fn test_rename_nonexistent_speaker() {
        let diarization = create_test_diarization();
        let text = "Hello world";

        let mut transcription =
            Transcription::new("note-1".to_string(), diarization, text.to_string()).unwrap();

        let result = transcription.rename_speaker("speaker-999", "Nobody".to_string());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Speaker not found"));
    }

    #[test]
    fn test_export_html_with_colors() {
        let diarization = create_test_diarization();
        let text = "Hello world";

        let transcription =
            Transcription::new("note-1".to_string(), diarization, text.to_string()).unwrap();

        let mut color_map = HashMap::new();
        color_map.insert("speaker-1".to_string(), "#FF5733".to_string());
        color_map.insert("speaker-2".to_string(), "#33FF57".to_string());

        let output = transcription.export_html_with_colors(&color_map).unwrap();

        assert!(output.contains("<html>"));
        assert!(output.contains("#FF5733"));
        assert!(output.contains("#33FF57"));
        assert!(output.contains("Alice:"));
        assert!(output.contains("Bob:"));
    }

    #[test]
    fn test_get_speaker_ids() {
        let diarization = create_test_diarization();
        let text = "Hello world";

        let transcription =
            Transcription::new("note-1".to_string(), diarization, text.to_string()).unwrap();

        let ids = transcription.get_speaker_ids();

        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"speaker-1".to_string()));
        assert!(ids.contains(&"speaker-2".to_string()));
    }

    #[test]
    fn test_get_speaker_stats() {
        let diarization = create_test_diarization();
        let text = "Hello world this is test";

        let transcription =
            Transcription::new("note-1".to_string(), diarization, text.to_string()).unwrap();

        let stats = transcription.get_speaker_stats();

        assert_eq!(stats.len(), 2);

        let alice_stats = stats.get("speaker-1").unwrap();
        assert_eq!(alice_stats.speaker_label, "Alice");
        assert_eq!(alice_stats.turn_count, 1);
        assert!(alice_stats.total_talk_time > 0.0);
        assert!(alice_stats.word_count > 0);
    }

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0.0), "00:00:00");
        assert_eq!(format_timestamp(65.0), "00:01:05");
        assert_eq!(format_timestamp(3665.0), "01:01:05");
    }

    #[test]
    fn test_format_srt_timestamp() {
        assert_eq!(format_srt_timestamp(0.0), "00:00:00,000");
        assert_eq!(format_srt_timestamp(65.5), "00:01:05,500");
        assert_eq!(format_srt_timestamp(3665.123), "01:01:05,123");
    }

    #[test]
    fn test_empty_text_transcription() {
        let diarization = create_test_diarization();
        let text = "";

        let transcription =
            Transcription::new("note-1".to_string(), diarization, text.to_string()).unwrap();

        assert_eq!(transcription.segments.len(), 0);
    }

    #[test]
    fn test_word_level_timestamps() {
        let diarization = create_test_diarization();
        let text = "Hello world";

        let transcription =
            Transcription::new("note-1".to_string(), diarization, text.to_string()).unwrap();

        // Check first segment has word-level timestamps
        assert!(!transcription.segments[0].words.is_empty());
        let first_word = &transcription.segments[0].words[0];
        assert!(first_word.start_time >= 0.0);
        assert!(first_word.end_time > first_word.start_time);
    }

    #[test]
    fn test_transcription_with_default_speaker_name() {
        let mut diarization = create_test_diarization();
        // Remove name from first speaker
        diarization.speakers[0].name = None;

        let text = "Hello world";

        let transcription =
            Transcription::new("note-1".to_string(), diarization, text.to_string()).unwrap();

        // Should use default "Speaker {id}" format
        assert!(transcription.segments[0]
            .speaker_label
            .starts_with("Speaker "));
    }
}
