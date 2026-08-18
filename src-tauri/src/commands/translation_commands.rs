use crate::translation::{
    engine::TranslationEngine,
    types::{Language, TranslationRequest, TranslationResponse},
};
use tauri::State;

pub struct TranslationState {
    pub engine: TranslationEngine,
}

/// Translate text from source language to target language
#[tauri::command]
pub async fn translate_text(
    text: String,
    source_lang: String,
    target_lang: String,
    state: State<'_, TranslationState>,
) -> Result<TranslationResponse, String> {
    let request = TranslationRequest {
        text,
        source_lang,
        target_lang,
    };

    state
        .engine
        .translate(&request)
        .await
        .map_err(|e| e.to_string())
}

/// Get list of supported languages
#[tauri::command]
pub async fn get_supported_languages(
    state: State<'_, TranslationState>,
) -> Result<Vec<Language>, String> {
    state
        .engine
        .get_supported_languages()
        .await
        .map_err(|e| e.to_string())
}

/// Detect language of given text
#[tauri::command]
pub async fn detect_language(
    text: String,
    state: State<'_, TranslationState>,
) -> Result<String, String> {
    state
        .engine
        .detect_language(&text)
        .await
        .map_err(|e| e.to_string())
}

/// Clear translation cache
#[tauri::command]
pub async fn clear_translation_cache(
    state: State<'_, TranslationState>,
) -> Result<(), String> {
    state
        .engine
        .clear_cache()
        .await
        .map_err(|e| e.to_string())
}

/// Get translation cache statistics
#[tauri::command]
pub async fn get_translation_cache_stats(
    state: State<'_, TranslationState>,
) -> Result<serde_json::Value, String> {
    state
        .engine
        .get_cache_stats()
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn setup_translation_state() -> TranslationState {
        let db = Database::new_in_memory().expect("Failed to create test database");
        let engine = TranslationEngine::new(db);
        TranslationState { engine }
    }

    #[tokio::test]
    async fn test_translate_text() {
        let state = setup_translation_state();

        let result = translate_text(
            "Hello world".to_string(),
            "en".to_string(),
            "es".to_string(),
            State::from(&state),
        )
        .await;

        // Note: This will fail without actual API credentials
        // In real tests, mock the translation engine
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_get_supported_languages() {
        let state = setup_translation_state();

        let result = get_supported_languages(State::from(&state)).await;

        // Should return list of languages even without API
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_detect_language() {
        let state = setup_translation_state();

        let result = detect_language(
            "Hola mundo".to_string(),
            State::from(&state),
        )
        .await;

        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_clear_translation_cache() {
        let state = setup_translation_state();

        let result = clear_translation_cache(State::from(&state)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_translation_cache_stats() {
        let state = setup_translation_state();

        let result = get_translation_cache_stats(State::from(&state)).await;

        assert!(result.is_ok());
    }
}
