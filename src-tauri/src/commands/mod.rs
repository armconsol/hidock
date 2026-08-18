// Tauri IPC commands

// pub mod audio; // TODO: Fix audio module compilation errors
pub mod auth_commands;
pub mod notes_commands;

use crate::db::{
    types::{
        CalendarEvent, InsertSmartLabel, InsertVocabulary, SmartLabel, Template, UpdateSmartLabel,
        Vocabulary,
    },
    Database,
};
use chrono::{DateTime, Utc};
use std::sync::Mutex;
use tauri::State;

// Re-export audio command functions and their generated helpers
// pub use audio::*; // TODO: Fix audio module compilation errors
pub use auth_commands::*;
pub use notes_commands::*;

pub struct AppState {
    pub db: Mutex<Database>,
}

/// Get calendar events within a time range
#[tauri::command]
pub async fn get_calendar_events(
    start_time: String,
    end_time: String,
    state: State<'_, AppState>,
) -> Result<Vec<CalendarEvent>, String> {
    let start = DateTime::parse_from_rfc3339(&start_time)
        .map_err(|e| format!("Invalid start_time: {}", e))?
        .with_timezone(&Utc);

    let end = DateTime::parse_from_rfc3339(&end_time)
        .map_err(|e| format!("Invalid end_time: {}", e))?
        .with_timezone(&Utc);

    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_calendar_events_in_range(start, end)
        .map_err(|e| format!("Failed to fetch calendar events: {}", e))
}

/// Get events for today
#[tauri::command]
pub async fn get_today_events(state: State<'_, AppState>) -> Result<Vec<CalendarEvent>, String> {
    let now = Utc::now();
    let start_of_day = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_of_day = now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc();

    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_calendar_events_in_range(start_of_day, end_of_day)
        .map_err(|e| format!("Failed to fetch today's events: {}", e))
}

/// Create a new calendar event
#[tauri::command]
pub async fn create_calendar_event(
    event: CalendarEvent,
    state: State<'_, AppState>,
) -> Result<CalendarEvent, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.insert_calendar_event(&event)
        .map_err(|e| format!("Failed to create calendar event: {}", e))
}

/// Update an existing calendar event
#[tauri::command]
pub async fn update_calendar_event(
    event: CalendarEvent,
    state: State<'_, AppState>,
) -> Result<CalendarEvent, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.update_calendar_event(&event)
        .map_err(|e| format!("Failed to update calendar event: {}", e))
}

/// Delete a calendar event
#[tauri::command]
pub async fn delete_calendar_event(
    event_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_calendar_event(&event_id)
        .map_err(|e| format!("Failed to delete calendar event: {}", e))
}

// get_audio moved to audio.rs module

// ===== TEMPLATE COMMANDS =====

/// List all templates with optional filtering
#[tauri::command]
pub async fn list_templates(
    favorite_only: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<Template>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_templates(
        favorite_only.unwrap_or(false),
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    )
    .map_err(|e| format!("Failed to list templates: {}", e))
}

/// Get a template by ID
#[tauri::command]
pub async fn get_template(
    template_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Template>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.get_template(&template_id)
        .map_err(|e| format!("Failed to get template: {}", e))
}

/// Get the default template
#[tauri::command]
pub async fn get_default_template(state: State<'_, AppState>) -> Result<Option<Template>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.get_default_template()
        .map_err(|e| format!("Failed to get default template: {}", e))
}

/// Create a new template
#[tauri::command]
pub async fn create_template(
    id: String,
    title: String,
    content: String,
    is_favorite: Option<bool>,
    is_default: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Template, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.insert_template(
        &id,
        &title,
        &content,
        is_favorite.unwrap_or(false),
        is_default.unwrap_or(false),
    )
    .map_err(|e| format!("Failed to create template: {}", e))
}

/// Update an existing template
#[tauri::command]
pub async fn update_template(
    id: String,
    title: Option<String>,
    content: Option<String>,
    is_favorite: Option<bool>,
    is_default: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Template, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.update_template(
        &id,
        title.as_deref(),
        content.as_deref(),
        is_favorite,
        is_default,
    )
    .map_err(|e| format!("Failed to update template: {}", e))
}

/// Toggle favorite status of a template
#[tauri::command]
pub async fn toggle_template_favorite(
    id: String,
    state: State<'_, AppState>,
) -> Result<Template, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.toggle_template_favorite(&id)
        .map_err(|e| format!("Failed to toggle template favorite: {}", e))
}

/// Set a template as default
#[tauri::command]
pub async fn set_template_default(
    id: String,
    state: State<'_, AppState>,
) -> Result<Template, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.set_template_default(&id)
        .map_err(|e| format!("Failed to set template as default: {}", e))
}

/// Delete a template
#[tauri::command]
pub async fn delete_template(
    template_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_template(&template_id)
        .map_err(|e| format!("Failed to delete template: {}", e))
}

// ===== SMART LABELS COMMANDS =====

/// List all smart labels
#[tauri::command]
pub async fn list_smart_labels(state: State<'_, AppState>) -> Result<Vec<SmartLabel>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_smart_labels()
        .map_err(|e| format!("Failed to list smart labels: {}", e))
}

/// Get a smart label by ID
#[tauri::command]
pub async fn get_smart_label(
    label_id: String,
    state: State<'_, AppState>,
) -> Result<Option<SmartLabel>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.get_smart_label(&label_id)
        .map_err(|e| format!("Failed to get smart label: {}", e))
}

/// Create a new smart label
#[tauri::command]
pub async fn create_smart_label(
    id: String,
    name: String,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<SmartLabel, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let label = InsertSmartLabel { id, name, color };

    db.insert_smart_label(&label)
        .map_err(|e| format!("Failed to create smart label: {}", e))
}

