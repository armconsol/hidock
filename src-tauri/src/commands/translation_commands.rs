use crate::api::types::{Language, TranslationResponse};
use crate::translation::cache::TranslationCache;
use crate::translation::live_session::{
    LiveSessionManager, LiveTranslationSession, TranslationSegment,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex as TokioMutex;

/// State for translation operations
pub struct TranslationState {
    pub cache: Arc<TokioMutex<TranslationCache>>,
    pub session_manager: Arc<TokioMutex<LiveSessionManager>>,
}

/// Request to translate text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateTextRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_translations: u64,
    pub cache_size_bytes: u64,
}

// ===== TRANSLATION COMMANDS =====

/// Translate text from source language to target language
#[tauri::command]
pub async fn translate_text(
    text: String,
    source_lang: String,
    target_lang: String,
    cache_state: State<'_, Arc<TokioMutex<TranslationCache>>>,
) -> Result<TranslationResponse, String> {
    // Check cache first
    let cache = cache_state.lock().await;
    if let Some(cached) = cache
        .get_translation(&text, &source_lang, &target_lang)
        .await
        .map_err(|e| format!("Cache lookup error: {}", e))?
    {
        return Ok(cached);
    }
    drop(cache);

    // TODO: In production, use HiNotes API client here
    // For now, return an error indicating API client is needed
    Err("Translation API client not yet integrated. Use cache or implement API call.".to_string())
}

/// Get list of supported languages for translation
///
/// Note: This returns a hardcoded list of common languages.
/// In production, this could fetch from HiNotes API via get_language_list()
/// if the API client is available and authenticated.
#[tauri::command]
pub async fn get_supported_languages() -> Result<Vec<Language>, String> {
    // Return supported languages
    // TODO: Optionally fetch from API if available
    Ok(vec![
        Language {
            code: "en".to_string(),
            name: "English".to_string(),
            native_name: Some("English".to_string()),
        },
        Language {
            code: "es".to_string(),
            name: "Spanish".to_string(),
            native_name: Some("Español".to_string()),
        },
        Language {
            code: "fr".to_string(),
            name: "French".to_string(),
            native_name: Some("Français".to_string()),
        },
        Language {
            code: "de".to_string(),
            name: "German".to_string(),
            native_name: Some("Deutsch".to_string()),
        },
        Language {
            code: "ja".to_string(),
            name: "Japanese".to_string(),
            native_name: Some("日本語".to_string()),
        },
        Language {
            code: "zh".to_string(),
            name: "Chinese".to_string(),
            native_name: Some("中文".to_string()),
        },
        Language {
            code: "it".to_string(),
            name: "Italian".to_string(),
            native_name: Some("Italiano".to_string()),
        },
        Language {
            code: "pt".to_string(),
            name: "Portuguese".to_string(),
            native_name: Some("Português".to_string()),
        },
        Language {
            code: "ko".to_string(),
            name: "Korean".to_string(),
            native_name: Some("한국어".to_string()),
        },
        Language {
            code: "ar".to_string(),
            name: "Arabic".to_string(),
            native_name: Some("العربية".to_string()),
        },
    ])
}

/// Set user's preferred target language for translation
#[tauri::command]
pub async fn set_target_language(
    language_code: String,
    state: State<'_, crate::commands::AppState>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.set_user_setting("translation_target_lang", &language_code)
        .map_err(|e| format!("Failed to set target language: {}", e))
}

/// Get user's preferred target language
#[tauri::command]
pub async fn get_target_language(
    state: State<'_, crate::commands::AppState>,
) -> Result<Option<String>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.get_user_setting("translation_target_lang")
        .map_err(|e| format!("Failed to get target language: {}", e))
}

/// Clear translation cache older than specified days
#[tauri::command]
pub async fn clear_translation_cache(
    days: i64,
    cache_state: State<'_, Arc<TokioMutex<TranslationCache>>>,
) -> Result<u64, String> {
    let cache = cache_state.lock().await;
    cache
        .clear_old_translations(days)
        .await
        .map_err(|e| format!("Failed to clear cache: {}", e))
}

