use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Request to translate text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationRequest {
    pub text: String,
    #[serde(rename = "sourceLang")]
    pub source_lang: String,
    #[serde(rename = "targetLang")]
    pub target_lang: String,
}

/// Response from translation API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationResponse {
    #[serde(rename = "translatedText")]
    pub translated_text: String,
    #[serde(rename = "sourceLang")]
    pub source_lang: String,
    #[serde(rename = "targetLang")]
    pub target_lang: String,
    #[serde(rename = "detectedLang")]
    pub detected_lang: Option<String>,
    pub confidence: Option<f64>,
}

/// Language information from API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Language {
    pub code: String,
    pub name: String,
    #[serde(rename = "nativeName")]
    pub native_name: Option<String>,
}

/// Cached translation in database
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CachedTranslation {
    pub id: String,
    pub source_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub translated_text: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: i64,
}

/// Live translation session (for real-time translation)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LiveTranslationSession {
    pub id: String,
    pub source_lang: String,
    pub target_lang: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

/// Rate translation quality request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateTranslationRequest {
    #[serde(rename = "translationId")]
    pub translation_id: String,
    pub rating: i32, // 1-5 stars
    pub feedback: Option<String>,
}

impl TranslationResponse {
    /// Create from cached translation
    pub fn from_cache(cached: &CachedTranslation) -> Self {
        Self {
            translated_text: cached.translated_text.clone(),
            source_lang: cached.source_lang.clone(),
            target_lang: cached.target_lang.clone(),
            detected_lang: None,
            confidence: Some(1.0), // Cached translations are assumed correct
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_request_serialization() {
        let request = TranslationRequest {
            text: "Hello".to_string(),
            source_lang: "en".to_string(),
            target_lang: "es".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("sourceLang"));
        assert!(json.contains("targetLang"));
    }

    #[test]
    fn test_translation_response_deserialization() {
        let json = r#"{
            "translatedText": "Hola",
            "sourceLang": "en",
            "targetLang": "es",
            "detectedLang": "en",
            "confidence": 0.99
        }"#;

        let response: TranslationResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.translated_text, "Hola");
        assert_eq!(response.source_lang, "en");
        assert_eq!(response.target_lang, "es");
        assert_eq!(response.detected_lang, Some("en".to_string()));
        assert_eq!(response.confidence, Some(0.99));
    }

    #[test]
    fn test_language_deserialization() {
        let json = r#"{
            "code": "es",
            "name": "Spanish",
            "nativeName": "Español"
        }"#;

        let lang: Language = serde_json::from_str(json).unwrap();
        assert_eq!(lang.code, "es");
        assert_eq!(lang.name, "Spanish");
        assert_eq!(lang.native_name, Some("Español".to_string()));
    }

    #[test]
    fn test_translation_response_from_cache() {
        let cached = CachedTranslation {
            id: "123".to_string(),
            source_text: "Hello".to_string(),
            source_lang: "en".to_string(),
            target_lang: "es".to_string(),
            translated_text: "Hola".to_string(),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 5,
        };

        let response = TranslationResponse::from_cache(&cached);
        assert_eq!(response.translated_text, "Hola");
        assert_eq!(response.source_lang, "en");
        assert_eq!(response.target_lang, "es");
        assert_eq!(response.confidence, Some(1.0));
    }

    #[test]
    fn test_rate_translation_request_serialization() {
        let request = RateTranslationRequest {
            translation_id: "trans_123".to_string(),
            rating: 5,
            feedback: Some("Excellent translation".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("translationId"));
        assert!(json.contains("rating"));
    }
}
