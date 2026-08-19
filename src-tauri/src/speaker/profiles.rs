// Speaker profile management for cross-session speaker recognition

use crate::db::types::{InsertSpeaker, InsertSpeakerSegment, Speaker, SpeakerSegment, UpdateSpeaker};
use crate::db::Database;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// History entry for speaker merge operations (for undo capability)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeakerMergeHistory {
    pub id: String,
    pub note_id: String,
    pub target_speaker_id: String,
    pub source_speaker_ids: Vec<String>,
    pub merged_at: chrono::DateTime<chrono::Utc>,
    /// Original segments before merge (for potential undo)
    pub original_segments: Vec<SpeakerSegment>,
}

/// Manages speaker profiles and cross-session recognition
pub struct SpeakerProfileManager {
    /// In-memory cache of merge history for undo capability
    merge_history: Vec<SpeakerMergeHistory>,
}

impl SpeakerProfileManager {
    /// Create a new speaker profile manager
    pub fn new() -> Self {
        Self {
            merge_history: Vec::new(),
        }
    }

    /// Get speaker segments for a note
    pub fn get_speakers_for_note(&self, db: &Database, note_id: &str) -> Result<Vec<SpeakerSegment>> {
        db.list_speaker_segments_for_note(note_id)
    }

    /// Update speaker label/name
    pub fn update_speaker_label(&self, db: &Database, speaker_id: &str, new_name: &str) -> Result<Speaker> {
        let update = UpdateSpeaker {
            name: Some(new_name.to_string()),
            voice_signature: None,
        };
        db.update_speaker(speaker_id, &update)
    }

    /// Merge multiple speakers into one
    ///
    /// This operation:
    /// 1. Updates all segments from source speakers to point to target speaker
    /// 2. Deletes source speaker profiles
    /// 3. Records merge history for potential undo
    pub fn merge_speakers(
        &mut self,
        db: &Database,
        note_id: &str,
        target_speaker_id: &str,
        source_speaker_ids: &[String],
    ) -> Result<Vec<SpeakerSegment>> {
        // Step 1: Get all segments that will be affected
        let all_segments = self.get_speakers_for_note(db, note_id)?;
        let affected_segments: Vec<SpeakerSegment> = all_segments
            .iter()
            .filter(|s| source_speaker_ids.contains(&s.speaker_id) || s.speaker_id == target_speaker_id)
            .cloned()
            .collect();

        // Step 2: Record merge history for undo
        let merge_record = SpeakerMergeHistory {
            id: Uuid::new_v4().to_string(),
            note_id: note_id.to_string(),
            target_speaker_id: target_speaker_id.to_string(),
            source_speaker_ids: source_speaker_ids.to_vec(),
            merged_at: Utc::now(),
            original_segments: affected_segments.clone(),
        };
        self.merge_history.push(merge_record);

        // Step 3: Update segments to point to target speaker
        for source_id in source_speaker_ids {
            db.update_segment_speaker(note_id, source_id, target_speaker_id)
                .with_context(|| format!("Failed to update segments from {} to {}", source_id, target_speaker_id))?;
        }

        // Step 4: Delete source speaker profiles (only if no other notes use them)
        for source_id in source_speaker_ids {
            let segments_count = db.count_segments_for_speaker(source_id)?;
            if segments_count == 0 {
                db.delete_speaker(source_id)
                    .with_context(|| format!("Failed to delete speaker {}", source_id))?;
            }
        }

        // Step 5: Return updated segments
        self.get_speakers_for_note(db, note_id)
    }

