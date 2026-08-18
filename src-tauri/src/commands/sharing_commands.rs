// Tauri IPC commands for note sharing

use crate::db::types::{InsertShareLink, ShareLink};
use chrono::{DateTime, Duration, Utc};
use tauri::State;
use uuid::Uuid;

use super::AppState;

/// Create a share link for a note
#[tauri::command]
pub async fn create_share_link(
    note_id: String,
    expires_in_days: Option<i64>,
    state: State<'_, AppState>,
) -> Result<ShareLink, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    // Verify the note exists
    let note = db
        .get_note(&note_id)
        .map_err(|e| format!("Failed to get note: {}", e))?;

    if note.is_none() {
        return Err(format!("Note not found: {}", note_id));
    }

    // Generate a unique token
    let token = Uuid::new_v4().to_string();

    // Calculate expiration date if provided
    let expires_at = expires_in_days.map(|days| Utc::now() + Duration::days(days));

    let share = InsertShareLink {
        id: Uuid::new_v4().to_string(),
        note_id,
        token,
        expires_at,
    };

    db.insert_share_link(&share)
        .map_err(|e| format!("Failed to create share link: {}", e))
}

/// List all share links for a note
#[tauri::command]
pub async fn list_share_links(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ShareLink>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_share_links_by_note(&note_id)
        .map_err(|e| format!("Failed to list share links: {}", e))
}

/// Get a shared note by token (for public viewing)
#[tauri::command]
pub async fn get_shared_note(
    token: String,
    state: State<'_, AppState>,
) -> Result<SharedNoteResponse, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    // Get share link by token
    let share_link = db
        .get_share_link_by_token(&token)
        .map_err(|e| format!("Failed to get share link: {}", e))?;

    let share_link = match share_link {
        Some(link) => link,
        None => return Err("Share link not found or expired".to_string()),
    };

    // Check if expired
    if let Some(expires_at) = share_link.expires_at {
        if expires_at < Utc::now() {
            return Err("Share link has expired".to_string());
        }
    }

    // Get the note
    let note = db
        .get_note(&share_link.note_id)
        .map_err(|e| format!("Failed to get note: {}", e))?
        .ok_or_else(|| "Note not found".to_string())?;

    // Increment access count
    db.increment_share_access(&share_link.id)
        .map_err(|e| format!("Failed to increment access count: {}", e))?;

    Ok(SharedNoteResponse {
        note_id: note.id,
        title: note.title,
        content: note.content,
        created_at: note.created_at,
        updated_at: note.updated_at,
    })
}

/// Delete a share link
#[tauri::command]
pub async fn delete_share_link(share_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_share_link(&share_id)
        .map_err(|e| format!("Failed to delete share link: {}", e))
}

/// Clean up expired share links
#[tauri::command]
pub async fn cleanup_expired_shares(state: State<'_, AppState>) -> Result<usize, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_expired_share_links()
        .map_err(|e| format!("Failed to cleanup expired shares: {}", e))
}

/// Response structure for shared note viewing
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SharedNoteResponse {
    pub note_id: String,
    pub title: String,
    pub content: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{types::InsertNote, Database};

    fn setup_db() -> Database {
        Database::new_in_memory().expect("Failed to create test database")
    }

    fn create_test_note(db: &Database, id: &str, title: &str) -> String {
        let note = InsertNote {
            id: id.to_string(),
            title: title.to_string(),
            content: Some("Test content".to_string()),
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();
        id.to_string()
    }

    #[test]
    fn test_create_and_get_share_link() {
        let db = setup_db();
        let note_id = create_test_note(&db, "note-1", "Test Note");

        let share = InsertShareLink {
            id: "share-1".to_string(),
            note_id: note_id.clone(),
            token: "test-token".to_string(),
            expires_at: None,
        };

        let created = db.insert_share_link(&share).unwrap();
        assert_eq!(created.note_id, note_id);
        assert_eq!(created.token, "test-token");
        assert_eq!(created.access_count, 0);

        let retrieved = db.get_share_link("share-1").unwrap().unwrap();
        assert_eq!(retrieved.id, "share-1");
        assert_eq!(retrieved.token, "test-token");
    }

    #[test]
    fn test_get_share_link_by_token() {
        let db = setup_db();
        let note_id = create_test_note(&db, "note-1", "Test Note");

        let share = InsertShareLink {
            id: "share-1".to_string(),
            note_id,
            token: "unique-token".to_string(),
            expires_at: None,
        };

        db.insert_share_link(&share).unwrap();

        let retrieved = db.get_share_link_by_token("unique-token").unwrap().unwrap();
        assert_eq!(retrieved.id, "share-1");
    }

    #[test]
    fn test_list_share_links_by_note() {
        let db = setup_db();
        let note_id = create_test_note(&db, "note-1", "Test Note");

        // Create multiple share links for the same note
        for i in 1..=3 {
            let share = InsertShareLink {
                id: format!("share-{}", i),
                note_id: note_id.clone(),
                token: format!("token-{}", i),
                expires_at: None,
            };
            db.insert_share_link(&share).unwrap();
        }

        let shares = db.list_share_links_by_note(&note_id).unwrap();
        assert_eq!(shares.len(), 3);
    }

    #[test]
    fn test_delete_share_link() {
        let db = setup_db();
        let note_id = create_test_note(&db, "note-1", "Test Note");

        let share = InsertShareLink {
            id: "share-1".to_string(),
            note_id,
            token: "test-token".to_string(),
            expires_at: None,
        };

        db.insert_share_link(&share).unwrap();
        assert!(db.get_share_link("share-1").unwrap().is_some());

        db.delete_share_link("share-1").unwrap();
        assert!(db.get_share_link("share-1").unwrap().is_none());
    }

    #[test]
    fn test_increment_share_access() {
        let db = setup_db();
        let note_id = create_test_note(&db, "note-1", "Test Note");

        let share = InsertShareLink {
            id: "share-1".to_string(),
            note_id,
            token: "test-token".to_string(),
            expires_at: None,
        };

        db.insert_share_link(&share).unwrap();

        let initial = db.get_share_link("share-1").unwrap().unwrap();
        assert_eq!(initial.access_count, 0);
        assert!(initial.last_accessed_at.is_none());

        db.increment_share_access("share-1").unwrap();

        let updated = db.get_share_link("share-1").unwrap().unwrap();
        assert_eq!(updated.access_count, 1);
        assert!(updated.last_accessed_at.is_some());
    }

    #[test]
    fn test_delete_expired_share_links() {
        let db = setup_db();
        let note_id = create_test_note(&db, "note-1", "Test Note");

        // Create an expired share link
        let expired_share = InsertShareLink {
            id: "share-expired".to_string(),
            note_id: note_id.clone(),
            token: "expired-token".to_string(),
            expires_at: Some(Utc::now() - Duration::days(1)),
        };
        db.insert_share_link(&expired_share).unwrap();

        // Create a valid share link
        let valid_share = InsertShareLink {
            id: "share-valid".to_string(),
            note_id,
            token: "valid-token".to_string(),
            expires_at: Some(Utc::now() + Duration::days(7)),
        };
        db.insert_share_link(&valid_share).unwrap();

        let deleted_count = db.delete_expired_share_links().unwrap();
        assert_eq!(deleted_count, 1);

        assert!(db.get_share_link("share-expired").unwrap().is_none());
        assert!(db.get_share_link("share-valid").unwrap().is_some());
    }
}
