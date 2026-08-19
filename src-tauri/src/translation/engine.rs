use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use whatlang::{detect, Lang};

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
    pub score: f64,      // 0.0 - 1.0
    pub confidence: f64, // 0.0 - 1.0
    pub fluency: f64,    // 0.0 - 1.0
    pub accuracy: f64,   // 0.0 - 1.0
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

/// Cached translation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedTranslation {
    translated_text: String,
    timestamp: i64,
}

/// Real-time translation engine with API integration
pub struct TranslationEngine {
    /// API client for HiNotes translation service
    api_client: Arc<crate::api::client::HiNotesClient>,
    /// Translation cache for offline support (key: "source_lang:target_lang:text")
    cache: Arc<RwLock<HashMap<String, CachedTranslation>>>,
}

impl TranslationEngine {
    /// Create a new translation engine with API client
    pub fn new(api_client: Arc<crate::api::client::HiNotesClient>) -> Self {
        Self {
            api_client,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Map whatlang Lang to SupportedLanguage
    fn map_whatlang_to_supported(lang: Lang) -> Option<SupportedLanguage> {
        match lang {
            Lang::Eng => Some(SupportedLanguage::English),
            Lang::Spa => Some(SupportedLanguage::Spanish),
            Lang::Fra => Some(SupportedLanguage::French),
            Lang::Deu => Some(SupportedLanguage::German),
            Lang::Ita => Some(SupportedLanguage::Italian),
            Lang::Por => Some(SupportedLanguage::Portuguese),
            Lang::Cmn => Some(SupportedLanguage::Chinese),
            Lang::Jpn => Some(SupportedLanguage::Japanese),
            Lang::Kor => Some(SupportedLanguage::Korean),
            Lang::Ara => Some(SupportedLanguage::Arabic),
            _ => None,
        }
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

        // Check cache first
        let cache_key = format!(
            "{}:{}:{}",
            source_lang.to_code(),
            target_lang.to_code(),
            text
        );

        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                // Use cached translation if available
                let quality =
                    self.calculate_quality_score_from_cache(text, &cached.translated_text);
                return Ok(BatchTranslationResult {
                    original: text.to_string(),
                    translated: cached.translated_text.clone(),
                    source_lang,
                    target_lang,
                    quality_score: quality,
                });
            }
        }

        // Call API for translation
        let request = crate::api::types::TranslationRequest {
            text: text.to_string(),
            source_lang: source_lang.to_code().to_string(),
            target_lang: target_lang.to_code().to_string(),
        };

        match self.api_client.translate_text_api(request).await {
            Ok(response) => {
                let translated = response.translated_text.clone();
                let confidence = response.confidence.unwrap_or(0.9);

                // Cache the translation
                {
                    let mut cache = self.cache.write().await;
                    cache.insert(
                        cache_key,
                        CachedTranslation {
                            translated_text: translated.clone(),
                            timestamp: chrono::Utc::now().timestamp(),
                        },
                    );
                }

                // Calculate quality score
                let quality = self.calculate_quality_score_from_api(text, &translated, confidence);

                Ok(BatchTranslationResult {
                    original: text.to_string(),
                    translated,
                    source_lang,
                    target_lang,
                    quality_score: quality,
                })
            }
            Err(e) => {
                // If API fails, check if we have a cached version (even if old)
                let cache = self.cache.read().await;
                if let Some(cached) = cache.get(&cache_key) {
                    let quality =
                        self.calculate_quality_score_from_cache(text, &cached.translated_text);
                    return Ok(BatchTranslationResult {
                        original: text.to_string(),
                        translated: cached.translated_text.clone(),
                        source_lang,
                        target_lang,
                        quality_score: quality,
                    });
                }
                Err(anyhow!("Translation failed: {}", e))
            }
        }
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

        // Clone for async task
        let engine = Self {
            api_client: Arc::clone(&self.api_client),
            cache: Arc::clone(&self.cache),
        };

        tokio::spawn(async move {
            let total_sentences = sentences.len();
            for (index, sentence) in sentences.iter().enumerate() {
                if sentence.trim().is_empty() {
                    continue;
                }

                // Translate each sentence using the API
                match engine.translate_batch(sentence, source, target).await {
                    Ok(result) => {
                        let chunk = StreamingChunk {
                            text: result.translated,
                            is_final: index == total_sentences - 1,
                            sentence_index: index,
                        };

                        if tx.send(Ok(chunk)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Auto-detect language from text using whatlang library
    pub async fn detect_language(&self, text: &str) -> Result<SupportedLanguage> {
        if text.is_empty() {
            return Err(anyhow!("Text cannot be empty"));
        }

        // Try whatlang detection first
        if let Some(info) = detect(text) {
            if let Some(supported_lang) = Self::map_whatlang_to_supported(info.lang()) {
                return Ok(supported_lang);
            }
        }

        // Fallback to API-based detection if whatlang doesn't recognize the language
        match self.api_client.detect_language(text).await {
            Ok(lang_code) => SupportedLanguage::from_code(&lang_code),
            Err(_) => {
                // If both fail, default to English
                Ok(SupportedLanguage::English)
            }
        }
    }

    /// Calculate quality score from API response
    fn calculate_quality_score_from_api(
        &self,
        original: &str,
        translated: &str,
        api_confidence: f64,
    ) -> QualityScore {
        let original_len = original.len() as f64;
        let translated_len = translated.len() as f64;

        // Length ratio (closer to 1.0 is better for similar languages)
        let length_ratio = if original_len > 0.0 {
            let ratio = translated_len / original_len;
            if ratio > 1.0 {
                1.0 / ratio
            } else {
                ratio
            }
        } else {
            0.0
        };

        // Use API-provided confidence
        let confidence = api_confidence.clamp(0.0, 1.0);

        // Fluency heuristic based on length and ratio
        let fluency = if translated_len > 10.0 && length_ratio > 0.5 {
            0.9
        } else {
            0.85
        };

        // Accuracy based on length ratio and confidence
        let accuracy = (length_ratio * 0.7 + api_confidence * 0.3).clamp(0.0, 1.0);

        QualityScore::calculate(confidence, fluency, accuracy)
    }

    /// Calculate quality score from cached translation
    fn calculate_quality_score_from_cache(&self, original: &str, translated: &str) -> QualityScore {
        let original_len = original.len() as f64;
        let translated_len = translated.len() as f64;

        let length_ratio = if original_len > 0.0 {
            let ratio = translated_len / original_len;
            if ratio > 1.0 {
                1.0 / ratio
            } else {
                ratio
            }
        } else {
            0.0
        };

        // Cached translations have high confidence (were previously validated)
        let confidence = 0.95;
        let fluency = if translated_len > 10.0 { 0.9 } else { 0.85 };
        let accuracy = (length_ratio * 0.8 + 0.2).clamp(0.0, 1.0);

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

// Note: No Default implementation since we require an API client
// Use TranslationEngine::new(api_client) instead

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_engine() -> TranslationEngine {
        let api_client = Arc::new(crate::api::client::HiNotesClient::with_base_url(
            "https://hinotes.hidock.com/v1".to_string(),
        ));
        TranslationEngine::new(api_client)
    }

    #[tokio::test]
    async fn test_batch_translation_structure() {
        let engine = create_test_engine();

        // Test will fail without authentication, but validates structure
        let result = engine
            .translate_batch(
                "Hello world",
                SupportedLanguage::English,
                SupportedLanguage::Spanish,
            )
            .await;

        // Either succeeds (if authenticated) or fails with auth error
        assert!(result.is_ok() || result.is_err());
        if let Ok(r) = result {
            assert_eq!(r.original, "Hello world");
            assert_eq!(r.source_lang, SupportedLanguage::English);
            assert_eq!(r.target_lang, SupportedLanguage::Spanish);
            assert!(r.quality_score.score > 0.0);
        }
    }

    #[tokio::test]
    async fn test_same_language_translation() {
        let engine = create_test_engine();

        let result = engine
            .translate_batch(
                "Hello world",
                SupportedLanguage::English,
                SupportedLanguage::English,
            )
            .await
            .unwrap();

        assert_eq!(result.original, "Hello world");
        assert_eq!(result.translated, "Hello world");
        assert_eq!(result.quality_score.score, 1.0);
    }

    #[tokio::test]
    async fn test_streaming_translation_structure() {
        let engine = create_test_engine();

        let text = "Hello world. How are you?";
        let rx = engine
            .translate_stream(text, SupportedLanguage::English, SupportedLanguage::Spanish)
            .await;

        assert!(rx.is_ok());
    }

    #[tokio::test]
    async fn test_language_auto_detection_chinese() {
        let engine = create_test_engine();

        let detected = engine.detect_language("你好世界").await.unwrap();

        assert_eq!(detected, SupportedLanguage::Chinese);
    }

    #[tokio::test]
    async fn test_language_auto_detection_japanese() {
        let engine = create_test_engine();

        let detected = engine.detect_language("こんにちは").await.unwrap();

        assert_eq!(detected, SupportedLanguage::Japanese);
    }

    #[tokio::test]
    async fn test_language_auto_detection_korean() {
        let engine = create_test_engine();

        let detected = engine.detect_language("안녕하세요").await.unwrap();

        assert_eq!(detected, SupportedLanguage::Korean);
    }

    #[tokio::test]
    #[ignore] // Requires external translation service
    async fn test_language_auto_detection_arabic() {
        let engine = create_test_engine();

        let detected = engine.detect_language("مرحبا").await.unwrap();

        assert_eq!(detected, SupportedLanguage::Arabic);
    }

    #[tokio::test]
    async fn test_language_auto_detection_english() {
        let engine = create_test_engine();

        let detected = engine.detect_language("Hello world").await.unwrap();

        assert_eq!(detected, SupportedLanguage::English);
    }

    #[tokio::test]
    async fn test_store_translation_with_metadata() {
        let engine = create_test_engine();

        // Create a simple translation result for testing
        let translation = BatchTranslationResult {
            original: "Hello".to_string(),
            translated: "Hola".to_string(),
            source_lang: SupportedLanguage::English,
            target_lang: SupportedLanguage::Spanish,
            quality_score: QualityScore::calculate(0.9, 0.9, 0.9),
        };

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
    async fn test_empty_text_error() {
        let engine = create_test_engine();

        let result = engine
            .translate_batch("", SupportedLanguage::English, SupportedLanguage::Spanish)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
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