    /// Split a speaker segment into two parts
    ///
    /// This creates a new segment from split_time to end_time, and updates
    /// the original segment to end at split_time.
    pub fn split_segment(
        &self,
        db: &Database,
        segment_id: &str,
        split_time: f64,
        new_speaker_id: Option<String>,
    ) -> Result<(SpeakerSegment, SpeakerSegment)> {
        // Step 1: Get the original segment
        let original = db
            .get_speaker_segment(segment_id)?
            .context("Segment not found")?;

        // Validate split time
        if split_time <= original.start_time || split_time >= original.end_time {
            anyhow::bail!(
                "Split time {} must be between {} and {}",
                split_time,
                original.start_time,
                original.end_time
            );
        }

        // Step 2: Create the new speaker if needed
        let second_speaker_id = if let Some(new_id) = new_speaker_id {
            // Check if speaker exists, if not create it
            if db.get_speaker(&new_id)?.is_none() {
                let speaker_number = db.count_speakers()? + 1;
                let insert_speaker = InsertSpeaker {
                    id: new_id.clone(),
                    name: Some(format!("Speaker {}", speaker_number)),
                    voice_signature: None,
                };
                db.insert_speaker(&insert_speaker)?;
            }
            new_id
        } else {
            original.speaker_id.clone()
        };

        // Step 3: Update the original segment (shorten it)
        let updated_first = db.update_segment_end_time(segment_id, split_time)?;

        // Step 4: Create the second segment
        let new_segment = InsertSpeakerSegment {
            id: Uuid::new_v4().to_string(),
            note_id: original.note_id.clone(),
            speaker_id: second_speaker_id,
            start_time: split_time,
            end_time: original.end_time,
            confidence: original.confidence,
        };
        let created_second = db.insert_speaker_segment(&new_segment)?;

        Ok((updated_first, created_second))
    }

    /// Get merge history for potential undo
    pub fn get_merge_history(&self) -> &[SpeakerMergeHistory] {
        &self.merge_history
    }

    /// Undo the last merge operation for a note
    pub fn undo_last_merge(&mut self, db: &Database, note_id: &str) -> Result<Vec<SpeakerSegment>> {
        // Find the most recent merge for this note
        let merge_idx = self
            .merge_history
            .iter()
            .rposition(|h| h.note_id == note_id)
            .context("No merge history found for this note")?;

        let merge = self.merge_history.remove(merge_idx);

        // Re-create source speakers if they don't exist
        for source_id in &merge.source_speaker_ids {
            if db.get_speaker(source_id)?.is_none() {
                let speaker_number = db.count_speakers()? + 1;
                let insert_speaker = InsertSpeaker {
                    id: source_id.clone(),
                    name: Some(format!("Speaker {}", speaker_number)),
                    voice_signature: None,
                };
                db.insert_speaker(&insert_speaker)?;
            }
        }

        // Delete all current segments for this note
        db.delete_segments_for_note(note_id)?;

        // Re-insert original segments
        for segment in &merge.original_segments {
            let insert = InsertSpeakerSegment {
                id: segment.id.clone(),
                note_id: segment.note_id.clone(),
                speaker_id: segment.speaker_id.clone(),
                start_time: segment.start_time,
                end_time: segment.end_time,
                confidence: segment.confidence,
            };
            db.insert_speaker_segment(&insert)?;
        }

        self.get_speakers_for_note(db, note_id)
    }

    /// Match a voice signature against known speakers
    ///
    /// This would use acoustic features to identify if a speaker has been
    /// seen in previous recordings. Returns speaker IDs ranked by similarity.
    pub fn match_voice_signature(
        &self,
        _db: &Database,
        _voice_signature: &str,
    ) -> Result<Vec<(String, f64)>> {
        // TODO: Implement actual voice matching algorithm
        // For now, return empty results
        Ok(vec![])
    }

    /// Generate statistics for speakers in a note
    pub fn calculate_speaker_statistics(
        &self,
        db: &Database,
        note_id: &str,
    ) -> Result<HashMap<String, SpeakerStats>> {
        let segments = self.get_speakers_for_note(db, note_id)?;

        let mut stats_map: HashMap<String, SpeakerStats> = HashMap::new();

        for segment in &segments {
            let stats = stats_map
                .entry(segment.speaker_id.clone())
                .or_insert_with(|| SpeakerStats {
                    speaker_id: segment.speaker_id.clone(),
                    total_talk_time: 0.0,
                    turn_count: 0,
                    average_turn_duration: 0.0,
                    percentage_of_total: 0.0,
                });

            stats.total_talk_time += segment.end_time - segment.start_time;
            stats.turn_count += 1;
        }

        // Calculate total duration and percentages
        let total_duration: f64 = segments.iter().map(|s| s.end_time - s.start_time).sum();

        for stats in stats_map.values_mut() {
            stats.average_turn_duration = if stats.turn_count > 0 {
                stats.total_talk_time / stats.turn_count as f64
            } else {
                0.0
            };
            stats.percentage_of_total = if total_duration > 0.0 {
                (stats.total_talk_time / total_duration) * 100.0
            } else {
                0.0
            };
        }

        Ok(stats_map)
    }
}

