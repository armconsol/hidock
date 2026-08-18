use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Supported languages for translation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedLanguage {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "it")]
    Italian,
    #[serde(rename = "pt")]
    Portuguese,
    #[serde(rename = "zh")]
    Chinese,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "ar")]
    Arabic,
}

impl SupportedLanguage {
    /// Convert to language code string
    pub fn to_code(&self) -> &'static str {
        match self {
            SupportedLanguage::English => "en",
            SupportedLanguage::Spanish => "es",
            SupportedLanguage::French => "fr",
            SupportedLanguage::German => "de",
            SupportedLanguage::Italian => "it",
            SupportedLanguage::Portuguese => "pt",
            SupportedLanguage::Chinese => "zh",
            SupportedLanguage::Japanese => "ja",
            SupportedLanguage::Korean => "ko",
            SupportedLanguage::Arabic => "ar",
        }
    }

    /// Parse from language code string
    pub fn from_code(code: &str) -> Result<Self> {
        match code.to_lowercase().as_str() {
            "en" => Ok(SupportedLanguage::English),
            "es" => Ok(SupportedLanguage::Spanish),
            "fr" => Ok(SupportedLanguage::French),
            "de" => Ok(SupportedLanguage::German),
            "it" => Ok(SupportedLanguage::Italian),
            "pt" => Ok(SupportedLanguage::Portuguese),
            "zh" => Ok(SupportedLanguage::Chinese),
            "ja" => Ok(SupportedLanguage::Japanese),
            "ko" => Ok(SupportedLanguage::Korean),
            "ar" => Ok(SupportedLanguage::Arabic),
            _ => Err(anyhow!("Unsupported language code: {}", code)),
        }
    }

    /// Get all supported languages
    pub fn all() -> Vec<Self> {
        vec![
            SupportedLanguage::English,
            SupportedLanguage::Spanish,
            SupportedLanguage::French,
            SupportedLanguage::German,
            SupportedLanguage::Italian,
            SupportedLanguage::Portuguese,
            SupportedLanguage::Chinese,
            SupportedLanguage::Japanese,
            SupportedLanguage::Korean,
            SupportedLanguage::Arabic,
        ]
    }
}

/// Translation quality score
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityScore {
    pub score: f64,           // 0.0 - 1.0
    pub confidence: f64,      // 0.0 - 1.0
    pub fluency: f64,         // 0.0 - 1.0
    pub accuracy: f64,        // 0.0 - 1.0
}

impl QualityScore {
    /// Calculate overall quality score
    pub fn calculate(confidence: f64, fluency: f64, accuracy: f64) -> Self {
        let score = (confidence * 0.3 + fluency * 0.3 + accuracy * 0.4).clamp(0.0, 1.0);
        Self {
            score,
            confidence: confidence.clamp(0.0, 1.0),
            fluency: fluency.clamp(0.0, 1.0),
            accuracy: accuracy.clamp(0.0, 1.0),
        }
    }
}

/// Batch translation result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchTranslationResult {
    pub original: String,
    pub translated: String,
    pub source_lang: SupportedLanguage,
    pub target_lang: SupportedLanguage,
    pub quality_score: QualityScore,
}

/// Streaming translation chunk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamingChunk {
    pub text: String,
    pub is_final: bool,
    pub sentence_index: usize,
}

/// Translation with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationWithMetadata {
    pub translation: BatchTranslationResult,
    pub metadata: HashMap<String, String>,
}

/// Real-time translation engine
pub struct TranslationEngine {
    /// Mock translation database (in production, this would call an API)
    mock_translations: HashMap<String, String>,
}

impl TranslationEngine {
    /// Create a new translation engine
    pub fn new() -> Self {
        Self {
            mock_translations: Self::init_mock_translations(),
        }
    }

    /// Initialize mock translations for testing
    fn init_mock_translations() -> HashMap<String, String> {
        let mut map = HashMap::new();

        // English to Spanish
        map.insert("en:es:Hello world".to_string(), "Hola mundo".to_string());
        map.insert("en:es:How are you?".to_string(), "¿Cómo estás?".to_string());
        map.insert("en:es:Good morning".to_string(), "Buenos días".to_string());

        // English to French
        map.insert("en:fr:Hello world".to_string(), "Bonjour le monde".to_string());
        map.insert("en:fr:How are you?".to_string(), "Comment allez-vous?".to_string());

        // English to German
        map.insert("en:de:Hello world".to_string(), "Hallo Welt".to_string());

        // English to Chinese
        map.insert("en:zh:Hello world".to_string(), "你好世界".to_string());

        // English to Japanese
        map.insert("en:ja:Hello world".to_string(), "こんにちは世界".to_string());

        map
    }

