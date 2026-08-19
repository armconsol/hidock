use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Live translation session for real-time translation during recording
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LiveTranslationSession {
    pub id: String,
    pub note_id: String,
    pub source_lang: String,
    pub target_lang: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

/// Translation segment - a piece of translated text with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationSegment {
    pub id: String,
    pub session_id: String,
    pub source_text: String,
    pub translated_text: String,
    pub speaker_id: Option<String>,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

/// Manager for live translation sessions
pub struct LiveSessionManager {
    conn: Arc<Mutex<Connection>>,
}

impl LiveSessionManager {
    /// Create a new live session manager
    pub fn new(conn: Arc<Mutex<Connection>>) -> Result<Self> {
        let manager = Self { conn };
        manager.initialize_schema()?;
        Ok(manager)
    }

    /// Initialize database schema for live translations
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS live_translation_sessions (
                id TEXT PRIMARY KEY,
                note_id TEXT NOT NULL,
                source_lang TEXT NOT NULL,
                target_lang TEXT NOT NULL,
                active BOOLEAN NOT NULL DEFAULT 1,
                created_at DATETIME NOT NULL,
                ended_at DATETIME,
                FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS live_translation_segments (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                source_text TEXT NOT NULL,
                translated_text TEXT NOT NULL,
                speaker_id TEXT,
                start_time REAL NOT NULL,
                end_time REAL NOT NULL,
                confidence REAL NOT NULL CHECK(confidence >= 0.0 AND confidence <= 1.0),
                created_at DATETIME NOT NULL,
                FOREIGN KEY (session_id) REFERENCES live_translation_sessions(id) ON DELETE CASCADE,
                FOREIGN KEY (speaker_id) REFERENCES speakers(id) ON DELETE SET NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_live_sessions_note
             ON live_translation_sessions(note_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_live_segments_session
             ON live_translation_segments(session_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_live_segments_time
             ON live_translation_segments(start_time, end_time)",
            [],
        )?;

        Ok(())
    }

    /// Start a new live translation session
    pub fn start_session(
        &self,
        note_id: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<LiveTranslationSession> {
        let session = LiveTranslationSession {
            id: Uuid::new_v4().to_string(),
            note_id: note_id.to_string(),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
            active: true,
            created_at: Utc::now(),
            ended_at: None,
        };

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO live_translation_sessions
             (id, note_id, source_lang, target_lang, active, created_at, ended_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &session.id,
                &session.note_id,
                &session.source_lang,
                &session.target_lang,
                session.active as i32,
                session.created_at.to_rfc3339(),
                session.ended_at.as_ref().map(|dt| dt.to_rfc3339()),
            ],
        )?;

        Ok(session)
    }

    /// End an active live translation session
    pub fn end_session(&self, session_id: &str) -> Result<LiveTranslationSession> {
        let now = Utc::now();
        let conn = self.conn.lock().unwrap();

        let updated = conn.execute(
            "UPDATE live_translation_sessions
             SET active = 0, ended_at = ?1
             WHERE id = ?2",
            params![now.to_rfc3339(), session_id],
        )?;

        if updated == 0 {
            anyhow::bail!("Session not found: {}", session_id);
        }

        drop(conn);
        self.get_session(session_id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve ended session"))
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Result<Option<LiveTranslationSession>> {
        let conn = self.conn.lock().unwrap();

        let session = conn
            .query_row(
                "SELECT id, note_id, source_lang, target_lang, active, created_at, ended_at
                 FROM live_translation_sessions WHERE id = ?1",
                params![session_id],
                |row| {
                    let created_at_str: String = row.get(5)?;
                    let ended_at_str: Option<String> = row.get(6)?;

                    Ok(LiveTranslationSession {
                        id: row.get(0)?,
                        note_id: row.get(1)?,
                        source_lang: row.get(2)?,
                        target_lang: row.get(3)?,
                        active: row.get::<_, i32>(4)? != 0,
                        created_at: DateTime::parse_from_rfc3339(&created_at_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now()),
                        ended_at: ended_at_str.and_then(|s| {
                            DateTime::parse_from_rfc3339(&s)
                                .map(|dt| dt.with_timezone(&Utc))
                                .ok()
                        }),
                    })
                },
            )
            .optional()?;

        Ok(session)
    }

    /// Get active session for a note
    pub fn get_active_session(&self, note_id: &str) -> Result<Option<LiveTranslationSession>> {
        let conn = self.conn.lock().unwrap();

        let session = conn
            .query_row(
                "SELECT id, note_id, source_lang, target_lang, active, created_at, ended_at
                 FROM live_translation_sessions
                 WHERE note_id = ?1 AND active = 1
                 ORDER BY created_at DESC
                 LIMIT 1",
                params![note_id],
                |row| {
                    let created_at_str: String = row.get(5)?;
                    let ended_at_str: Option<String> = row.get(6)?;

                    Ok(LiveTranslationSession {
                        id: row.get(0)?,
                        note_id: row.get(1)?,
                        source_lang: row.get(2)?,
                        target_lang: row.get(3)?,
                        active: row.get::<_, i32>(4)? != 0,
                        created_at: DateTime::parse_from_rfc3339(&created_at_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now()),
                        ended_at: ended_at_str.and_then(|s| {
                            DateTime::parse_from_rfc3339(&s)
                                .map(|dt| dt.with_timezone(&Utc))
                                .ok()
                        }),
                    })
                },
            )
            .optional()?;

        Ok(session)
    }

    /// List all sessions for a note
    pub fn list_sessions(&self, note_id: &str) -> Result<Vec<LiveTranslationSession>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, note_id, source_lang, target_lang, active, created_at, ended_at
             FROM live_translation_sessions
             WHERE note_id = ?1
             ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map(params![note_id], |row| {
            let created_at_str: String = row.get(5)?;
            let ended_at_str: Option<String> = row.get(6)?;

            Ok(LiveTranslationSession {
                id: row.get(0)?,
                note_id: row.get(1)?,
                source_lang: row.get(2)?,
                target_lang: row.get(3)?,
                active: row.get::<_, i32>(4)? != 0,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                ended_at: ended_at_str.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
            })
        })?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }

        Ok(sessions)
    }

    /// Add a translation segment to a session
    pub fn add_segment(
        &self,
        session_id: &str,
        source_text: &str,
        translated_text: &str,
        speaker_id: Option<&str>,
        start_time: f64,
        end_time: f64,
        confidence: f64,
    ) -> Result<TranslationSegment> {
        let segment = TranslationSegment {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            source_text: source_text.to_string(),
            translated_text: translated_text.to_string(),
            speaker_id: speaker_id.map(|s| s.to_string()),
            start_time,
            end_time,
            confidence,
            created_at: Utc::now(),
        };

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO live_translation_segments
             (id, session_id, source_text, translated_text, speaker_id, start_time, end_time, confidence, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &segment.id,
                &segment.session_id,
                &segment.source_text,
                &segment.translated_text,
                &segment.speaker_id,
                segment.start_time,
                segment.end_time,
                segment.confidence,
                segment.created_at.to_rfc3339(),
            ],
        )?;

        Ok(segment)
    }

    /// Get segments for a session
    pub fn get_segments(&self, session_id: &str) -> Result<Vec<TranslationSegment>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, session_id, source_text, translated_text, speaker_id,
                    start_time, end_time, confidence, created_at
             FROM live_translation_segments
             WHERE session_id = ?1
             ORDER BY start_time ASC",
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            let created_at_str: String = row.get(8)?;

            Ok(TranslationSegment {
                id: row.get(0)?,
                session_id: row.get(1)?,
                source_text: row.get(2)?,
                translated_text: row.get(3)?,
                speaker_id: row.get(4)?,
                start_time: row.get(5)?,
                end_time: row.get(6)?,
                confidence: row.get(7)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        let mut segments = Vec::new();
        for row in rows {
            segments.push(row?);
        }

        Ok(segments)
    }

    /// Delete old sessions (older than days)
    pub fn cleanup_old_sessions(&self, days: i64) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let conn = self.conn.lock().unwrap();

        let deleted = conn.execute(
            "DELETE FROM live_translation_sessions
             WHERE ended_at < ?1",
            params![cutoff.to_rfc3339()],
        )?;

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_manager() -> (LiveSessionManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_live_sessions.db");
        let conn = Connection::open(&db_path).unwrap();

        // Create notes table for foreign key
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

        // Create speakers table for foreign key
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
        conn.execute(
            "INSERT INTO notes (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)",
            params![
                "test-note-1",
                "Test Note",
                Utc::now().to_rfc3339(),
                Utc::now().to_rfc3339()
            ],
        )
        .unwrap();

        let manager = LiveSessionManager::new(Arc::new(Mutex::new(conn))).unwrap();
        (manager, temp_dir)
    }

    #[test]
    fn test_start_and_get_session() {
        let (manager, _temp) = setup_manager();

        let session = manager.start_session("test-note-1", "en", "es").unwrap();

        assert_eq!(session.note_id, "test-note-1");
        assert_eq!(session.source_lang, "en");
        assert_eq!(session.target_lang, "es");
        assert!(session.active);

        let retrieved = manager.get_session(&session.id).unwrap().unwrap();
        assert_eq!(retrieved.id, session.id);
    }

    #[test]
    fn test_end_session() {
        let (manager, _temp) = setup_manager();

        let session = manager.start_session("test-note-1", "en", "es").unwrap();
        assert!(session.active);

        let ended = manager.end_session(&session.id).unwrap();
        assert!(!ended.active);
        assert!(ended.ended_at.is_some());
    }

    #[test]
    fn test_get_active_session() {
        let (manager, _temp) = setup_manager();

        let _session1 = manager.start_session("test-note-1", "en", "es").unwrap();
        manager.end_session(&_session1.id).unwrap();

        let session2 = manager.start_session("test-note-1", "en", "fr").unwrap();

        let active = manager.get_active_session("test-note-1").unwrap().unwrap();
        assert_eq!(active.id, session2.id);
        assert_eq!(active.target_lang, "fr");
    }

    #[test]
    fn test_add_and_get_segments() {
        let (manager, _temp) = setup_manager();

        let session = manager.start_session("test-note-1", "en", "es").unwrap();

        let segment1 = manager
            .add_segment(&session.id, "Hello", "Hola", None, 0.0, 1.5, 0.95)
            .unwrap();

        let segment2 = manager
            .add_segment(&session.id, "World", "Mundo", None, 1.5, 3.0, 0.98)
            .unwrap();

        let segments = manager.get_segments(&session.id).unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].id, segment1.id);
        assert_eq!(segments[0].source_text, "Hello");
        assert_eq!(segments[1].id, segment2.id);
        assert_eq!(segments[1].source_text, "World");
    }

    #[test]
    fn test_segments_ordered_by_time() {
        let (manager, _temp) = setup_manager();

        let session = manager.start_session("test-note-1", "en", "es").unwrap();

        manager
            .add_segment(&session.id, "Third", "Tercero", None, 2.0, 3.0, 0.95)
            .unwrap();
        manager
            .add_segment(&session.id, "First", "Primero", None, 0.0, 1.0, 0.95)
            .unwrap();
        manager
            .add_segment(&session.id, "Second", "Segundo", None, 1.0, 2.0, 0.95)
            .unwrap();

        let segments = manager.get_segments(&session.id).unwrap();
        assert_eq!(segments[0].source_text, "First");
        assert_eq!(segments[1].source_text, "Second");
        assert_eq!(segments[2].source_text, "Third");
    }

    #[test]
    fn test_list_sessions() {
        let (manager, _temp) = setup_manager();

        let session1 = manager.start_session("test-note-1", "en", "es").unwrap();
        let session2 = manager.start_session("test-note-1", "en", "fr").unwrap();

        let sessions = manager.list_sessions("test-note-1").unwrap();
        assert_eq!(sessions.len(), 2);
        // Should be ordered by created_at DESC
        assert_eq!(sessions[0].id, session2.id);
        assert_eq!(sessions[1].id, session1.id);
    }

    #[test]
    fn test_cleanup_old_sessions() {
        let (manager, _temp) = setup_manager();

        let session = manager.start_session("test-note-1", "en", "es").unwrap();
        manager.end_session(&session.id).unwrap();

        // Cleanup sessions older than 0 days (all ended sessions)
        let deleted = manager.cleanup_old_sessions(0).unwrap();
        assert_eq!(deleted, 1);

        let retrieved = manager.get_session(&session.id).unwrap();
        assert!(retrieved.is_none());
    }
}
