// Speaker color assignment for visual identification in UI

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Predefined speaker colors (cycling through 8 colors)
pub const SPEAKER_COLORS: [&str; 8] = [
    "#3B82F6", // Blue
    "#EF4444", // Red
    "#10B981", // Green
    "#F59E0B", // Amber
    "#8B5CF6", // Purple
    "#EC4899", // Pink
    "#06B6D4", // Cyan
    "#F97316", // Orange
];

/// Color information for a speaker
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeakerColor {
    pub speaker_id: String,
    pub color: String,
    pub index: usize,
}

/// Manages color assignment for speakers in a recording
#[derive(Debug, Clone)]
pub struct SpeakerColorAssigner {
    assignments: HashMap<String, SpeakerColor>,
    next_index: usize,
}

impl SpeakerColorAssigner {
    /// Create a new color assigner
    pub fn new() -> Self {
        Self {
            assignments: HashMap::new(),
            next_index: 0,
        }
    }

    /// Assign a color to a speaker (idempotent - returns existing if already assigned)
    pub fn assign_color(&mut self, speaker_id: &str) -> SpeakerColor {
        if let Some(color) = self.assignments.get(speaker_id) {
            return color.clone();
        }

        let index = self.next_index % SPEAKER_COLORS.len();
        let color = SpeakerColor {
            speaker_id: speaker_id.to_string(),
            color: SPEAKER_COLORS[index].to_string(),
            index,
        };

        self.assignments
            .insert(speaker_id.to_string(), color.clone());
        self.next_index += 1;

        color
    }

    /// Get the color for a speaker (if already assigned)
    pub fn get_color(&self, speaker_id: &str) -> Option<&SpeakerColor> {
        self.assignments.get(speaker_id)
    }

    /// Get all color assignments
    pub fn get_all_assignments(&self) -> Vec<SpeakerColor> {
        self.assignments.values().cloned().collect()
    }

    /// Update the color for a speaker after merge
    pub fn merge_speakers(&mut self, target_id: &str, source_ids: &[String]) {
        // Keep the target's color if it exists
        if let Some(target_color) = self.assignments.get(target_id).cloned() {
            // Remove source speakers
            for source_id in source_ids {
                self.assignments.remove(source_id);
            }
            // Ensure target color is still assigned
            self.assignments.insert(target_id.to_string(), target_color);
        } else {
            // If target doesn't have a color, use the first source's color
            if let Some(source_id) = source_ids.first() {
                if let Some(source_color) = self.assignments.remove(source_id) {
                    let mut new_color = source_color;
                    new_color.speaker_id = target_id.to_string();
                    self.assignments.insert(target_id.to_string(), new_color);
                }
            }
            // Remove remaining source speakers
            for source_id in source_ids.iter().skip(1) {
                self.assignments.remove(source_id);
            }
        }
    }

    /// Remove a speaker's color assignment
    pub fn remove_speaker(&mut self, speaker_id: &str) {
        self.assignments.remove(speaker_id);
    }

    /// Reset all assignments
    pub fn reset(&mut self) {
        self.assignments.clear();
        self.next_index = 0;
    }
}

impl Default for SpeakerColorAssigner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign_color_sequential() {
        let mut assigner = SpeakerColorAssigner::new();

        let color1 = assigner.assign_color("speaker-1");
        assert_eq!(color1.speaker_id, "speaker-1");
        assert_eq!(color1.color, SPEAKER_COLORS[0]);
        assert_eq!(color1.index, 0);

