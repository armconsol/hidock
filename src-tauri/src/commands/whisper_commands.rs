// Tauri IPC commands for whisper notes

use crate::db::types::{InsertWhisperNote, WhisperNote};
use tauri::State;

use super::AppState;

/// Create a new whisper note (quick voice note)
#[tauri::command]
pub async fn create_whisper_note(
    id: String,
    content: String,
    audio_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<WhisperNote, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let whisper = InsertWhisperNote {
        id,
        content,
        audio_url,
    };

    db.create_whisper_note(&whisper)
        .map_err(|e| format!("Failed to create whisper note: {}", e))
}

/// List whisper notes with pagination
#[tauri::command]
pub async fn list_whisper_notes(
    page_index: Option<i64>,
    page_size: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<WhisperNote>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_whisper_notes(page_index.unwrap_or(0), page_size.unwrap_or(20))
        .map_err(|e| format!("Failed to list whisper notes: {}", e))
}

/// Get a single whisper note by ID
#[tauri::command]
pub async fn get_whisper_note(
    whisper_id: String,
    state: State<'_, AppState>,
) -> Result<Option<WhisperNote>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.get_whisper_note(&whisper_id)
        .map_err(|e| format!("Failed to get whisper note: {}", e))
}

/// Delete a whisper note
#[tauri::command]
pub async fn delete_whisper_note(
    whisper_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_whisper_note(&whisper_id)
        .map_err(|e| format!("Failed to delete whisper note: {}", e))
}

/// Convert whisper note to a full note
/// API: POST /v1/note/whisper/create/note
#[tauri::command]
pub async fn convert_whisper_to_note(
    whisper_id: String,
    folder_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<crate::db::types::Note, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.convert_whisper_to_note(&whisper_id, folder_id)
        .map_err(|e| format!("Failed to convert whisper to note: {}", e))
}

/// Convert whisper note to a todo
/// API: POST /v1/note/whisper/add/todo
#[tauri::command]
pub async fn convert_whisper_to_todo(
    whisper_id: String,
    state: State<'_, AppState>,
) -> Result<crate::db::types::Todo, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.convert_whisper_to_todo(&whisper_id)
        .map_err(|e| format!("Failed to convert whisper to todo: {}", e))
}

/// Extract calendar events from whisper note
/// API: POST /v1/note/whisper/extract/calendar
///
/// Note: This is a placeholder implementation. The actual calendar event extraction
/// would require NLP or AI processing to parse dates/times from the whisper content.
/// For now, this returns an empty list and should be implemented with proper
/// date/time extraction logic in the future.
#[tauri::command]
pub async fn extract_calendar_from_whisper(
    whisper_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::types::CalendarEvent>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    // Verify the whisper note exists
    let _whisper = db
        .get_whisper_note(&whisper_id)
        .map_err(|e| format!("Failed to get whisper note: {}", e))?
        .ok_or_else(|| format!("Whisper note not found: {}", whisper_id))?;

    // TODO: Implement calendar event extraction logic
    // This would parse the whisper content for date/time mentions
    // and create CalendarEvent objects

    // For now, return empty list
    Ok(Vec::new())
}

/// Count total whisper notes
#[tauri::command]
pub async fn count_whisper_notes(state: State<'_, AppState>) -> Result<i64, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.count_whisper_notes()
        .map_err(|e| format!("Failed to count whisper notes: {}", e))
}

#[cfg(test)]
mod tests {
    use crate::db::{types::InsertWhisperNote, Database};

    fn setup_db() -> Database {
        Database::new_in_memory().expect("Failed to create test database")
    }

    fn create_test_whisper(id: &str, content: &str) -> InsertWhisperNote {
        InsertWhisperNote {
            id: id.to_string(),
            content: content.to_string(),
            audio_url: None,
        }
    }