/// Get translation cache statistics
#[tauri::command]
pub async fn get_cache_stats(
    cache_state: State<'_, Arc<TokioMutex<TranslationCache>>>,
) -> Result<CacheStats, String> {
    let cache = cache_state.lock().await;
    let (count, size) = cache
        .get_cache_stats()
        .await
        .map_err(|e| format!("Failed to get cache stats: {}", e))?;

    Ok(CacheStats {
        total_translations: count,
        cache_size_bytes: size,
    })
}

// ===== LIVE TRANSLATION SESSION COMMANDS =====

/// Start a new live translation session
#[tauri::command]
pub async fn start_translation_session(
    note_id: String,
    source_lang: String,
    target_lang: String,
    state: State<'_, crate::commands::AppState>,
) -> Result<LiveTranslationSession, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let db_path = db.get_db_path();
    drop(db);

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let session_manager = LiveSessionManager::new(Arc::new(std::sync::Mutex::new(conn)))
        .map_err(|e| format!("Failed to initialize session manager: {}", e))?;

    session_manager
        .start_session(&note_id, &source_lang, &target_lang)
        .map_err(|e| format!("Failed to start session: {}", e))
}

/// End an active live translation session
#[tauri::command]
pub async fn end_translation_session(
    session_id: String,
    state: State<'_, crate::commands::AppState>,
) -> Result<LiveTranslationSession, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let db_path = db.get_db_path();
    drop(db);

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let session_manager = LiveSessionManager::new(Arc::new(std::sync::Mutex::new(conn)))
        .map_err(|e| format!("Failed to initialize session manager: {}", e))?;

    session_manager
        .end_session(&session_id)
        .map_err(|e| format!("Failed to end session: {}", e))
}

/// Get active translation session for a note
#[tauri::command]
pub async fn get_active_translation_session(
    note_id: String,
    state: State<'_, crate::commands::AppState>,
) -> Result<Option<LiveTranslationSession>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let db_path = db.get_db_path();
    drop(db);

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let session_manager = LiveSessionManager::new(Arc::new(std::sync::Mutex::new(conn)))
        .map_err(|e| format!("Failed to initialize session manager: {}", e))?;

    session_manager
        .get_active_session(&note_id)
        .map_err(|e| format!("Failed to get active session: {}", e))
}

/// Get translation segments for a session
#[tauri::command]
pub async fn get_translation_segments(
    session_id: String,
    state: State<'_, crate::commands::AppState>,
) -> Result<Vec<TranslationSegment>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let db_path = db.get_db_path();
    drop(db);

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let session_manager = LiveSessionManager::new(Arc::new(std::sync::Mutex::new(conn)))
        .map_err(|e| format!("Failed to initialize session manager: {}", e))?;

    session_manager
        .get_segments(&session_id)
        .map_err(|e| format!("Failed to get segments: {}", e))
}

/// List all translation sessions for a note
#[tauri::command]
pub async fn list_translation_sessions(
    note_id: String,
    state: State<'_, crate::commands::AppState>,
) -> Result<Vec<LiveTranslationSession>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let db_path = db.get_db_path();
    drop(db);

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let session_manager = LiveSessionManager::new(Arc::new(std::sync::Mutex::new(conn)))
        .map_err(|e| format!("Failed to initialize session manager: {}", e))?;

    session_manager
        .list_sessions(&note_id)
        .map_err(|e| format!("Failed to list sessions: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_text_request_serialization() {
        let request = TranslateTextRequest {
            text: "Hello".to_string(),
            source_lang: "en".to_string(),
            target_lang: "es".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Hello"));
        assert!(json.contains("en"));
        assert!(json.contains("es"));
    }

    #[test]
    fn test_cache_stats_serialization() {
        let stats = CacheStats {
            total_translations: 100,
            cache_size_bytes: 1024,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("1024"));
    }
}