    /// Translate a batch of text
    pub async fn translate_batch(
        &self,
        text: &str,
        source_lang: SupportedLanguage,
        target_lang: SupportedLanguage,
    ) -> Result<BatchTranslationResult> {
        if text.is_empty() {
            return Err(anyhow!("Text cannot be empty"));
        }

        // Check if source and target are the same
        if source_lang == target_lang {
            return Ok(BatchTranslationResult {
                original: text.to_string(),
                translated: text.to_string(),
                source_lang,
                target_lang,
                quality_score: QualityScore::calculate(1.0, 1.0, 1.0),
            });
        }

        // Simulate translation
        let key = format!("{}:{}:{}", source_lang.to_code(), target_lang.to_code(), text);
        let translated = self.mock_translations
            .get(&key)
            .cloned()
            .unwrap_or_else(|| format!("[{}->{}] {}", source_lang.to_code(), target_lang.to_code(), text));

        // Calculate quality score based on translation length and complexity
        let quality = self.calculate_quality_score(text, &translated);

        Ok(BatchTranslationResult {
            original: text.to_string(),
            translated,
            source_lang,
            target_lang,
            quality_score: quality,
        })
    }

    /// Translate streaming text (sentence by sentence)
    pub async fn translate_stream(
        &self,
        text: &str,
        source_lang: SupportedLanguage,
        target_lang: SupportedLanguage,
    ) -> Result<mpsc::Receiver<Result<StreamingChunk>>> {
        let (tx, rx) = mpsc::channel(100);

        // Split text into sentences
        let sentences = self.split_into_sentences(text);
        let source = source_lang;
        let target = target_lang;

        // Clone data for async task
        let mock_translations = self.mock_translations.clone();

        tokio::spawn(async move {
            for (index, sentence) in sentences.iter().enumerate() {
                if sentence.trim().is_empty() {
                    continue;
                }

                // Translate each sentence
                let key = format!("{}:{}:{}", source.to_code(), target.to_code(), sentence);
                let translated = mock_translations
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| format!("[{}->{}] {}", source.to_code(), target.to_code(), sentence));

                let chunk = StreamingChunk {
                    text: translated,
                    is_final: index == sentences.len() - 1,
                    sentence_index: index,
                };

                if tx.send(Ok(chunk)).await.is_err() {
                    break;
                }
            }
        });