    #[test]
    fn test_whisper_note_operations() {
        let db = setup_db();

        // Create whisper note
        let whisper = create_test_whisper("whisper-1", "Quick voice memo");
        let created = db.create_whisper_note(&whisper).unwrap();
        assert_eq!(created.id, "whisper-1");
        assert_eq!(created.content, "Quick voice memo");

        // Get whisper note
        let retrieved = db.get_whisper_note("whisper-1").unwrap().unwrap();
        assert_eq!(retrieved.id, "whisper-1");
        assert_eq!(retrieved.content, "Quick voice memo");

        // List whisper notes
        let whispers = db.list_whisper_notes(0, 10).unwrap();
        assert_eq!(whispers.len(), 1);

        // Count whisper notes
        let count = db.count_whisper_notes().unwrap();
        assert_eq!(count, 1);

        // Delete whisper note
        db.delete_whisper_note("whisper-1").unwrap();
        let deleted_whisper = db.get_whisper_note("whisper-1").unwrap();
        assert!(deleted_whisper.is_none());
    }

    #[test]
    fn test_convert_whisper_to_note() {
        let db = setup_db();

        // Create folder
        db.insert_folder("folder-1", "Work").unwrap();

        // Create whisper note
        let whisper = create_test_whisper("whisper-1", "Convert this to note");
        db.create_whisper_note(&whisper).unwrap();

        // Convert to note
        let note = db
            .convert_whisper_to_note("whisper-1", Some("folder-1".to_string()))
            .unwrap();

        assert_eq!(note.id, "whisper-1");
        assert_eq!(note.title, "Convert this to note");
        assert_eq!(note.folder_id, Some("folder-1".to_string()));

        // Verify whisper is deleted
        let whisper_gone = db.get_whisper_note("whisper-1").unwrap();
        assert!(whisper_gone.is_none());

        // Verify note exists
        let note_exists = db.get_note("whisper-1").unwrap();
        assert!(note_exists.is_some());
    }

    #[test]
    fn test_convert_whisper_to_todo() {
        let db = setup_db();

        // Create whisper note
        let whisper = create_test_whisper("whisper-2", "Remember to buy milk");
        db.create_whisper_note(&whisper).unwrap();

        // Convert to todo
        let todo = db.convert_whisper_to_todo("whisper-2").unwrap();

        assert_eq!(todo.id, "whisper-2");
        assert_eq!(todo.description, "Remember to buy milk");
        assert_eq!(todo.state, crate::db::types::TodoState::Open);

        // Verify whisper is deleted
        let whisper_gone = db.get_whisper_note("whisper-2").unwrap();
        assert!(whisper_gone.is_none());

        // Verify todo exists
        let todo_exists = db.get_todo("whisper-2").unwrap();
        assert!(todo_exists.is_some());
    }

    #[test]
    fn test_list_whisper_notes_with_pagination() {
        let db = setup_db();

        // Create multiple whisper notes
        for i in 1..=15 {
            let whisper = create_test_whisper(&format!("whisper-{}", i), &format!("Content {}", i));
            db.create_whisper_note(&whisper).unwrap();
        }

        // Test first page
        let page1 = db.list_whisper_notes(0, 5).unwrap();
        assert_eq!(page1.len(), 5);

        // Test second page
        let page2 = db.list_whisper_notes(1, 5).unwrap();
        assert_eq!(page2.len(), 5);

        // Test third page
        let page3 = db.list_whisper_notes(2, 5).unwrap();
        assert_eq!(page3.len(), 5);

        // Verify total count
        let count = db.count_whisper_notes().unwrap();
        assert_eq!(count, 15);
    }

    #[test]
    fn test_whisper_with_audio_url() {
        let db = setup_db();

        let whisper = InsertWhisperNote {
            id: "whisper-audio".to_string(),
            content: "Whisper with audio".to_string(),
            audio_url: Some("https://example.com/audio.mp3".to_string()),
        };

        let created = db.create_whisper_note(&whisper).unwrap();
        assert_eq!(created.audio_url, Some("https://example.com/audio.mp3".to_string()));
    }

    #[test]
    fn test_delete_nonexistent_whisper() {
        let db = setup_db();

        // Attempt to delete non-existent whisper
        let result = db.delete_whisper_note("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_nonexistent_whisper_to_note() {
        let db = setup_db();

        // Attempt to convert non-existent whisper
        let result = db.convert_whisper_to_note("nonexistent", None);
        assert!(result.is_err());
    }
}
