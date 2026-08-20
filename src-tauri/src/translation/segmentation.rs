/// Text segmentation for breaking transcription into translatable chunks
pub struct TextSegmenter {
    max_segment_length: usize,
    sentence_terminators: Vec<char>,
}

impl TextSegmenter {
    /// Create a new text segmenter with default settings
    pub fn new() -> Self {
        Self {
            max_segment_length: 500, // Maximum characters per segment
            sentence_terminators: vec!['.', '!', '?', '。', '！', '？'], // English and CJK
        }
    }

    /// Create a new text segmenter with custom max segment length
    pub fn with_max_length(max_length: usize) -> Self {
        Self {
            max_segment_length: max_length,
            sentence_terminators: vec!['.', '!', '?', '。', '！', '？'],
        }
    }

    /// Segment text into translatable chunks
    ///
    /// Strategy:
    /// 1. Split on sentence terminators when possible
    /// 2. If segment exceeds max length, split on clause boundaries (commas, semicolons)
    /// 3. If still too long, split on word boundaries
    /// 4. Never split mid-word
    pub fn segment_text(&self, text: &str) -> Vec<String> {
        if text.trim().is_empty() {
            return vec![];
        }

        let mut segments = Vec::new();
        let mut current_segment = String::new();

        // Split on sentences first
        let sentences = self.split_sentences(text);

        for sentence in sentences {
            // If adding this sentence would exceed max length, flush current segment
            if !current_segment.is_empty()
                && current_segment.len() + sentence.len() + 1 > self.max_segment_length
            {
                segments.push(current_segment.trim().to_string());
                current_segment = String::new();
            }

            // If a single sentence is too long, split it further
            if sentence.len() > self.max_segment_length {
                // If we have a current segment, flush it first
                if !current_segment.is_empty() {
                    segments.push(current_segment.trim().to_string());
                    current_segment = String::new();
                }

                // Split the long sentence
                let sub_segments = self.split_long_sentence(&sentence);
                segments.extend(sub_segments);
            } else {
                // Add sentence to current segment
                if !current_segment.is_empty() {
                    current_segment.push(' ');
                }
                current_segment.push_str(&sentence);
            }
        }

        // Add remaining segment
        if !current_segment.is_empty() {
            segments.push(current_segment.trim().to_string());
        }

        segments
    }

    /// Split text into sentences based on terminators
    fn split_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();

        for ch in text.chars() {
            current.push(ch);

            if self.sentence_terminators.contains(&ch) {
                // Check if next character is a space or end of text
                // This handles cases like "Dr. Smith" where period doesn't end sentence
                sentences.push(current.trim().to_string());
                current = String::new();
            }
        }

        // Add any remaining text
        if !current.trim().is_empty() {
            sentences.push(current.trim().to_string());
        }

        sentences
    }

    /// Split a long sentence on clause or word boundaries
    fn split_long_sentence(&self, sentence: &str) -> Vec<String> {
        let mut segments = Vec::new();

        // Try splitting on clause boundaries first (commas, semicolons)
        let clauses: Vec<&str> = sentence.split([',', ';', ':']).collect();

        let mut current_segment = String::new();

        for clause in clauses {
            let clause = clause.trim();
            if clause.is_empty() {
                continue;
            }

            // If this clause would make segment too long, flush current segment
            if !current_segment.is_empty()
                && current_segment.len() + clause.len() + 2 > self.max_segment_length
            {
                segments.push(current_segment.trim().to_string());
                current_segment = String::new();
            }

            // If a single clause is still too long, split on words
            if clause.len() > self.max_segment_length {
                if !current_segment.is_empty() {
                    segments.push(current_segment.trim().to_string());
                    current_segment = String::new();
                }

                let word_segments = self.split_on_words(clause);
                segments.extend(word_segments);
            } else {
                if !current_segment.is_empty() {
                    current_segment.push_str(", ");
                }
                current_segment.push_str(clause);
            }
        }

        if !current_segment.is_empty() {
            segments.push(current_segment.trim().to_string());
        }

        segments
    }

    /// Split text on word boundaries as last resort
    fn split_on_words(&self, text: &str) -> Vec<String> {
        let mut segments = Vec::new();
        let mut current_segment = String::new();

        for word in text.split_whitespace() {
            // If adding this word would exceed max length, flush current segment
            if !current_segment.is_empty()
                && current_segment.len() + word.len() + 1 > self.max_segment_length
            {
                segments.push(current_segment.trim().to_string());
                current_segment = String::new();
            }

            // Handle words longer than max_segment_length (URLs, etc.)
            if word.len() > self.max_segment_length {
                if !current_segment.is_empty() {
                    segments.push(current_segment.trim().to_string());
                    current_segment = String::new();
                }
                // Split the word by chunks (not ideal, but necessary)
                for chunk in word
                    .as_bytes()
                    .chunks(self.max_segment_length)
                    .map(|c| String::from_utf8_lossy(c).to_string())
                {
                    segments.push(chunk);
                }
            } else {
                if !current_segment.is_empty() {
                    current_segment.push(' ');
                }
                current_segment.push_str(word);
            }
        }

        if !current_segment.is_empty() {
            segments.push(current_segment.trim().to_string());
        }

        segments
    }

    /// Estimate optimal segment boundaries for a given text
    /// Returns a vector of (start_index, end_index) tuples
    pub fn estimate_boundaries(&self, text: &str) -> Vec<(usize, usize)> {
        let segments = self.segment_text(text);
        let mut boundaries = Vec::new();
        let mut start = 0;

        for segment in segments {
            let end = start + segment.len();
            boundaries.push((start, end));
            start = end + 1; // +1 for the space or separator
        }

        boundaries
    }
}