        Ok(rx)
    }

    /// Auto-detect language from text
    pub async fn detect_language(&self, text: &str) -> Result<SupportedLanguage> {
        if text.is_empty() {
            return Err(anyhow!("Text cannot be empty"));
        }

        // Simple heuristic-based language detection
        // In production, this would use a proper language detection library

        // Check for common patterns
        if text.chars().any(|c| '\u{4E00}' <= c && c <= '\u{9FFF}') {
            return Ok(SupportedLanguage::Chinese);
        }

        if text.chars().any(|c| '\u{3040}' <= c && c <= '\u{309F}' || '\u{30A0}' <= c && c <= '\u{30FF}') {
            return Ok(SupportedLanguage::Japanese);
        }

        if text.chars().any(|c| '\u{AC00}' <= c && c <= '\u{D7AF}') {
            return Ok(SupportedLanguage::Korean);
        }

        if text.chars().any(|c| '\u{0600}' <= c && c <= '\u{06FF}') {
            return Ok(SupportedLanguage::Arabic);
        }

        // Default to English for Latin script
        Ok(SupportedLanguage::English)
    }

    /// Calculate quality score for a translation
    fn calculate_quality_score(&self, original: &str, translated: &str) -> QualityScore {
        // Simple quality scoring based on length ratio and character diversity
        let original_len = original.len() as f64;
        let translated_len = translated.len() as f64;

        // Length ratio (closer to 1.0 is better, assuming similar languages)
        let length_ratio = if original_len > 0.0 {
            (translated_len / original_len).min(original_len / translated_len)
        } else {
            0.0
        };

        // Confidence based on whether it's a mock translation
        let confidence = if translated.starts_with('[') && translated.contains("->") {
            0.5 // Mock translation
        } else {
            0.95 // Real translation from database
        };

        // Fluency based on length (longer texts might be more complex)
        let fluency = if translated_len > 10.0 { 0.9 } else { 0.85 };

        // Accuracy approximation
        let accuracy = length_ratio * 0.8 + 0.2;

        QualityScore::calculate(confidence, fluency, accuracy)
    }

    /// Split text into sentences for streaming translation
    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();

        for ch in text.chars() {
            current.push(ch);

            // Simple sentence boundary detection
            if ch == '.' || ch == '!' || ch == '?' {
                if !current.trim().is_empty() {
                    sentences.push(current.trim().to_string());
                    current.clear();
                }
            }
        }

        // Add remaining text
        if !current.trim().is_empty() {
            sentences.push(current.trim().to_string());
        }

        sentences
    }

    /// Store translation with metadata
    pub async fn store_with_metadata(
        &self,
        translation: BatchTranslationResult,
        metadata: HashMap<String, String>,
    ) -> Result<TranslationWithMetadata> {
        // In production, this would save to database
        Ok(TranslationWithMetadata {
            translation,
            metadata,
        })
    }

    /// Translate multiple languages in parallel
    pub async fn translate_multiple(
        &self,
        text: &str,
        source_lang: SupportedLanguage,
        target_langs: Vec<SupportedLanguage>,
    ) -> Result<Vec<BatchTranslationResult>> {
        let mut results = Vec::new();

        for target_lang in target_langs {
            let result = self.translate_batch(text, source_lang, target_lang).await?;
            results.push(result);
        }

        Ok(results)
    }
}

impl Default for TranslationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_translation_english_to_spanish() {
        let engine = TranslationEngine::new();

        let result = engine
            .translate_batch("Hello world", SupportedLanguage::English, SupportedLanguage::Spanish)
            .await
            .unwrap();

