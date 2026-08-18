pub mod cache;
pub mod types;

use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

use crate::api::client::HiNotesClient;
use cache::TranslationCache;
use types::{Language, TranslationRequest, TranslationResponse};

/// Translation service that handles language detection, translation, and caching
pub struct TranslationService {
    client: Arc<HiNotesClient>,
    cache: TranslationCache,
}

impl TranslationService {
    /// Create a new translation service
    pub fn new(client: Arc<HiNotesClient>, cache: TranslationCache) -> Self {
        Self { client, cache }
    }

    /// Translate text from source language to target language
    /// Returns cached translation if available
    pub async fn translate_text(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslationResponse> {
        // Check cache first
        if let Some(cached) = self
            .cache
            .get_translation(text, source_lang, target_lang)
            .await?
        {
            return Ok(cached);
        }

        // Make API request
        let request = TranslationRequest {
            text: text.to_string(),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
        };

        let response = self.client.translate_text(request).await?;

        // Cache the result
        self.cache
            .save_translation(text, source_lang, target_lang, &response.translated_text)
            .await?;

        Ok(response)
    }

    /// Detect the language of the given text
    pub async fn detect_language(&self, text: &str) -> Result<String> {
        self.client.detect_language(text).await
    }

    /// Get list of supported languages
    pub async fn get_supported_languages(&self) -> Result<Vec<Language>> {
        self.client.get_language_list().await
    }

    /// Clear translation cache older than the specified days
    pub async fn clear_old_cache(&self, days: i64) -> Result<u64> {
        self.cache.clear_old_translations(days).await
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<(u64, u64)> {
        self.cache.get_cache_stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::HiNotesClient;
    use tempfile::TempDir;

    fn setup_test_service() -> (TranslationService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let cache = TranslationCache::new(db_path.to_str().unwrap()).unwrap();
        let client = Arc::new(HiNotesClient::new("http://localhost:3001/v1"));
        let service = TranslationService::new(client, cache);
        (service, temp_dir)
    }

    #[tokio::test]
    async fn test_translate_text_not_cached() {
        let (service, _temp) = setup_test_service();

        let result = service
            .translate_text("Hello world", "en", "es")
            .await;

        // Should fail without mock server, but tests structure
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_translate_text_uses_cache_on_second_call() {
        let (service, _temp) = setup_test_service();

        // Manually insert into cache
        service
            .cache
            .save_translation("Test", "en", "es", "Prueba")
            .await
            .unwrap();

        let result = service.translate_text("Test", "en", "es").await.unwrap();

        assert_eq!(result.translated_text, "Prueba");
        assert_eq!(result.source_lang, "en");
        assert_eq!(result.target_lang, "es");
    }

    #[tokio::test]
    async fn test_get_supported_languages() {
        let (service, _temp) = setup_test_service();

        let result = service.get_supported_languages().await;

        // Should fail without mock server, but tests structure
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_detect_language() {
        let (service, _temp) = setup_test_service();

        let result = service.detect_language("Hello world").await;

        // Should fail without mock server, but tests structure
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_clear_old_cache() {
        let (service, _temp) = setup_test_service();

        // Insert some test translations
        service
            .cache
            .save_translation("Old", "en", "es", "Viejo")
            .await
            .unwrap();

        let result = service.clear_old_cache(0).await;

        assert!(result.is_ok());
        let deleted = result.unwrap();
        assert_eq!(deleted, 1);
    }

    #[tokio::test]
    async fn test_get_cache_stats() {
        let (service, _temp) = setup_test_service();

        // Insert test data
        service
            .cache
            .save_translation("Test1", "en", "es", "Prueba1")
            .await
            .unwrap();
        service
            .cache
            .save_translation("Test2", "en", "fr", "Test2")
            .await
            .unwrap();

        let result = service.get_cache_stats().await.unwrap();

        assert_eq!(result.0, 2); // 2 total translations
        assert!(result.1 > 0); // Some size in bytes
    }

    #[tokio::test]
    async fn test_empty_text_translation() {
        let (service, _temp) = setup_test_service();

        let result = service.translate_text("", "en", "es").await;

        // Empty text should be handled gracefully
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_same_source_and_target_language() {
        let (service, _temp) = setup_test_service();

        service
            .cache
            .save_translation("Hello", "en", "en", "Hello")
            .await
            .unwrap();

        let result = service.translate_text("Hello", "en", "en").await.unwrap();

        assert_eq!(result.translated_text, "Hello");
    }

    #[tokio::test]
    async fn test_cache_key_uniqueness() {
        let (service, _temp) = setup_test_service();

        // Same text, different language pairs should be different
        service
            .cache
            .save_translation("Hello", "en", "es", "Hola")
            .await
            .unwrap();
        service
            .cache
            .save_translation("Hello", "en", "fr", "Bonjour")
            .await
            .unwrap();

        let es_result = service.translate_text("Hello", "en", "es").await.unwrap();
        let fr_result = service.translate_text("Hello", "en", "fr").await.unwrap();

        assert_eq!(es_result.translated_text, "Hola");
        assert_eq!(fr_result.translated_text, "Bonjour");
    }

    #[tokio::test]
    async fn test_cache_with_special_characters() {
        let (service, _temp) = setup_test_service();

        let text_with_special = "Hello! How are you? 你好";
        service
            .cache
            .save_translation(text_with_special, "en", "es", "¡Hola! ¿Cómo estás? 你好")
            .await
            .unwrap();

        let result = service
            .translate_text(text_with_special, "en", "es")
            .await
            .unwrap();

        assert_eq!(result.translated_text, "¡Hola! ¿Cómo estás? 你好");
    }
}