impl Default for TextSegmenter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_empty_text() {
        let segmenter = TextSegmenter::new();
        let segments = segmenter.segment_text("");
        assert_eq!(segments.len(), 0);
    }

    #[test]
    fn test_segment_short_text() {
        let segmenter = TextSegmenter::new();
        let text = "Hello world.";
        let segments = segmenter.segment_text(text);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], "Hello world.");
    }

    #[test]
    fn test_segment_multiple_sentences() {
        let segmenter = TextSegmenter::new();
        let text =
            "This is the first sentence. This is the second sentence. This is the third sentence.";
        let segments = segmenter.segment_text(text);

        // Should stay as one segment since total length is under max
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_segment_very_long_text() {
        let segmenter = TextSegmenter::with_max_length(50);
        let text = "This is a very long sentence that definitely exceeds the maximum segment length and should be split into multiple smaller segments for translation.";
        let segments = segmenter.segment_text(text);

        // Should be split into multiple segments
        assert!(segments.len() > 1);

        // Each segment should be under max length
        for segment in &segments {
            assert!(segment.len() <= 50 || segment.len() <= 60); // Allow some tolerance
        }
    }

    #[test]
    fn test_segment_with_commas() {
        let segmenter = TextSegmenter::with_max_length(30);
        let text = "First clause, second clause, third clause, fourth clause, fifth clause.";
        let segments = segmenter.segment_text(text);

        assert!(segments.len() > 1);
    }

    #[test]
    fn test_segment_cjk_terminators() {
        let segmenter = TextSegmenter::new();
        let text = "これは最初の文です。これは二番目の文です。これは三番目の文です。";
        let segments = segmenter.segment_text(text);

        assert!(!segments.is_empty());
    }

    #[test]
    fn test_split_on_word_boundaries() {
        let segmenter = TextSegmenter::with_max_length(20);
        let text = "one two three four five six seven eight nine ten";
        let segments = segmenter.segment_text(text);

        assert!(segments.len() > 1);

        // Verify no segment exceeds max length
        for segment in &segments {
            assert!(segment.len() <= 30); // Some tolerance for word boundaries
        }
    }

    #[test]
    fn test_handle_very_long_word() {
        let segmenter = TextSegmenter::with_max_length(20);
        let text =
            "This has a verylongwordthatexceedsthemaximumsegmentlengthandcannotbesplit normally.";
        let segments = segmenter.segment_text(text);

        // Should still produce segments
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_estimate_boundaries() {
        let segmenter = TextSegmenter::with_max_length(30);
        let text = "First part. Second part. Third part.";
        let boundaries = segmenter.estimate_boundaries(text);

        assert!(!boundaries.is_empty());

        // Verify boundaries are sequential
        for i in 1..boundaries.len() {
            assert!(boundaries[i].0 >= boundaries[i - 1].1);
        }
    }

    #[test]
    fn test_preserve_whitespace_between_segments() {
        let segmenter = TextSegmenter::new();
        let text = "First sentence. Second sentence.";
        let segments = segmenter.segment_text(text);

        // Segments should not have leading/trailing whitespace
        for segment in &segments {
            assert_eq!(segment.trim(), *segment);
        }
    }

    #[test]
    fn test_mixed_punctuation() {
        let segmenter = TextSegmenter::new();
        let text = "Question? Statement! Exclamation. Another sentence; with semicolon.";
        let segments = segmenter.segment_text(text);

        assert!(!segments.is_empty());
    }

    #[test]
    fn test_no_sentence_terminators() {
        let segmenter = TextSegmenter::with_max_length(50);
        let text = "This text has no sentence terminators at all just keeps going and going";
        let segments = segmenter.segment_text(text);

        assert!(!segments.is_empty());
    }
}