/// Update an existing smart label
#[tauri::command]
pub async fn update_smart_label(
    id: String,
    name: Option<String>,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<SmartLabel, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let update = UpdateSmartLabel { name, color };

    db.update_smart_label(&id, &update)
        .map_err(|e| format!("Failed to update smart label: {}", e))
}

/// Delete a smart label
#[tauri::command]
pub async fn delete_smart_label(
    label_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_smart_label(&label_id)
        .map_err(|e| format!("Failed to delete smart label: {}", e))
}

// ===== VOCABULARY COMMANDS =====

/// List all vocabulary words
#[tauri::command]
pub async fn list_vocabulary(state: State<'_, AppState>) -> Result<Vec<Vocabulary>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_vocabulary()
        .map_err(|e| format!("Failed to list vocabulary: {}", e))
}

/// Get a vocabulary word by ID
#[tauri::command]
pub async fn get_vocabulary(
    vocab_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Vocabulary>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.get_vocabulary(&vocab_id)
        .map_err(|e| format!("Failed to get vocabulary: {}", e))
}

/// Create a new vocabulary word
#[tauri::command]
pub async fn create_vocabulary(
    id: String,
    word: String,
    pronunciation: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vocabulary, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let vocab = InsertVocabulary {
        id,
        word,
        pronunciation,
    };

    db.insert_vocabulary(&vocab)
        .map_err(|e| format!("Failed to create vocabulary: {}", e))
}

/// Delete a vocabulary word
#[tauri::command]
pub async fn delete_vocabulary(vocab_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_vocabulary(&vocab_id)
        .map_err(|e| format!("Failed to delete vocabulary: {}", e))
}

/// Export vocabulary as JSON
#[tauri::command]
pub async fn export_vocabulary(state: State<'_, AppState>) -> Result<String, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.export_vocabulary_json()
        .map_err(|e| format!("Failed to export vocabulary: {}", e))
}

/// Import vocabulary from JSON
#[tauri::command]
pub async fn import_vocabulary(json: String, state: State<'_, AppState>) -> Result<usize, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.import_vocabulary_json(&json)
        .map_err(|e| format!("Failed to import vocabulary: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::types::EventSource;
    use chrono::Duration;

    fn setup_db() -> Database {
        Database::new_in_memory().expect("Failed to create test database")
    }

    fn create_test_event(id: &str, title: &str, hours_from_now: i64) -> CalendarEvent {
        let now = Utc::now();
        CalendarEvent {
            id: id.to_string(),
            title: title.to_string(),
            start_time: now + Duration::hours(hours_from_now),
            end_time: now + Duration::hours(hours_from_now + 1),
            source: EventSource::Hinotes,
            meeting_url: None,
            created_at: now,
            synced_at: None,
        }
    }

    #[test]
    fn test_calendar_event_database_operations() {
        let db = setup_db();
        let event = create_test_event("event-1", "Test Event", 2);

        // Test insert
        let inserted = db.insert_calendar_event(&event).unwrap();
        assert_eq!(inserted.id, "event-1");
        assert_eq!(inserted.title, "Test Event");

        // Test get
        let retrieved = db.get_calendar_event("event-1").unwrap().unwrap();
        assert_eq!(retrieved.title, "Test Event");

        // Test update
        let mut updated_event = retrieved.clone();
        updated_event.title = "Updated Event".to_string();
        let updated = db.update_calendar_event(&updated_event).unwrap();
        assert_eq!(updated.title, "Updated Event");

        // Test delete
        db.delete_calendar_event("event-1").unwrap();
        assert!(db.get_calendar_event("event-1").unwrap().is_none());
    }

    #[test]
    fn test_list_calendar_events_by_date_range() {
        let db = setup_db();
        let now = Utc::now();

        // Insert multiple events at different times
        let event1 = create_test_event("event-1", "Event 1", -2);
        let event2 = create_test_event("event-2", "Event 2", 1);
        let event3 = create_test_event("event-3", "Event 3", 5);

        db.insert_calendar_event(&event1).unwrap();
        db.insert_calendar_event(&event2).unwrap();
        db.insert_calendar_event(&event3).unwrap();

        // Query events in range
        let start = now - Duration::hours(1);
        let end = now + Duration::hours(3);

        let events = db.list_calendar_events_by_date_range(&start, &end).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Event 2");
    }

    #[test]
    fn test_calendar_event_sources() {
        let db = setup_db();
        let now = Utc::now();

        let google_event = CalendarEvent {
            id: "google-1".to_string(),
            title: "Google Meeting".to_string(),
            start_time: now,
            end_time: now + Duration::hours(1),
            source: EventSource::GoogleCalendar,
            meeting_url: Some("https://meet.google.com/test".to_string()),
            created_at: now,
            synced_at: None,
        };

        let hinotes_event = CalendarEvent {
            id: "hinotes-1".to_string(),
            title: "HiNotes Event".to_string(),
            start_time: now,
            end_time: now + Duration::hours(1),
            source: EventSource::Hinotes,
            meeting_url: None,
            created_at: now,
            synced_at: None,
        };

        db.insert_calendar_event(&google_event).unwrap();
        db.insert_calendar_event(&hinotes_event).unwrap();

        let retrieved_google = db.get_calendar_event("google-1").unwrap().unwrap();
        assert_eq!(retrieved_google.source, EventSource::GoogleCalendar);

        let retrieved_hinotes = db.get_calendar_event("hinotes-1").unwrap().unwrap();
        assert_eq!(retrieved_hinotes.source, EventSource::Hinotes);
    }
}