impl Default for SpeakerProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about a speaker's participation in a recording
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeakerStats {
    pub speaker_id: String,
    pub total_talk_time: f64,
    pub turn_count: usize,
    pub average_turn_duration: f64,
    pub percentage_of_total: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn setup_test_db() -> Database {
        Database::new_in_memory().unwrap()
    }

    fn create_test_speaker(db: &Database, id: &str, name: &str) -> Speaker {
        let insert = InsertSpeaker {
            id: id.to_string(),
            name: Some(name.to_string()),
            voice_signature: None,
        };
        db.insert_speaker(&insert).unwrap()
    }

    fn create_test_segment(
        db: &Database,
        note_id: &str,
        speaker_id: &str,
        start: f64,
        end: f64,
    ) -> SpeakerSegment {
        let insert = InsertSpeakerSegment {
            id: Uuid::new_v4().to_string(),
            note_id: note_id.to_string(),
            speaker_id: speaker_id.to_string(),
            start_time: start,
            end_time: end,
            confidence: 0.9,
        };
        db.insert_speaker_segment(&insert).unwrap()
    }

    #[test]
    fn test_profile_manager_creation() {
        let manager = SpeakerProfileManager::new();
        assert_eq!(manager.merge_history.len(), 0);
    }

    #[test]
    fn test_update_speaker_label() {
        let db = setup_test_db();
        let manager = SpeakerProfileManager::new();

        create_test_speaker(&db, "speaker-1", "Speaker 1");

        let updated = manager
            .update_speaker_label(&db, "speaker-1", "John Doe")
            .unwrap();

        assert_eq!(updated.name, Some("John Doe".to_string()));
    }

    #[test]
    fn test_merge_speakers() {
        let db = setup_test_db();
        let mut manager = SpeakerProfileManager::new();

        // Create speakers
        create_test_speaker(&db, "speaker-1", "Speaker 1");
        create_test_speaker(&db, "speaker-2", "Speaker 2");
        create_test_speaker(&db, "speaker-3", "Speaker 3");

        // Create segments
        create_test_segment(&db, "note-1", "speaker-1", 0.0, 5.0);
        create_test_segment(&db, "note-1", "speaker-2", 5.0, 10.0);
        create_test_segment(&db, "note-1", "speaker-3", 10.0, 15.0);
        create_test_segment(&db, "note-1", "speaker-1", 15.0, 20.0);

        // Merge speaker-2 and speaker-3 into speaker-1
        let result = manager
            .merge_speakers(
                &db,
                "note-1",
                "speaker-1",
                &["speaker-2".to_string(), "speaker-3".to_string()],
            )
            .unwrap();

        // All segments should now belong to speaker-1
        for segment in &result {
            assert_eq!(segment.speaker_id, "speaker-1");
        }

        // Should have 4 segments total
        assert_eq!(result.len(), 4);

        // Merge history should be recorded
        assert_eq!(manager.merge_history.len(), 1);
        assert_eq!(manager.merge_history[0].target_speaker_id, "speaker-1");
    }

    #[test]
    fn test_split_segment() {
        let db = setup_test_db();
        let manager = SpeakerProfileManager::new();

        create_test_speaker(&db, "speaker-1", "Speaker 1");
        let segment = create_test_segment(&db, "note-1", "speaker-1", 0.0, 10.0);

        // Split at 5 seconds
        let (first, second) = manager
            .split_segment(&db, &segment.id, 5.0, None)
            .unwrap();

        assert_eq!(first.start_time, 0.0);
        assert_eq!(first.end_time, 5.0);
        assert_eq!(first.speaker_id, "speaker-1");

        assert_eq!(second.start_time, 5.0);
        assert_eq!(second.end_time, 10.0);
        assert_eq!(second.speaker_id, "speaker-1");
    }

    #[test]
    fn test_split_segment_with_new_speaker() {
        let db = setup_test_db();
        let manager = SpeakerProfileManager::new();

        create_test_speaker(&db, "speaker-1", "Speaker 1");
        let segment = create_test_segment(&db, "note-1", "speaker-1", 0.0, 10.0);

        // Split at 5 seconds and assign second part to new speaker
        let (first, second) = manager
            .split_segment(&db, &segment.id, 5.0, Some("speaker-2".to_string()))
            .unwrap();

        assert_eq!(first.speaker_id, "speaker-1");
        assert_eq!(second.speaker_id, "speaker-2");

        // New speaker should have been created
        let new_speaker = db.get_speaker("speaker-2").unwrap();
        assert!(new_speaker.is_some());
    }

    #[test]
    fn test_split_segment_invalid_time() {
        let db = setup_test_db();
        let manager = SpeakerProfileManager::new();

        create_test_speaker(&db, "speaker-1", "Speaker 1");
        let segment = create_test_segment(&db, "note-1", "speaker-1", 5.0, 10.0);

        // Try to split before segment start
        let result = manager.split_segment(&db, &segment.id, 3.0, None);
        assert!(result.is_err());

        // Try to split after segment end
        let result = manager.split_segment(&db, &segment.id, 12.0, None);
        assert!(result.is_err());

        // Try to split at segment boundaries
        let result = manager.split_segment(&db, &segment.id, 5.0, None);
        assert!(result.is_err());

        let result = manager.split_segment(&db, &segment.id, 10.0, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_speaker_statistics() {
        let db = setup_test_db();
        let manager = SpeakerProfileManager::new();

        create_test_speaker(&db, "speaker-1", "Speaker 1");
        create_test_speaker(&db, "speaker-2", "Speaker 2");

        // Speaker 1: 15 seconds total (3 turns)
        create_test_segment(&db, "note-1", "speaker-1", 0.0, 5.0);
        create_test_segment(&db, "note-1", "speaker-1", 10.0, 15.0);
        create_test_segment(&db, "note-1", "speaker-1", 20.0, 25.0);

        // Speaker 2: 10 seconds total (2 turns)
        create_test_segment(&db, "note-1", "speaker-2", 5.0, 10.0);
        create_test_segment(&db, "note-1", "speaker-2", 15.0, 20.0);

        let stats = manager.calculate_speaker_statistics(&db, "note-1").unwrap();

        assert_eq!(stats.len(), 2);

        let speaker1_stats = stats.get("speaker-1").unwrap();
        assert_eq!(speaker1_stats.total_talk_time, 15.0);
        assert_eq!(speaker1_stats.turn_count, 3);
        assert_eq!(speaker1_stats.average_turn_duration, 5.0);
        assert!((speaker1_stats.percentage_of_total - 60.0).abs() < 0.1);

        let speaker2_stats = stats.get("speaker-2").unwrap();
        assert_eq!(speaker2_stats.total_talk_time, 10.0);
        assert_eq!(speaker2_stats.turn_count, 2);
        assert_eq!(speaker2_stats.average_turn_duration, 5.0);
        assert!((speaker2_stats.percentage_of_total - 40.0).abs() < 0.1);
    }

    #[test]
    fn test_undo_merge() {
        let db = setup_test_db();
        let mut manager = SpeakerProfileManager::new();

        // Create speakers
        create_test_speaker(&db, "speaker-1", "Speaker 1");
        create_test_speaker(&db, "speaker-2", "Speaker 2");

        // Create segments
        create_test_segment(&db, "note-1", "speaker-1", 0.0, 5.0);
        create_test_segment(&db, "note-1", "speaker-2", 5.0, 10.0);
        create_test_segment(&db, "note-1", "speaker-1", 10.0, 15.0);

        // Merge speaker-2 into speaker-1
        manager
            .merge_speakers(&db, "note-1", "speaker-1", &["speaker-2".to_string()])
            .unwrap();

        // Verify merge
        let merged_segments = manager.get_speakers_for_note(&db, "note-1").unwrap();
        assert!(merged_segments.iter().all(|s| s.speaker_id == "speaker-1"));

        // Undo merge
        let restored_segments = manager.undo_last_merge(&db, "note-1").unwrap();

        // Verify restoration
        assert_eq!(restored_segments.len(), 3);
        let speaker2_segments: Vec<_> = restored_segments
            .iter()
            .filter(|s| s.speaker_id == "speaker-2")
            .collect();
        assert_eq!(speaker2_segments.len(), 1);
    }
}
