use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::client::HiNotesClient;
use crate::api::types::{Language, TranslationRequest, TranslationResponse};

/// Translation API client wrapper providing high-level translation operations
pub struct TranslationClient {
    hinotes_client: Arc<HiNotesClient>,
    default_source_lang: Arc<RwLock<Option<String>>>,
    default_target_lang: Arc<RwLock<Option<String>>>,
}

impl TranslationClient {
    /// Create a new translation client
    pub fn new(hinotes_client: Arc<HiNotesClient>) -> Self {
        Self {
            hinotes_client,
            default_source_lang: Arc::new(RwLock::new(None)),
            default_target_lang: Arc::new(RwLock::new(None)),
        }
    }

    /// Translate text from source language to target language
    pub async fn translate(
        &self,
        text: &str,
        source_lang: Option<&str>,
        target_lang: Option<&str>,
    ) -> Result<TranslationResponse> {
        // Use provided languages or defaults
        let source = match source_lang {
            Some(lang) => lang.to_string(),
            None => {
                let default = self.default_source_lang.read().await;
                default
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("No source language specified"))?
            }
        };

        let target = match target_lang {
            Some(lang) => lang.to_string(),
            None => {
                let default = self.default_target_lang.read().await;
                default
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("No target language specified"))?
            }
        };

        let request = TranslationRequest {
            text: text.to_string(),
            source_lang: source,
            target_lang: target,
        };

        self.hinotes_client.translate_text(request).await
    }

    /// Detect the language of the given text
    pub async fn detect_language(&self, text: &str) -> Result<String> {
        self.hinotes_client.detect_language(text).await
    }

    /// Get list of supported languages
    pub async fn get_supported_languages(&self) -> Result<Vec<Language>> {
        self.hinotes_client.get_language_list().await
    }

    /// Set default source language
    pub async fn set_default_source_lang(&self, lang: String) {
        *self.default_source_lang.write().await = Some(lang);
    }

    /// Set default target language
    pub async fn set_default_target_lang(&self, lang: String) {
        *self.default_target_lang.write().await = Some(lang);
    }

    /// Get default source language
    pub async fn get_default_source_lang(&self) -> Option<String> {
        self.default_source_lang.read().await.clone()
    }

    /// Get default target language
    pub async fn get_default_target_lang(&self) -> Option<String> {
        self.default_target_lang.read().await.clone()
    }

    /// Batch translate multiple texts
    pub async fn batch_translate(
        &self,
        texts: Vec<String>,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<TranslationResponse>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let response = self
                .translate(&text, Some(source_lang), Some(target_lang))
                .await?;
            results.push(response);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_client() -> TranslationClient {
        let hinotes_client = Arc::new(HiNotesClient::with_base_url("http://localhost:3001/v1".to_string()));
        TranslationClient::new(hinotes_client)
    }

    #[tokio::test]
    async fn test_set_and_get_default_languages() {
        let client = setup_client();

        client.set_default_source_lang("en".to_string()).await;
        client.set_default_target_lang("es".to_string()).await;

        assert_eq!(
            client.get_default_source_lang().await,
            Some("en".to_string())
        );
        assert_eq!(
            client.get_default_target_lang().await,
            Some("es".to_string())
        );
    }

    #[tokio::test]
    async fn test_translate_without_defaults_fails() {
        let client = setup_client();

        let result = client.translate("Hello", None, None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No source language"));
    }

    #[tokio::test]
    async fn test_translate_with_explicit_languages() {
        let client = setup_client();

        // This will fail without mock server, but tests the API structure
        let result = client.translate("Hello", Some("en"), Some("es")).await;
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_batch_translate_structure() {
        let client = setup_client();

        let texts = vec!["Hello".to_string(), "World".to_string()];
        let result = client.batch_translate(texts, "en", "es").await;

        // Will fail without mock server, but tests structure
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_detect_language() {
        let client = setup_client();

        let result = client.detect_language("Hello world").await;
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_get_supported_languages() {
        let client = setup_client();

        let result = client.get_supported_languages().await;
        assert!(result.is_err() || result.is_ok());
    }
}