        let color2 = assigner.assign_color("speaker-2");
        assert_eq!(color2.speaker_id, "speaker-2");
        assert_eq!(color2.color, SPEAKER_COLORS[1]);
        assert_eq!(color2.index, 1);
    }

    #[test]
    fn test_assign_color_idempotent() {
        let mut assigner = SpeakerColorAssigner::new();

        let color1 = assigner.assign_color("speaker-1");
        let color2 = assigner.assign_color("speaker-1");

        assert_eq!(color1, color2);
    }

    #[test]
    fn test_color_cycling() {
        let mut assigner = SpeakerColorAssigner::new();

        // Assign 9 speakers (more than the 8 available colors)
        for i in 0..9 {
            let speaker_id = format!("speaker-{}", i);
            let color = assigner.assign_color(&speaker_id);
            let expected_index = i % SPEAKER_COLORS.len();
            assert_eq!(color.index, expected_index);
            assert_eq!(color.color, SPEAKER_COLORS[expected_index]);
        }
    }

    #[test]
    fn test_get_color() {
        let mut assigner = SpeakerColorAssigner::new();
        assigner.assign_color("speaker-1");

        let color = assigner.get_color("speaker-1");
        assert!(color.is_some());
        assert_eq!(color.unwrap().speaker_id, "speaker-1");

        let no_color = assigner.get_color("speaker-999");
        assert!(no_color.is_none());
    }

    #[test]
    fn test_merge_speakers() {
        let mut assigner = SpeakerColorAssigner::new();

        assigner.assign_color("speaker-1");
        assigner.assign_color("speaker-2");
        assigner.assign_color("speaker-3");

        // Merge speaker-2 and speaker-3 into speaker-1
        assigner.merge_speakers(
            "speaker-1",
            &["speaker-2".to_string(), "speaker-3".to_string()],
        );

        // speaker-1 should still have its color
        assert!(assigner.get_color("speaker-1").is_some());

        // speaker-2 and speaker-3 should be removed
        assert!(assigner.get_color("speaker-2").is_none());
        assert!(assigner.get_color("speaker-3").is_none());
    }

    #[test]
    fn test_merge_speakers_target_has_no_color() {
        let mut assigner = SpeakerColorAssigner::new();

        assigner.assign_color("speaker-1");
        assigner.assign_color("speaker-2");

        // Merge into speaker-3 which has no color yet
        assigner.merge_speakers(
            "speaker-3",
            &["speaker-1".to_string(), "speaker-2".to_string()],
        );

        // speaker-3 should now have speaker-1's color
        let color = assigner.get_color("speaker-3");
        assert!(color.is_some());
        assert_eq!(color.unwrap().speaker_id, "speaker-3");

        // speaker-1 and speaker-2 should be removed
        assert!(assigner.get_color("speaker-1").is_none());
        assert!(assigner.get_color("speaker-2").is_none());
    }

    #[test]
    fn test_remove_speaker() {
        let mut assigner = SpeakerColorAssigner::new();
        assigner.assign_color("speaker-1");
        assigner.assign_color("speaker-2");

        assigner.remove_speaker("speaker-1");

        assert!(assigner.get_color("speaker-1").is_none());
        assert!(assigner.get_color("speaker-2").is_some());
    }

    #[test]
    fn test_reset() {
        let mut assigner = SpeakerColorAssigner::new();
        assigner.assign_color("speaker-1");
        assigner.assign_color("speaker-2");

        assigner.reset();

        assert_eq!(assigner.get_all_assignments().len(), 0);
        assert_eq!(assigner.next_index, 0);

        // After reset, should start from first color again
        let color = assigner.assign_color("speaker-new");
        assert_eq!(color.index, 0);
    }

    #[test]
    fn test_get_all_assignments() {
        let mut assigner = SpeakerColorAssigner::new();
        assigner.assign_color("speaker-1");
        assigner.assign_color("speaker-2");
        assigner.assign_color("speaker-3");

        let assignments = assigner.get_all_assignments();
        assert_eq!(assignments.len(), 3);

        // Check that all speakers are present
        let speaker_ids: Vec<String> = assignments.iter().map(|a| a.speaker_id.clone()).collect();
        assert!(speaker_ids.contains(&"speaker-1".to_string()));
        assert!(speaker_ids.contains(&"speaker-2".to_string()));
        assert!(speaker_ids.contains(&"speaker-3".to_string()));
    }
}
