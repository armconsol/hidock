// Tauri IPC commands for notes

use crate::db::types::{InsertNote, Note, UpdateNote};
use tauri::State;

use super::AppState;

/// List notes with pagination
#[tauri::command]
pub async fn list_notes(
    limit: Option<i64>,
    offset: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<Note>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_notes(limit.unwrap_or(20), offset.unwrap_or(0))
        .map_err(|e| format!("Failed to list notes: {}", e))
}

/// List notes by folder with pagination
#[tauri::command]
pub async fn list_notes_by_folder(
    folder_id: String,
    limit: Option<i64>,
    offset: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<Note>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_notes_by_folder(&folder_id, limit.unwrap_or(20), offset.unwrap_or(0))
        .map_err(|e| format!("Failed to list notes by folder: {}", e))
}

/// Get a note by ID
#[tauri::command]
pub async fn get_note(note_id: String, state: State<'_, AppState>) -> Result<Option<Note>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.get_note(&note_id)
        .map_err(|e| format!("Failed to get note: {}", e))
}

/// Create a new note
#[tauri::command]
pub async fn create_note(
    id: String,
    title: String,
    content: Option<String>,
    folder_id: Option<String>,
    audio_url: Option<String>,
    duration: Option<String>,
    rating: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Note, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let note = InsertNote {
        id,
        title,
        content,
        folder_id,
        audio_url,
        duration,
        rating,
    };

    db.insert_note(&note)
        .map_err(|e| format!("Failed to create note: {}", e))
}

/// Update an existing note
#[tauri::command]
pub async fn update_note(
    id: String,
    title: Option<String>,
    content: Option<String>,
    folder_id: Option<String>,
    audio_url: Option<String>,
    duration: Option<String>,
    rating: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Note, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let update = UpdateNote {
        title,
        content,
        folder_id,
        audio_url,
        duration,
        rating,
    };

    db.update_note(&id, &update)
        .map_err(|e| format!("Failed to update note: {}", e))
}

/// Delete a note
#[tauri::command]
pub async fn delete_note(note_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_note(&note_id)
        .map_err(|e| format!("Failed to delete note: {}", e))
}

/// Count total notes
#[tauri::command]
pub async fn count_notes(state: State<'_, AppState>) -> Result<i64, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.count_notes()
        .map_err(|e| format!("Failed to count notes: {}", e))
}

#[cfg(test)]
mod tests {
    use crate::db::{types::InsertNote, Database};

    fn setup_db() -> Database {
        Database::new_in_memory().expect("Failed to create test database")
    }

    fn create_test_note(id: &str, title: &str) -> InsertNote {
        InsertNote {
            id: id.to_string(),
            title: title.to_string(),
            content: Some("Test content".to_string()),
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        }
    }

    #[test]
    fn test_database_operations() {
        let db = setup_db();

        // Test: Notes functionality is tested via the database layer
        // The Tauri commands are thin wrappers that call database methods

        // Create note
        let note = create_test_note("note-1", "Test Note");
        let created = db.insert_note(&note).unwrap();
        assert_eq!(created.id, "note-1");
        assert_eq!(created.title, "Test Note");

        // Get note
        let retrieved = db.get_note("note-1").unwrap().unwrap();
        assert_eq!(retrieved.id, "note-1");

        // List notes
        let notes = db.list_notes(10, 0).unwrap();
        assert_eq!(notes.len(), 1);

        // Update note
        let update = crate::db::types::UpdateNote {
            title: Some("Updated".to_string()),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        let updated = db.update_note("note-1", &update).unwrap();
        assert_eq!(updated.title, "Updated");

        // Count notes
        let count = db.count_notes().unwrap();
        assert_eq!(count, 1);

        // Delete note
        db.delete_note("note-1").unwrap();
        let deleted_note = db.get_note("note-1").unwrap();
        assert!(deleted_note.is_none());
    }

    #[test]
    fn test_list_notes_by_folder() {
        let db = setup_db();

        // Create a folder
        db.insert_folder("folder-1", "Work").unwrap();

        // Create notes in folder
        let note1 = InsertNote {
            id: "note-1".to_string(),
            title: "Work Note 1".to_string(),
            content: None,
            folder_id: Some("folder-1".to_string()),
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note1).unwrap();

        // List notes by folder
        let notes = db.list_notes_by_folder("folder-1", 10, 0).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].folder_id, Some("folder-1".to_string()));
    }
}
