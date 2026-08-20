use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use super::client::TranslationClient;
use super::live_session::{LiveSessionManager, TranslationSegment};
use super::segmentation::TextSegmenter;
use crate::translation::cache::TranslationCache;

/// Event emitted when a translation segment is completed
#[derive(Debug, Clone)]
pub struct TranslationEvent {
    pub session_id: String,
    pub segment: TranslationSegment,
}

/// Real-time translation streaming coordinator
pub struct TranslationStreamer {
    client: Arc<TranslationClient>,
    session_manager: Arc<LiveSessionManager>,
    cache: Arc<TranslationCache>,
    segmenter: Arc<TextSegmenter>,
    event_sender: broadcast::Sender<TranslationEvent>,
    pending_translations: Arc<RwLock<Vec<PendingTranslation>>>,
}

#[derive(Debug, Clone)]
struct PendingTranslation {
    session_id: String,
    source_text: String,
    speaker_id: Option<String>,
    start_time: f64,
    end_time: f64,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TranslationStreamer {
    /// Create a new translation streamer
    pub fn new(
        client: Arc<TranslationClient>,
        session_manager: Arc<LiveSessionManager>,
        cache: Arc<TranslationCache>,
        segmenter: Arc<TextSegmenter>,
    ) -> Self {
        let (event_sender, _) = broadcast::channel(1000);

        Self {
            client,
            session_manager,
            cache,
            segmenter,
            event_sender,
            pending_translations: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Subscribe to translation events
    pub fn subscribe(&self) -> broadcast::Receiver<TranslationEvent> {
        self.event_sender.subscribe()
    }

    /// Process incoming transcription text and translate it in real-time
    pub async fn process_transcription(
        &self,
        session_id: &str,
        text: &str,
        speaker_id: Option<&str>,
        start_time: f64,
        end_time: f64,
    ) -> Result<()> {
        // Get session to determine languages
        let session = self
            .session_manager
            .get_session(session_id)?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        if !session.active {
            anyhow::bail!("Session is not active: {}", session_id);
        }

        // Segment the text if needed
        let segments = self.segmenter.segment_text(text);

        for segment_text in segments {
            if segment_text.trim().is_empty() {
                continue;
            }

            // Try to translate
            match self
                .translate_segment(
                    session_id,
                    &session.source_lang,
                    &session.target_lang,
                    &segment_text,
                    speaker_id,
                    start_time,
                    end_time,
                )
                .await
            {
                Ok(segment) => {
                    // Emit translation event
                    let event = TranslationEvent {
                        session_id: session_id.to_string(),
                        segment,
                    };

                    // Ignore send errors (no active receivers)
                    let _ = self.event_sender.send(event);
                }
                Err(e) => {
                    // If translation fails (e.g., offline), queue for later
                    log::warn!("Failed to translate segment, queueing: {}", e);
                    self.queue_pending_translation(
                        session_id,
                        segment_text,
                        speaker_id,
                        start_time,
                        end_time,
                    )
                    .await;
                }
            }
        }

        Ok(())
    }

    /// Translate a single segment
    async fn translate_segment(
        &self,
        session_id: &str,
        source_lang: &str,
        target_lang: &str,
        text: &str,
        speaker_id: Option<&str>,
        start_time: f64,
        end_time: f64,
    ) -> Result<TranslationSegment> {
        // Check cache first
        if let Some(cached) = self
            .cache
            .get_translation(text, source_lang, target_lang)
            .await?
        {
            let segment = self.session_manager.add_segment(
                session_id,
                text,
                &cached.translated_text,
                speaker_id,
                start_time,
                end_time,
                1.0, // Cached translations have high confidence
            )?;

            return Ok(segment);
        }

        // Translate via API
        let response = self
            .client
            .translate(text, Some(source_lang), Some(target_lang))
            .await?;

        // Cache the result
        self.cache
            .save_translation(text, source_lang, target_lang, &response.translated_text)
            .await?;

        // Save segment
        let segment = self.session_manager.add_segment(
            session_id,
            text,
            &response.translated_text,
            speaker_id,
            start_time,
            end_time,
            response.confidence.unwrap_or(0.9),
        )?;

        Ok(segment)
    }

    /// Queue a translation for later processing (offline mode)
    async fn queue_pending_translation(
        &self,
        session_id: &str,
        text: String,
        speaker_id: Option<&str>,
        start_time: f64,
        end_time: f64,
    ) {
        let pending = PendingTranslation {
            session_id: session_id.to_string(),
            source_text: text,
            speaker_id: speaker_id.map(|s| s.to_string()),
            start_time,
            end_time,
            created_at: chrono::Utc::now(),
        };

        let mut pending_list = self.pending_translations.write().await;
        pending_list.push(pending);
    }

    /// Process all pending translations (call when coming back online)
    pub async fn process_pending_translations(&self) -> Result<usize> {
        let mut pending_list = self.pending_translations.write().await;

        if pending_list.is_empty() {
            return Ok(0);
        }

        let mut processed_count = 0;
        let mut failed_translations = Vec::new();

        // Process in chronological order
        pending_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        for pending in pending_list.drain(..) {
            // Get session to determine languages
            if let Ok(Some(session)) = self.session_manager.get_session(&pending.session_id) {
                match self
                    .translate_segment(
                        &pending.session_id,
                        &session.source_lang,
                        &session.target_lang,
                        &pending.source_text,
                        pending.speaker_id.as_deref(),
                        pending.start_time,
                        pending.end_time,
                    )
                    .await
                {
                    Ok(segment) => {
                        // Emit translation event
                        let event = TranslationEvent {
                            session_id: pending.session_id.clone(),
                            segment,
                        };

                        let _ = self.event_sender.send(event);
                        processed_count += 1;
                    }
                    Err(e) => {
                        log::error!("Failed to process pending translation: {}", e);
                        failed_translations.push(pending);
                    }
                }
            }
        }

        // Re-queue failed translations
        pending_list.extend(failed_translations);

        Ok(processed_count)
    }

    /// Get count of pending translations
    pub async fn pending_count(&self) -> usize {
        self.pending_translations.read().await.len()
    }

    /// Clear all pending translations (e.g., on session end)
    pub async fn clear_pending_for_session(&self, session_id: &str) {
        let mut pending_list = self.pending_translations.write().await;
        pending_list.retain(|p| p.session_id != session_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::HiNotesClient;
    use rusqlite::Connection;
    use std::sync::Mutex;
    use tempfile::TempDir;

    fn setup_streamer() -> (TranslationStreamer, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_streaming.db");
        let conn = Connection::open(&db_path).unwrap();

        // Create required tables
        conn.execute(
            "CREATE TABLE notes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE speakers (
                id TEXT PRIMARY KEY,
                name TEXT,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL
            )",
            [],
        )
        .unwrap();

        // Insert test note
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO notes (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)",
            rusqlite::params!["test-note-1", "Test Note", &now, &now],
        )
        .unwrap();

        let conn_arc = Arc::new(Mutex::new(conn));

        let hinotes_client = Arc::new(HiNotesClient::with_base_url(
            "http://localhost:3001/v1".to_string(),
        ));
        let client = Arc::new(TranslationClient::new(hinotes_client));
        let session_manager = Arc::new(LiveSessionManager::new(conn_arc.clone()).unwrap());
        let cache = Arc::new(TranslationCache::new(&db_path.to_string_lossy()).unwrap());
        let segmenter = Arc::new(TextSegmenter::new());

        let streamer = TranslationStreamer::new(client, session_manager, cache, segmenter);

        (streamer, temp_dir)
    }

    #[tokio::test]
    async fn test_subscribe_to_events() {
        let (streamer, _temp) = setup_streamer();

        // Just test that subscription works; reaching this point without a
        // panic confirms subscribe() succeeded.
        let _receiver = streamer.subscribe();
    }

    #[tokio::test]
    async fn test_queue_pending_translation() {
        let (streamer, _temp) = setup_streamer();

        assert_eq!(streamer.pending_count().await, 0);

        streamer
            .queue_pending_translation("session-1", "Hello".to_string(), None, 0.0, 1.0)
            .await;

        assert_eq!(streamer.pending_count().await, 1);
    }

    #[tokio::test]
    async fn test_clear_pending_for_session() {
        let (streamer, _temp) = setup_streamer();

        streamer
            .queue_pending_translation("session-1", "Hello".to_string(), None, 0.0, 1.0)
            .await;
        streamer
            .queue_pending_translation("session-2", "World".to_string(), None, 1.0, 2.0)
            .await;

        assert_eq!(streamer.pending_count().await, 2);

        streamer.clear_pending_for_session("session-1").await;

        assert_eq!(streamer.pending_count().await, 1);
    }

    #[tokio::test]
    async fn test_process_transcription_inactive_session() {
        let (streamer, _temp) = setup_streamer();

        let session = streamer
            .session_manager
            .start_session("test-note-1", "en", "es")
            .unwrap();

        // End the session
        streamer.session_manager.end_session(&session.id).unwrap();

        // Should fail because session is inactive
        let result = streamer
            .process_transcription(&session.id, "Hello", None, 0.0, 1.0)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not active"));
    }

    #[tokio::test]
    async fn test_process_transcription_nonexistent_session() {
        let (streamer, _temp) = setup_streamer();

        let result = streamer
            .process_transcription("nonexistent", "Hello", None, 0.0, 1.0)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