        assert_eq!(result.original, "Hello world");
        assert_eq!(result.translated, "Hola mundo");
        assert_eq!(result.source_lang, SupportedLanguage::English);
        assert_eq!(result.target_lang, SupportedLanguage::Spanish);
        assert!(result.quality_score.score > 0.0);
    }

    #[tokio::test]
    async fn test_batch_translation_english_to_french() {
        let engine = TranslationEngine::new();

        let result = engine
            .translate_batch("Hello world", SupportedLanguage::English, SupportedLanguage::French)
            .await
            .unwrap();

        assert_eq!(result.translated, "Bonjour le monde");
        assert_eq!(result.target_lang, SupportedLanguage::French);
    }

    #[tokio::test]
    async fn test_batch_translation_multiple_languages() {
        let engine = TranslationEngine::new();

        let target_langs = vec![
            SupportedLanguage::Spanish,
            SupportedLanguage::French,
            SupportedLanguage::German,
        ];

        let results = engine
            .translate_multiple("Hello world", SupportedLanguage::English, target_langs)
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].translated, "Hola mundo");
        assert_eq!(results[1].translated, "Bonjour le monde");
        assert_eq!(results[2].translated, "Hallo Welt");
    }

    #[tokio::test]
    async fn test_streaming_translation() {
        let engine = TranslationEngine::new();

        let text = "Hello world. How are you?";
        let mut rx = engine
            .translate_stream(text, SupportedLanguage::English, SupportedLanguage::Spanish)
            .await
            .unwrap();

        let mut chunks = Vec::new();
        while let Some(result) = rx.recv().await {
            chunks.push(result.unwrap());
        }

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].sentence_index, 0);
        assert!(chunks.last().unwrap().is_final);
    }

    #[tokio::test]
    async fn test_language_auto_detection_chinese() {
        let engine = TranslationEngine::new();

        let detected = engine.detect_language("你好世界").await.unwrap();

        assert_eq!(detected, SupportedLanguage::Chinese);
    }

    #[tokio::test]
    async fn test_language_auto_detection_japanese() {
        let engine = TranslationEngine::new();

        let detected = engine.detect_language("こんにちは").await.unwrap();

        assert_eq!(detected, SupportedLanguage::Japanese);
    }

    #[tokio::test]
    async fn test_language_auto_detection_korean() {
        let engine = TranslationEngine::new();

        let detected = engine.detect_language("안녕하세요").await.unwrap();

        assert_eq!(detected, SupportedLanguage::Korean);
    }

    #[tokio::test]
    async fn test_language_auto_detection_arabic() {
        let engine = TranslationEngine::new();

        let detected = engine.detect_language("مرحبا").await.unwrap();

        assert_eq!(detected, SupportedLanguage::Arabic);
    }

    #[tokio::test]
    async fn test_language_auto_detection_english_default() {
        let engine = TranslationEngine::new();

        let detected = engine.detect_language("Hello world").await.unwrap();

        assert_eq!(detected, SupportedLanguage::English);
    }

    #[tokio::test]
    async fn test_quality_scoring() {
        let engine = TranslationEngine::new();

        let result = engine
            .translate_batch("Hello world", SupportedLanguage::English, SupportedLanguage::Spanish)
            .await
            .unwrap();

        let score = &result.quality_score;
        assert!(score.score >= 0.0 && score.score <= 1.0);
        assert!(score.confidence >= 0.0 && score.confidence <= 1.0);
        assert!(score.fluency >= 0.0 && score.fluency <= 1.0);
        assert!(score.accuracy >= 0.0 && score.accuracy <= 1.0);
    }

    #[tokio::test]
    async fn test_store_translation_with_metadata() {
        let engine = TranslationEngine::new();

        let translation = engine
            .translate_batch("Hello world", SupportedLanguage::English, SupportedLanguage::Spanish)
            .await
            .unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("note_id".to_string(), "note_123".to_string());
        metadata.insert("user_id".to_string(), "user_456".to_string());

        let result = engine
            .store_with_metadata(translation.clone(), metadata.clone())
            .await
            .unwrap();

        assert_eq!(result.translation, translation);
        assert_eq!(result.metadata, metadata);
    }

    #[tokio::test]
    async fn test_partial_sentence_translation() {
        let engine = TranslationEngine::new();

        let text = "Hello world.";
        let mut rx = engine
            .translate_stream(text, SupportedLanguage::English, SupportedLanguage::Spanish)
            .await
            .unwrap();

        let chunk = rx.recv().await.unwrap().unwrap();

        assert_eq!(chunk.sentence_index, 0);
        assert!(chunk.is_final);
        assert!(!chunk.text.is_empty());
    }

    #[tokio::test]
    async fn test_empty_text_error() {
        let engine = TranslationEngine::new();

        let result = engine
            .translate_batch("", SupportedLanguage::English, SupportedLanguage::Spanish)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[tokio::test]
    async fn test_same_source_and_target_language() {
        let engine = TranslationEngine::new();

        let result = engine
            .translate_batch("Hello world", SupportedLanguage::English, SupportedLanguage::English)
            .await
            .unwrap();

        assert_eq!(result.original, result.translated);
        assert_eq!(result.quality_score.score, 1.0);
    }

    #[tokio::test]
    async fn test_supported_language_from_code() {
        assert_eq!(
            SupportedLanguage::from_code("en").unwrap(),
            SupportedLanguage::English
        );
        assert_eq!(
            SupportedLanguage::from_code("ES").unwrap(),
            SupportedLanguage::Spanish
        );
        assert!(SupportedLanguage::from_code("xx").is_err());
    }

    #[tokio::test]
    async fn test_supported_language_to_code() {
        assert_eq!(SupportedLanguage::English.to_code(), "en");
        assert_eq!(SupportedLanguage::Spanish.to_code(), "es");
        assert_eq!(SupportedLanguage::Chinese.to_code(), "zh");
    }

    #[tokio::test]
    async fn test_all_supported_languages_count() {
        let languages = SupportedLanguage::all();
        assert_eq!(languages.len(), 10);
    }

    #[test]
    fn test_quality_score_calculation() {
        let score = QualityScore::calculate(0.9, 0.85, 0.95);

        assert!(score.score >= 0.0 && score.score <= 1.0);
        assert_eq!(score.confidence, 0.9);
        assert_eq!(score.fluency, 0.85);
        assert_eq!(score.accuracy, 0.95);
    }

    #[test]
    fn test_quality_score_clamping() {
        let score = QualityScore::calculate(1.5, -0.5, 0.5);

        assert_eq!(score.confidence, 1.0);
        assert_eq!(score.fluency, 0.0);
        assert_eq!(score.accuracy, 0.5);
    }
}
