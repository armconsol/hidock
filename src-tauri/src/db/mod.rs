// Database layer - SQLite

pub mod types;

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use types::{
    CalendarEvent, DbSubscription, Device, DeviceStatus, EventSource, Folder, InsertNote,
    InsertShareLink, InsertSmartLabel, InsertSpeaker, InsertSpeakerSegment, InsertSubscription,
    InsertSubscriptionEvent, InsertTodo, InsertVocabulary, InsertWhisperNote, Note,
    PaginationParams, ShareLink, SmartLabel, Speaker, SpeakerSegment, SubscriptionEvent,
    SubscriptionEventType, SubscriptionStatus, SyncStatus, Template, Todo, TodoState, UpdateNote,
    UpdateSmartLabel, UpdateSpeaker, UpdateSubscription, UpdateTodo, Vocabulary, WhisperNote,
};

/// Helper function to parse datetime from SQLite TEXT field (for use outside query_map)
fn parse_datetime(s: String) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| anyhow::anyhow!("Failed to parse datetime: {}", e))
}

/// Helper function to parse optional datetime from SQLite TEXT field (for use outside query_map)
fn parse_datetime_opt(s: Option<String>) -> Result<Option<DateTime<Utc>>> {
    match s {
        Some(s) => Ok(Some(parse_datetime(s)?)),
        None => Ok(None),
    }
}

pub struct Database {
    conn: Connection,
    path: Option<std::path::PathBuf>,
}

impl Database {
    /// Open or create a new SQLite database
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let conn = Connection::open(&path_buf)?;
        let db = Database {
            conn,
            path: Some(path_buf),
        };
        db.initialize()?;
        Ok(db)
    }

    /// Create an in-memory database for testing
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Database { conn, path: None };
        db.initialize()?;
        Ok(db)
    }

    /// Get the database path
    pub fn get_db_path(&self) -> std::path::PathBuf {
        self.path
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from(":memory:"))
    }

    /// Initialize database schema
    fn initialize(&self) -> Result<()> {
        let schema = include_str!("schema.sql");
        self.conn.execute_batch(schema)?;
        Ok(())
    }

    // ===== FOLDER CRUD OPERATIONS =====

    /// Insert a new folder
    pub fn insert_folder(&self, id: &str, name: &str) -> Result<Folder> {
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO folders (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, now.to_rfc3339(), now.to_rfc3339()],
        )?;

        self.get_folder(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted folder"))
    }

    /// Get a folder by ID
    pub fn get_folder(&self, id: &str) -> Result<Option<Folder>> {
        let folder = self
            .conn
            .query_row(
                "SELECT id, name, created_at, updated_at, synced_at FROM folders WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .optional()?;

        match folder {
            Some((id, name, created_at, updated_at, synced_at)) => Ok(Some(Folder {
                id,
                name,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            })),
            None => Ok(None),
        }
    }

    /// List all folders with pagination
    pub fn list_folders(&self, limit: i64, offset: i64) -> Result<Vec<Folder>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at, updated_at, synced_at
             FROM folders
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })?;

        let mut folders = Vec::new();
        for row in rows {
            let (id, name, created_at, updated_at, synced_at) = row?;
            folders.push(Folder {
                id,
                name,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            });
        }

        Ok(folders)
    }

    /// Count total number of folders
    pub fn count_folders(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM folders", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Update a folder's name
    pub fn update_folder(&self, id: &str, name: &str) -> Result<Folder> {
        let now = Utc::now();

        let updated = self.conn.execute(
            "UPDATE folders SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, now.to_rfc3339(), id],
        )?;

        if updated == 0 {
            anyhow::bail!("Folder not found: {}", id);
        }

        self.get_folder(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated folder"))
    }

    /// Delete a folder by ID
    pub fn delete_folder(&self, id: &str) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM folders WHERE id = ?1", params![id])?;

        if deleted == 0 {
            anyhow::bail!("Folder not found: {}", id);
        }

        Ok(())
    }

    /// Mark a folder as synced
    pub fn mark_folder_synced(&self, id: &str) -> Result<()> {
        let now = Utc::now();

        let updated = self.conn.execute(
            "UPDATE folders SET synced_at = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), id],
        )?;

        if updated == 0 {
            anyhow::bail!("Folder not found: {}", id);
        }

        Ok(())
    }

    // ===== NOTES CRUD OPERATIONS =====

    /// Insert a new note
    pub fn insert_note(&self, note: &InsertNote) -> Result<Note> {
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO notes (id, title, content, folder_id, audio_url, duration, rating, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &note.id,
                &note.title,
                &note.content,
                &note.folder_id,
                &note.audio_url,
                &note.duration,
                &note.rating,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        self.get_note(&note.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted note"))
    }

    /// Get a note by ID
    pub fn get_note(&self, id: &str) -> Result<Option<Note>> {
        let note = self
            .conn
            .query_row(
                "SELECT id, title, content, folder_id, audio_url, duration, rating, created_at, updated_at, synced_at
                 FROM notes WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<i32>>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, Option<String>>(9)?,
                    ))
                },
            )
            .optional()?;

        match note {
            Some((
                id,
                title,
                content,
                folder_id,
                audio_url,
                duration,
                rating,
                created_at,
                updated_at,
                synced_at,
            )) => Ok(Some(Note {
                id,
                title,
                content,
                folder_id,
                audio_url,
                duration,
                rating,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            })),
            None => Ok(None),
        }
    }

    /// List notes with pagination
    pub fn list_notes(&self, limit: i64, offset: i64) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, folder_id, audio_url, duration, rating, created_at, updated_at, synced_at
             FROM notes
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<i32>>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })?;

        let mut notes = Vec::new();
        for row in rows {
            let (
                id,
                title,
                content,
                folder_id,
                audio_url,
                duration,
                rating,
                created_at,
                updated_at,
                synced_at,
            ) = row?;
            notes.push(Note {
                id,
                title,
                content,
                folder_id,
                audio_url,
                duration,
                rating,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            });
        }

        Ok(notes)
    }

    /// List notes by folder with pagination
    pub fn list_notes_by_folder(
        &self,
        folder_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, folder_id, audio_url, duration, rating, created_at, updated_at, synced_at
             FROM notes
             WHERE folder_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2 OFFSET ?3",
        )?;

        let rows = stmt.query_map(params![folder_id, limit, offset], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<i32>>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })?;

        let mut notes = Vec::new();
        for row in rows {
            let (
                id,
                title,
                content,
                folder_id,
                audio_url,
                duration,
                rating,
                created_at,
                updated_at,
                synced_at,
            ) = row?;
            notes.push(Note {
                id,
                title,
                content,
                folder_id,
                audio_url,
                duration,
                rating,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            });
        }

        Ok(notes)
    }

    /// Update a note
    pub fn update_note(&self, id: &str, update: &UpdateNote) -> Result<Note> {
        let now = Utc::now();

        // Build dynamic SQL for partial updates
        let mut updates = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref title) = update.title {
            updates.push("title = ?");
            params_vec.push(Box::new(title.clone()));
        }
        if update.content.is_some() {
            updates.push("content = ?");
            params_vec.push(Box::new(update.content.clone()));
        }
        if update.folder_id.is_some() {
            updates.push("folder_id = ?");
            params_vec.push(Box::new(update.folder_id.clone()));
        }
        if update.audio_url.is_some() {
            updates.push("audio_url = ?");
            params_vec.push(Box::new(update.audio_url.clone()));
        }
        if update.duration.is_some() {
            updates.push("duration = ?");
            params_vec.push(Box::new(update.duration.clone()));
        }
        if update.rating.is_some() {
            updates.push("rating = ?");
            params_vec.push(Box::new(update.rating));
        }

        if updates.is_empty() {
            return self
                .get_note(id)?
                .ok_or_else(|| anyhow::anyhow!("Note not found"));
        }

        updates.push("updated_at = ?");
        params_vec.push(Box::new(now.to_rfc3339()));
        params_vec.push(Box::new(id.to_string()));

        let sql = format!("UPDATE notes SET {} WHERE id = ?", updates.join(", "));

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let rows_updated = self.conn.execute(&sql, params_refs.as_slice())?;

        if rows_updated == 0 {
            anyhow::bail!("Note not found: {}", id);
        }

        self.get_note(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated note"))
    }

    /// Delete a note
    pub fn delete_note(&self, id: &str) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM notes WHERE id = ?1", params![id])?;

        if deleted == 0 {
            anyhow::bail!("Note not found: {}", id);
        }

        Ok(())
    }

    /// Count total notes
    pub fn count_notes(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Mark a note as synced
    pub fn mark_note_synced(&self, id: &str) -> Result<()> {
        let now = Utc::now();

        let updated = self.conn.execute(
            "UPDATE notes SET synced_at = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), id],
        )?;

        if updated == 0 {
            anyhow::bail!("Note not found: {}", id);
        }

        Ok(())
    }

    /// Update note with merged audio
    ///
    /// # Arguments
    /// * `note_id` - ID of the note to update
    /// * `audio_url` - Path or URL to the merged audio file
    /// * `duration` - Optional duration string (e.g., "00:05:30")
    ///
    /// # Returns
    /// Updated Note object
    pub fn update_note_with_merged_audio(
        &self,
        note_id: &str,
        audio_url: &str,
        duration: Option<String>,
    ) -> Result<Note> {
        let update = UpdateNote {
            title: None,
            content: None,
            folder_id: None,
            audio_url: Some(audio_url.to_string()),
            duration,
            rating: None,
        };

        self.update_note(note_id, &update)
    }

    // ===== WHISPER NOTES CRUD OPERATIONS =====

    /// Insert a new whisper note
    pub fn create_whisper_note(&self, whisper: &InsertWhisperNote) -> Result<WhisperNote> {
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO whisper_notes (id, content, audio_url, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                &whisper.id,
                &whisper.content,
                &whisper.audio_url,
                now.to_rfc3339(),
            ],
        )?;

        self.get_whisper_note(&whisper.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted whisper note"))
    }

    /// Get a whisper note by ID
    pub fn get_whisper_note(&self, id: &str) -> Result<Option<WhisperNote>> {
        let whisper = self
            .conn
            .query_row(
                "SELECT id, content, audio_url, created_at, synced_at FROM whisper_notes WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .optional()?;

        match whisper {
            Some((id, content, audio_url, created_at, synced_at)) => Ok(Some(WhisperNote {
                id,
                content,
                audio_url,
                created_at: parse_datetime(created_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            })),
            None => Ok(None),
        }
    }

    /// List whisper notes with pagination
    pub fn list_whisper_notes(&self, page_index: i64, page_size: i64) -> Result<Vec<WhisperNote>> {
        let offset = page_index * page_size;

        let mut stmt = self.conn.prepare(
            "SELECT id, content, audio_url, created_at, synced_at
             FROM whisper_notes
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map(params![page_size, offset], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })?;

        let mut whispers = Vec::new();
        for row in rows {
            let (id, content, audio_url, created_at, synced_at) = row?;
            whispers.push(WhisperNote {
                id,
                content,
                audio_url,
                created_at: parse_datetime(created_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            });
        }

        Ok(whispers)
    }

    /// Delete a whisper note by ID
    pub fn delete_whisper_note(&self, id: &str) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM whisper_notes WHERE id = ?1", params![id])?;

        if deleted == 0 {
            anyhow::bail!("Whisper note not found: {}", id);
        }

        Ok(())
    }

    /// Convert a whisper note to a regular note
    pub fn convert_whisper_to_note(
        &self,
        whisper_id: &str,
        folder_id: Option<String>,
    ) -> Result<Note> {
        // Get the whisper note
        let whisper = self
            .get_whisper_note(whisper_id)?
            .ok_or_else(|| anyhow::anyhow!("Whisper note not found: {}", whisper_id))?;

        // Create a new note from the whisper
        let note = InsertNote {
            id: whisper.id.clone(),
            title: whisper.content.clone(),
            content: Some(whisper.content.clone()),
            folder_id,
            audio_url: whisper.audio_url.clone(),
            duration: None,
            rating: None,
        };

        // Insert the note
        let created_note = self.insert_note(&note)?;

        // Delete the whisper note
        self.delete_whisper_note(whisper_id)?;

        Ok(created_note)
    }

    /// Convert a whisper note to a todo
    pub fn convert_whisper_to_todo(&self, whisper_id: &str) -> Result<Todo> {
        // Get the whisper note
        let whisper = self
            .get_whisper_note(whisper_id)?
            .ok_or_else(|| anyhow::anyhow!("Whisper note not found: {}", whisper_id))?;

        // Create a new todo from the whisper
        let todo = InsertTodo {
            id: whisper.id.clone(),
            description: whisper.content.clone(),
            due_date: None,
            state: TodoState::Open,
            smart_label: None,
        };

        // Insert the todo
        let created_todo = self.insert_todo(todo)?;

        // Delete the whisper note
        self.delete_whisper_note(whisper_id)?;

        Ok(created_todo)
    }

    /// Count total whisper notes
    pub fn count_whisper_notes(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM whisper_notes", [], |row| row.get(0))?;
        Ok(count)
    }

    // ===== TODO CRUD OPERATIONS =====

    /// Insert a new todo
    pub fn insert_todo(&self, todo: InsertTodo) -> Result<Todo> {
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO todos (id, description, due_date, state, smart_label, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &todo.id,
                &todo.description,
                todo.due_date.as_ref().map(|dt| dt.to_rfc3339()),
                todo.state.as_str(),
                &todo.smart_label,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        self.get_todo(&todo.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted todo"))
    }

    /// Get a todo by ID
    pub fn get_todo(&self, id: &str) -> Result<Option<Todo>> {
        let result = self
            .conn
            .query_row(
                "SELECT id, description, due_date, state, smart_label, created_at, updated_at, synced_at
                 FROM todos WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, Option<String>>(7)?,
                    ))
                },
            )
            .optional()?;

        match result {
            Some((
                id,
                description,
                due_date,
                state,
                smart_label,
                created_at,
                updated_at,
                synced_at,
            )) => Ok(Some(Todo {
                id,
                description,
                due_date: parse_datetime_opt(due_date)?,
                state: TodoState::from_str(&state)
                    .map_err(|e| anyhow::anyhow!("Invalid todo state: {}", e))?,
                smart_label,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            })),
            None => Ok(None),
        }
    }

    /// List all todos with pagination
    pub fn list_todos(&self, pagination: Option<PaginationParams>) -> Result<Vec<Todo>> {
        let params_obj = pagination.unwrap_or_default();

        let mut stmt = self.conn.prepare(
            "SELECT id, description, due_date, state, smart_label, created_at, updated_at, synced_at
             FROM todos
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map(params![params_obj.limit, params_obj.offset], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        })?;

        let mut todos = Vec::new();
        for row in rows {
            let (id, description, due_date, state, smart_label, created_at, updated_at, synced_at) =
                row?;
            todos.push(Todo {
                id,
                description,
                due_date: parse_datetime_opt(due_date)?,
                state: TodoState::from_str(&state)
                    .map_err(|e| anyhow::anyhow!("Invalid todo state: {}", e))?,
                smart_label,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            });
        }

        Ok(todos)
    }

    /// List todos by state with pagination
    pub fn list_todos_by_state(
        &self,
        state: TodoState,
        pagination: Option<PaginationParams>,
    ) -> Result<Vec<Todo>> {
        let params_obj = pagination.unwrap_or_default();

        let mut stmt = self.conn.prepare(
            "SELECT id, description, due_date, state, smart_label, created_at, updated_at, synced_at
             FROM todos
             WHERE state = ?1
             ORDER BY created_at DESC
             LIMIT ?2 OFFSET ?3",
        )?;

        let rows = stmt.query_map(
            params![state.as_str(), params_obj.limit, params_obj.offset],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            },
        )?;

        let mut todos = Vec::new();
        for row in rows {
            let (id, description, due_date, state, smart_label, created_at, updated_at, synced_at) =
                row?;
            todos.push(Todo {
                id,
                description,
                due_date: parse_datetime_opt(due_date)?,
                state: TodoState::from_str(&state)
                    .map_err(|e| anyhow::anyhow!("Invalid todo state: {}", e))?,
                smart_label,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            });
        }

        Ok(todos)
    }

    /// Update a todo
    pub fn update_todo(&self, id: &str, updates: UpdateTodo) -> Result<Todo> {
        let now = Utc::now();
        let mut update_parts = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref description) = updates.description {
            update_parts.push("description = ?");
            params_vec.push(Box::new(description.clone()));
        }
        if updates.due_date.is_some() {
            update_parts.push("due_date = ?");
            params_vec.push(Box::new(updates.due_date.map(|dt| dt.to_rfc3339())));
        }
        if let Some(ref state) = updates.state {
            update_parts.push("state = ?");
            params_vec.push(Box::new(state.as_str().to_string()));
        }
        if updates.smart_label.is_some() {
            update_parts.push("smart_label = ?");
            params_vec.push(Box::new(updates.smart_label.clone()));
        }

        // Always update updated_at
        update_parts.push("updated_at = ?");
        params_vec.push(Box::new(now.to_rfc3339()));

        // Add id as last param
        params_vec.push(Box::new(id.to_string()));

        let sql = format!("UPDATE todos SET {} WHERE id = ?", update_parts.join(", "));

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();
        let updated = self.conn.execute(&sql, params_refs.as_slice())?;

        if updated == 0 {
            anyhow::bail!("Todo not found: {}", id);
        }

        self.get_todo(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated todo"))
    }

    /// Delete a todo by ID
    pub fn delete_todo(&self, id: &str) -> Result<bool> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM todos WHERE id = ?1", params![id])?;
        Ok(rows_affected > 0)
    }

    /// Count total todos
    pub fn count_todos(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM todos", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Count todos by state
    pub fn count_todos_by_state(&self, state: TodoState) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM todos WHERE state = ?1",
            params![state.as_str()],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Mark a todo as synced
    pub fn mark_todo_synced(&self, id: &str) -> Result<()> {
        let now = Utc::now();

        let updated = self.conn.execute(
            "UPDATE todos SET synced_at = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), id],
        )?;

        if updated == 0 {
            anyhow::bail!("Todo not found: {}", id);
        }

        Ok(())
    }

    // ===== CALENDAR EVENTS CRUD OPERATIONS =====

    /// Insert a new calendar event
    pub fn insert_calendar_event(&self, event: &CalendarEvent) -> Result<CalendarEvent> {
        self.conn.execute(
            "INSERT INTO calendar_events (id, title, start_time, end_time, source, meeting_url, created_at, updated_at, synced_at, google_event_id, sync_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                &event.id,
                &event.title,
                event.start_time.to_rfc3339(),
                event.end_time.to_rfc3339(),
                event.source.as_str(),
                &event.meeting_url,
                event.created_at.to_rfc3339(),
                event.updated_at.to_rfc3339(),
                event.synced_at.as_ref().map(|dt| dt.to_rfc3339()),
                &event.google_event_id,
                event.sync_status.as_str(),
            ],
        )?;

        self.get_calendar_event(&event.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted calendar event"))
    }

    /// Get a calendar event by ID
    pub fn get_calendar_event(&self, id: &str) -> Result<Option<CalendarEvent>> {
        let event = self
            .conn
            .query_row(
                "SELECT id, title, start_time, end_time, source, meeting_url, created_at, updated_at, synced_at, google_event_id, sync_status
                 FROM calendar_events WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, Option<String>>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, String>(10)?,
                    ))
                },
            )
            .optional()?;

        match event {
            Some((
                id,
                title,
                start_time,
                end_time,
                source,
                meeting_url,
                created_at,
                updated_at,
                synced_at,
                google_event_id,
                sync_status,
            )) => Ok(Some(CalendarEvent {
                id,
                title,
                start_time: parse_datetime(start_time)?,
                end_time: parse_datetime(end_time)?,
                source: EventSource::from_str(&source).map_err(|e| anyhow::anyhow!(e))?,
                meeting_url,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
                google_event_id,
                sync_status: SyncStatus::from_str(&sync_status).map_err(|e| anyhow::anyhow!(e))?,
            })),
            None => Ok(None),
        }
    }

    /// List calendar events with pagination
    pub fn list_calendar_events(&self, limit: i64, offset: i64) -> Result<Vec<CalendarEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, start_time, end_time, source, meeting_url, created_at, updated_at, synced_at, google_event_id, sync_status
             FROM calendar_events
             ORDER BY start_time ASC
             LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, String>(10)?,
            ))
        })?;

        let mut events = Vec::new();
        for row in rows {
            let (
                id,
                title,
                start_time,
                end_time,
                source,
                meeting_url,
                created_at,
                updated_at,
                synced_at,
                google_event_id,
                sync_status,
            ) = row?;
            events.push(CalendarEvent {
                id,
                title,
                start_time: parse_datetime(start_time)?,
                end_time: parse_datetime(end_time)?,
                source: EventSource::from_str(&source).map_err(|e| anyhow::anyhow!(e))?,
                meeting_url,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
                google_event_id,
                sync_status: SyncStatus::from_str(&sync_status).map_err(|e| anyhow::anyhow!(e))?,
            });
        }

        Ok(events)
    }

    /// List calendar events for a specific date range (alias for consistency)
    pub fn list_calendar_events_by_date_range(
        &self,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>> {
        self.list_calendar_events_in_range(*start, *end)
    }

    /// List calendar events for a specific date range
    pub fn list_calendar_events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, start_time, end_time, source, meeting_url, created_at, updated_at, synced_at, google_event_id, sync_status
             FROM calendar_events
             WHERE start_time >= ?1 AND start_time < ?2
             ORDER BY start_time ASC",
        )?;

        let rows = stmt.query_map(params![&start.to_rfc3339(), &end.to_rfc3339()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, String>(10)?,
            ))
        })?;

        let mut events = Vec::new();
        for row in rows {
            let (
                id,
                title,
                start_time,
                end_time,
                source,
                meeting_url,
                created_at,
                updated_at,
                synced_at,
                google_event_id,
                sync_status,
            ) = row?;
            events.push(CalendarEvent {
                id,
                title,
                start_time: parse_datetime(start_time)?,
                end_time: parse_datetime(end_time)?,
                source: EventSource::from_str(&source).map_err(|e| anyhow::anyhow!(e))?,
                meeting_url,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
                google_event_id,
                sync_status: SyncStatus::from_str(&sync_status).map_err(|e| anyhow::anyhow!(e))?,
            });
        }

        Ok(events)
    }

    /// Update a calendar event
    pub fn update_calendar_event(&self, event: &CalendarEvent) -> Result<CalendarEvent> {
        let updated = self.conn.execute(
            "UPDATE calendar_events SET title = ?1, start_time = ?2, end_time = ?3, source = ?4, meeting_url = ?5, updated_at = ?6, synced_at = ?7, google_event_id = ?8, sync_status = ?9
             WHERE id = ?10",
            params![
                &event.title,
                event.start_time.to_rfc3339(),
                event.end_time.to_rfc3339(),
                event.source.as_str(),
                &event.meeting_url,
                event.updated_at.to_rfc3339(),
                event.synced_at.as_ref().map(|dt| dt.to_rfc3339()),
                &event.google_event_id,
                event.sync_status.as_str(),
                &event.id,
            ],
        )?;

        if updated == 0 {
            anyhow::bail!("Calendar event not found: {}", event.id);
        }

        self.get_calendar_event(&event.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated calendar event"))
    }

    /// Delete a calendar event
    pub fn delete_calendar_event(&self, id: &str) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM calendar_events WHERE id = ?1", params![id])?;

        if deleted == 0 {
            anyhow::bail!("Calendar event not found: {}", id);
        }

        Ok(())
    }

    /// Count total calendar events
    pub fn count_calendar_events(&self) -> Result<i64> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM calendar_events", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Mark a calendar event as synced
    pub fn mark_calendar_event_synced(&self, id: &str) -> Result<()> {
        let now = Utc::now();

        let updated = self.conn.execute(
            "UPDATE calendar_events SET synced_at = ?1, sync_status = ?2 WHERE id = ?3",
            params![now.to_rfc3339(), SyncStatus::Synced.as_str(), id],
        )?;

        if updated == 0 {
            anyhow::bail!("Calendar event not found: {}", id);
        }

        Ok(())
    }

    /// Get calendar events that need to be pushed to Google
    pub fn get_pending_push_events(&self) -> Result<Vec<CalendarEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, start_time, end_time, source, meeting_url, created_at, updated_at, synced_at, google_event_id, sync_status
             FROM calendar_events
             WHERE sync_status = ?1
             ORDER BY updated_at ASC",
        )?;

        let rows = stmt.query_map(params![SyncStatus::PendingPush.as_str()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, String>(10)?,
            ))
        })?;

        let mut events = Vec::new();
        for row in rows {
            let (
                id,
                title,
                start_time,
                end_time,
                source,
                meeting_url,
                created_at,
                updated_at,
                synced_at,
                google_event_id,
                sync_status,
            ) = row?;
            events.push(CalendarEvent {
                id,
                title,
                start_time: parse_datetime(start_time)?,
                end_time: parse_datetime(end_time)?,
                source: EventSource::from_str(&source).map_err(|e| anyhow::anyhow!(e))?,
                meeting_url,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
                google_event_id,
                sync_status: SyncStatus::from_str(&sync_status).map_err(|e| anyhow::anyhow!(e))?,
            });
        }

        Ok(events)
    }

    /// Update sync status for a calendar event
    pub fn update_calendar_event_sync_status(&self, id: &str, status: SyncStatus) -> Result<()> {
        let now = Utc::now();

        let updated = self.conn.execute(
            "UPDATE calendar_events SET sync_status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), now.to_rfc3339(), id],
        )?;

        if updated == 0 {
            anyhow::bail!("Calendar event not found: {}", id);
        }

        Ok(())
    }

    /// Get calendar event by Google event ID
    pub fn get_calendar_event_by_google_id(
        &self,
        google_event_id: &str,
    ) -> Result<Option<CalendarEvent>> {
        let event = self
            .conn
            .query_row(
                "SELECT id, title, start_time, end_time, source, meeting_url, created_at, updated_at, synced_at, google_event_id, sync_status
                 FROM calendar_events WHERE google_event_id = ?1",
                params![google_event_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, Option<String>>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, String>(10)?,
                    ))
                },
            )
            .optional()?;

        match event {
            Some((
                id,
                title,
                start_time,
                end_time,
                source,
                meeting_url,
                created_at,
                updated_at,
                synced_at,
                google_event_id,
                sync_status,
            )) => Ok(Some(CalendarEvent {
                id,
                title,
                start_time: parse_datetime(start_time)?,
                end_time: parse_datetime(end_time)?,
                source: EventSource::from_str(&source).map_err(|e| anyhow::anyhow!(e))?,
                meeting_url,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
                google_event_id,
                sync_status: SyncStatus::from_str(&sync_status).map_err(|e| anyhow::anyhow!(e))?,
            })),
            None => Ok(None),
        }
    }

    // ===== TEMPLATE CRUD OPERATIONS =====

    /// Insert a new template
    pub fn insert_template(
        &self,
        id: &str,
        title: &str,
        content: &str,
        is_favorite: bool,
        is_default: bool,
    ) -> Result<Template> {
        let now = Utc::now();

        // If setting as default, unset all other default templates first
        if is_default {
            self.conn.execute(
                "UPDATE templates SET is_default = 0 WHERE is_default = 1",
                [],
            )?;
        }

        self.conn.execute(
            "INSERT INTO templates (id, title, content, is_favorite, is_default, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                title,
                content,
                is_favorite as i32,
                is_default as i32,
                now.to_rfc3339(),
                now.to_rfc3339()
            ],
        )?;

        self.get_template(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted template"))
    }

    /// Get a template by ID
    pub fn get_template(&self, id: &str) -> Result<Option<Template>> {
        let template = self
            .conn
            .query_row(
                "SELECT id, title, content, is_favorite, is_default, created_at, updated_at, synced_at
                 FROM templates WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i32>(3)?,
                        row.get::<_, i32>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, Option<String>>(7)?,
                    ))
                },
            )
            .optional()?;

        match template {
            Some((
                id,
                title,
                content,
                is_favorite,
                is_default,
                created_at,
                updated_at,
                synced_at,
            )) => Ok(Some(Template {
                id,
                title,
                content,
                is_favorite: is_favorite != 0,
                is_default: is_default != 0,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            })),
            None => Ok(None),
        }
    }

    /// List all templates with optional filtering
    pub fn list_templates(
        &self,
        favorite_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Template>> {
        let query = if favorite_only {
            "SELECT id, title, content, is_favorite, is_default, created_at, updated_at, synced_at
             FROM templates
             WHERE is_favorite = 1
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2"
        } else {
            "SELECT id, title, content, is_favorite, is_default, created_at, updated_at, synced_at
             FROM templates
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2"
        };

        let mut stmt = self.conn.prepare(query)?;

        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        })?;

        let mut templates = Vec::new();
        for row in rows {
            let (id, title, content, is_favorite, is_default, created_at, updated_at, synced_at) =
                row?;
            templates.push(Template {
                id,
                title,
                content,
                is_favorite: is_favorite != 0,
                is_default: is_default != 0,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            });
        }

        Ok(templates)
    }

    /// Get the default template
    pub fn get_default_template(&self) -> Result<Option<Template>> {
        let template = self
            .conn
            .query_row(
                "SELECT id, title, content, is_favorite, is_default, created_at, updated_at, synced_at
                 FROM templates WHERE is_default = 1 LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i32>(3)?,
                        row.get::<_, i32>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, Option<String>>(7)?,
                    ))
                },
            )
            .optional()?;

        match template {
            Some((
                id,
                title,
                content,
                is_favorite,
                is_default,
                created_at,
                updated_at,
                synced_at,
            )) => Ok(Some(Template {
                id,
                title,
                content,
                is_favorite: is_favorite != 0,
                is_default: is_default != 0,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
                synced_at: parse_datetime_opt(synced_at)?,
            })),
            None => Ok(None),
        }
    }

    /// Update a template
    pub fn update_template(
        &self,
        id: &str,
        title: Option<&str>,
        content: Option<&str>,
        is_favorite: Option<bool>,
        is_default: Option<bool>,
    ) -> Result<Template> {
        let now = Utc::now();

        // Get current template to use existing values where not updated
        let current = self
            .get_template(id)?
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", id))?;

        let new_title = title.unwrap_or(&current.title);
        let new_content = content.unwrap_or(&current.content);
        let new_is_favorite = is_favorite.unwrap_or(current.is_favorite);
        let new_is_default = is_default.unwrap_or(current.is_default);

        // If setting as default, unset all other default templates first
        if new_is_default && !current.is_default {
            self.conn.execute(
                "UPDATE templates SET is_default = 0 WHERE is_default = 1",
                [],
            )?;
        }

        self.conn.execute(
            "UPDATE templates SET title = ?1, content = ?2, is_favorite = ?3, is_default = ?4, updated_at = ?5
             WHERE id = ?6",
            params![
                new_title,
                new_content,
                new_is_favorite as i32,
                new_is_default as i32,
                now.to_rfc3339(),
                id
            ],
        )?;

        self.get_template(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated template"))
    }

    /// Toggle favorite status of a template
    pub fn toggle_template_favorite(&self, id: &str) -> Result<Template> {
        let current = self
            .get_template(id)?
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", id))?;

        self.update_template(id, None, None, Some(!current.is_favorite), None)
    }

    /// Set a template as default (unsets all others)
    pub fn set_template_default(&self, id: &str) -> Result<Template> {
        // Verify template exists
        self.get_template(id)?
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", id))?;

        // Unset all default templates
        self.conn.execute(
            "UPDATE templates SET is_default = 0 WHERE is_default = 1",
            [],
        )?;

        // Set this template as default
        self.update_template(id, None, None, None, Some(true))
    }

    /// Delete a template
    pub fn delete_template(&self, id: &str) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM templates WHERE id = ?1", params![id])?;

        if deleted == 0 {
            anyhow::bail!("Template not found: {}", id);
        }

        Ok(())
    }

    /// Count total templates
    pub fn count_templates(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM templates", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Mark a template as synced
    pub fn mark_template_synced(&self, id: &str) -> Result<()> {
        let now = Utc::now();

        let updated = self.conn.execute(
            "UPDATE templates SET synced_at = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), id],
        )?;

        if updated == 0 {
            anyhow::bail!("Template not found: {}", id);
        }

        Ok(())
    }

    // ===== PENDING OPERATIONS QUEUE =====

    /// Add a pending operation to the queue
    pub fn add_pending_operation(
        &self,
        operation_type: &str,
        entity_type: &str,
        entity_id: &str,
        payload: &str,
    ) -> Result<i64> {
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO pending_operations (operation_type, entity_type, entity_id, payload, created_at, retry_count)
             VALUES (?1, ?2, ?3, ?4, ?5, 0)",
            params![operation_type, entity_type, entity_id, payload, now.to_rfc3339()],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get all pending operations
    pub fn get_pending_operations(&self) -> Result<Vec<PendingOperationRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, operation_type, entity_type, entity_id, payload, created_at, retry_count
             FROM pending_operations
             ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(PendingOperationRow {
                id: row.get(0)?,
                operation_type: row.get(1)?,
                entity_type: row.get(2)?,
                entity_id: row.get(3)?,
                payload: row.get(4)?,
                created_at: row.get(5)?,
                retry_count: row.get(6)?,
            })
        })?;

        let mut operations = Vec::new();
        for row in rows {
            operations.push(row?);
        }

        Ok(operations)
    }

    /// Remove a pending operation by ID
    pub fn remove_pending_operation(&self, id: i64) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM pending_operations WHERE id = ?1", params![id])?;

        if deleted == 0 {
            anyhow::bail!("Pending operation not found: {}", id);
        }

        Ok(())
    }

    /// Increment retry count for a pending operation
    pub fn increment_pending_operation_retry(&self, id: i64) -> Result<()> {
        let updated = self.conn.execute(
            "UPDATE pending_operations SET retry_count = retry_count + 1 WHERE id = ?1",
            params![id],
        )?;

        if updated == 0 {
            anyhow::bail!("Pending operation not found: {}", id);
        }

        Ok(())
    }

    /// Count pending operations
    pub fn count_pending_operations(&self) -> Result<i64> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM pending_operations", [], |row| {
                    row.get(0)
                })?;
        Ok(count)
    }

    // ===== SMART LABELS CRUD OPERATIONS =====

    /// Insert a new smart label
    pub fn insert_smart_label(&self, label: &InsertSmartLabel) -> Result<SmartLabel> {
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO smart_labels (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![&label.id, &label.name, &label.color, now.to_rfc3339()],
        )?;

        self.get_smart_label(&label.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted smart label"))
    }

    /// Get a smart label by ID
    pub fn get_smart_label(&self, id: &str) -> Result<Option<SmartLabel>> {
        let label = self
            .conn
            .query_row(
                "SELECT id, name, color, created_at FROM smart_labels WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()?;

        match label {
            Some((id, name, color, created_at)) => Ok(Some(SmartLabel {
                id,
                name,
                color,
                created_at: parse_datetime(created_at)?,
            })),
            None => Ok(None),
        }
    }

    /// List all smart labels
    pub fn list_smart_labels(&self) -> Result<Vec<SmartLabel>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, color, created_at FROM smart_labels ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        let mut labels = Vec::new();
        for row in rows {
            let (id, name, color, created_at) = row?;
            labels.push(SmartLabel {
                id,
                name,
                color,
                created_at: parse_datetime(created_at)?,
            });
        }

        Ok(labels)
    }

    /// Update a smart label
    pub fn update_smart_label(&self, id: &str, update: &UpdateSmartLabel) -> Result<SmartLabel> {
        let mut update_parts = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref name) = update.name {
            update_parts.push("name = ?");
            params_vec.push(Box::new(name.clone()));
        }
        if update.color.is_some() {
            update_parts.push("color = ?");
            params_vec.push(Box::new(update.color.clone()));
        }

        if update_parts.is_empty() {
            return self
                .get_smart_label(id)?
                .ok_or_else(|| anyhow::anyhow!("Smart label not found"));
        }

        params_vec.push(Box::new(id.to_string()));

        let sql = format!(
            "UPDATE smart_labels SET {} WHERE id = ?",
            update_parts.join(", ")
        );

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();
        let updated = self.conn.execute(&sql, params_refs.as_slice())?;

        if updated == 0 {
            anyhow::bail!("Smart label not found: {}", id);
        }

        self.get_smart_label(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated smart label"))
    }

    /// Delete a smart label
    pub fn delete_smart_label(&self, id: &str) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM smart_labels WHERE id = ?1", params![id])?;

        if deleted == 0 {
            anyhow::bail!("Smart label not found: {}", id);
        }

        Ok(())
    }

    /// Count total smart labels
    pub fn count_smart_labels(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM smart_labels", [], |row| row.get(0))?;
        Ok(count)
    }

    // ===== VOCABULARY CRUD OPERATIONS =====

    /// Insert a new vocabulary word
    pub fn insert_vocabulary(&self, vocab: &InsertVocabulary) -> Result<Vocabulary> {
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO vocabulary (id, word, pronunciation, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                &vocab.id,
                &vocab.word,
                &vocab.pronunciation,
                now.to_rfc3339()
            ],
        )?;

        self.get_vocabulary(&vocab.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted vocabulary"))
    }

    /// Get a vocabulary word by ID
    pub fn get_vocabulary(&self, id: &str) -> Result<Option<Vocabulary>> {
        let vocab = self
            .conn
            .query_row(
                "SELECT id, word, pronunciation, created_at FROM vocabulary WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()?;

        match vocab {
            Some((id, word, pronunciation, created_at)) => Ok(Some(Vocabulary {
                id,
                word,
                pronunciation,
                created_at: parse_datetime(created_at)?,
            })),
            None => Ok(None),
        }
    }

    /// List all vocabulary words
    pub fn list_vocabulary(&self) -> Result<Vec<Vocabulary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, word, pronunciation, created_at FROM vocabulary ORDER BY word ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        let mut words = Vec::new();
        for row in rows {
            let (id, word, pronunciation, created_at) = row?;
            words.push(Vocabulary {
                id,
                word,
                pronunciation,
                created_at: parse_datetime(created_at)?,
            });
        }

        Ok(words)
    }

    /// Delete a vocabulary word
    pub fn delete_vocabulary(&self, id: &str) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM vocabulary WHERE id = ?1", params![id])?;

        if deleted == 0 {
            anyhow::bail!("Vocabulary not found: {}", id);
        }

        Ok(())
    }

    /// Count total vocabulary words
    pub fn count_vocabulary(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM vocabulary", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Export vocabulary as JSON array
    pub fn export_vocabulary_json(&self) -> Result<String> {
        let words = self.list_vocabulary()?;
        serde_json::to_string_pretty(&words)
            .map_err(|e| anyhow::anyhow!("Failed to serialize vocabulary: {}", e))
    }

    /// Import vocabulary from JSON array
    pub fn import_vocabulary_json(&self, json: &str) -> Result<usize> {
        let words: Vec<InsertVocabulary> = serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to parse vocabulary JSON: {}", e))?;

        let mut imported_count = 0;
        for word in words {
            // Skip if word already exists (based on unique constraint)
            match self.insert_vocabulary(&word) {
                Ok(_) => imported_count += 1,
                Err(e) => {
                    // Log error but continue with other words
                    eprintln!("Failed to import word '{}': {}", word.word, e);
                }
            }
        }

        Ok(imported_count)
    }

    // ===== DEVICE CRUD OPERATIONS =====

    /// Insert a new device
    pub fn insert_device(&self, device: &Device) -> Result<Device> {
        self.conn.execute(
            "INSERT INTO devices (id, name, status, last_sync, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &device.id,
                &device.name,
                device.status.as_str(),
                device.last_sync.as_ref().map(|dt| dt.to_rfc3339()),
                device.created_at.to_rfc3339(),
            ],
        )?;

        self.get_device(&device.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted device"))
    }

    /// Get a device by ID
    pub fn get_device(&self, device_id: &str) -> Result<Option<Device>> {
        let device = self
            .conn
            .query_row(
                "SELECT id, name, status, last_sync, created_at FROM devices WHERE id = ?1",
                params![device_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()?;

        match device {
            Some((id, name, status, last_sync, created_at)) => Ok(Some(Device {
                id,
                name,
                status: DeviceStatus::from_str(&status)
                    .map_err(|e| anyhow::anyhow!("Invalid device status: {}", e))?,
                last_sync: parse_datetime_opt(last_sync)?,
                created_at: parse_datetime(created_at)?,
            })),
            None => Ok(None),
        }
    }

    /// List all devices
    pub fn list_devices(&self) -> Result<Vec<Device>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, status, last_sync, created_at FROM devices ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        let mut devices = Vec::new();
        for row in rows {
            let (id, name, status, last_sync, created_at) = row?;
            devices.push(Device {
                id,
                name,
                status: DeviceStatus::from_str(&status)
                    .map_err(|e| anyhow::anyhow!("Invalid device status: {}", e))?,
                last_sync: parse_datetime_opt(last_sync)?,
                created_at: parse_datetime(created_at)?,
            });
        }

        Ok(devices)
    }

    /// Update device status
    pub fn update_device_status(&self, device_id: &str, status: DeviceStatus) -> Result<Device> {
        let updated = self.conn.execute(
            "UPDATE devices SET status = ?1 WHERE id = ?2",
            params![status.as_str(), device_id],
        )?;

        if updated == 0 {
            anyhow::bail!("Device not found: {}", device_id);
        }

        self.get_device(device_id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated device"))
    }

    /// Delete a device
    pub fn delete_device(&self, device_id: &str) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM devices WHERE id = ?1", params![device_id])?;

        if deleted == 0 {
            anyhow::bail!("Device not found: {}", device_id);
        }

        Ok(())
    }

    /// Update device last sync time
    pub fn update_device_last_sync(&self, device_id: &str) -> Result<Device> {
        let now = Utc::now();

        let updated = self.conn.execute(
            "UPDATE devices SET last_sync = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), device_id],
        )?;

        if updated == 0 {
            anyhow::bail!("Device not found: {}", device_id);
        }

        self.get_device(device_id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated device"))
    }

    /// Count total devices
    pub fn count_devices(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM devices", [], |row| row.get(0))?;
        Ok(count)
    }

    // ===== SHARE LINKS CRUD OPERATIONS =====

    /// Insert a new share link
    pub fn insert_share_link(&self, share: &InsertShareLink) -> Result<ShareLink> {
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO share_links (id, note_id, token, expires_at, created_at, last_accessed_at, access_count)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL, 0)",
            params![
                &share.id,
                &share.note_id,
                &share.token,
                share.expires_at.as_ref().map(|dt| dt.to_rfc3339()),
                now.to_rfc3339(),
            ],
        )?;

        self.get_share_link(&share.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted share link"))
    }

    /// Get a share link by ID
    pub fn get_share_link(&self, id: &str) -> Result<Option<ShareLink>> {
        let share = self
            .conn
            .query_row(
                "SELECT id, note_id, token, expires_at, created_at, last_accessed_at, access_count
                 FROM share_links WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, i64>(6)?,
                    ))
                },
            )
            .optional()?;

        match share {
            Some((id, note_id, token, expires_at, created_at, last_accessed_at, access_count)) => {
                Ok(Some(ShareLink {
                    id,
                    note_id,
                    token,
                    expires_at: parse_datetime_opt(expires_at)?,
                    created_at: parse_datetime(created_at)?,
                    last_accessed_at: parse_datetime_opt(last_accessed_at)?,
                    access_count,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get a share link by token
    pub fn get_share_link_by_token(&self, token: &str) -> Result<Option<ShareLink>> {
        let share = self
            .conn
            .query_row(
                "SELECT id, note_id, token, expires_at, created_at, last_accessed_at, access_count
                 FROM share_links WHERE token = ?1",
                params![token],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, i64>(6)?,
                    ))
                },
            )
            .optional()?;

        match share {
            Some((id, note_id, token, expires_at, created_at, last_accessed_at, access_count)) => {
                Ok(Some(ShareLink {
                    id,
                    note_id,
                    token,
                    expires_at: parse_datetime_opt(expires_at)?,
                    created_at: parse_datetime(created_at)?,
                    last_accessed_at: parse_datetime_opt(last_accessed_at)?,
                    access_count,
                }))
            }
            None => Ok(None),
        }
    }

    /// List share links for a note
    pub fn list_share_links_by_note(&self, note_id: &str) -> Result<Vec<ShareLink>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, note_id, token, expires_at, created_at, last_accessed_at, access_count
             FROM share_links
             WHERE note_id = ?1
             ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map(params![note_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })?;

        let mut shares = Vec::new();
        for row in rows {
            let (id, note_id, token, expires_at, created_at, last_accessed_at, access_count) = row?;
            shares.push(ShareLink {
                id,
                note_id,
                token,
                expires_at: parse_datetime_opt(expires_at)?,
                created_at: parse_datetime(created_at)?,
                last_accessed_at: parse_datetime_opt(last_accessed_at)?,
                access_count,
            });
        }

        Ok(shares)
    }

    /// Delete a share link
    pub fn delete_share_link(&self, id: &str) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM share_links WHERE id = ?1", params![id])?;

        if deleted == 0 {
            anyhow::bail!("Share link not found: {}", id);
        }

        Ok(())
    }

    /// Increment share link access count and update last accessed time
    pub fn increment_share_access(&self, id: &str) -> Result<()> {
        let now = Utc::now();

        let updated = self.conn.execute(
            "UPDATE share_links SET access_count = access_count + 1, last_accessed_at = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), id],
        )?;

        if updated == 0 {
            anyhow::bail!("Share link not found: {}", id);
        }

        Ok(())
    }

    /// Delete expired share links
    pub fn delete_expired_share_links(&self) -> Result<usize> {
        let now = Utc::now();

        let deleted = self.conn.execute(
            "DELETE FROM share_links WHERE expires_at IS NOT NULL AND expires_at < ?1",
            params![now.to_rfc3339()],
        )?;

        Ok(deleted)
    }

    // ===== SPEAKER CRUD OPERATIONS =====

    /// Insert a new speaker
    pub fn insert_speaker(&self, speaker: &InsertSpeaker) -> Result<Speaker> {
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO speakers (id, name, voice_signature, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &speaker.id,
                &speaker.name,
                &speaker.voice_signature,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        self.get_speaker(&speaker.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted speaker"))
    }

    /// Get a speaker by ID
    pub fn get_speaker(&self, id: &str) -> Result<Option<Speaker>> {
        let speaker = self
            .conn
            .query_row(
                "SELECT id, name, voice_signature, created_at, updated_at FROM speakers WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()?;

        match speaker {
            Some((id, name, voice_signature, created_at, updated_at)) => Ok(Some(Speaker {
                id,
                name,
                voice_signature,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
            })),
            None => Ok(None),
        }
    }

    /// List all speakers
    pub fn list_speakers(&self) -> Result<Vec<Speaker>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, voice_signature, created_at, updated_at FROM speakers ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        let mut speakers = Vec::new();
        for row in rows {
            let (id, name, voice_signature, created_at, updated_at) = row?;
            speakers.push(Speaker {
                id,
                name,
                voice_signature,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
            });
        }

        Ok(speakers)
    }

    /// Update a speaker
    pub fn update_speaker(&self, id: &str, update: &UpdateSpeaker) -> Result<Speaker> {
        let now = Utc::now();
        let mut update_parts = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if update.name.is_some() {
            update_parts.push("name = ?");
            params_vec.push(Box::new(update.name.clone()));
        }
        if update.voice_signature.is_some() {
            update_parts.push("voice_signature = ?");
            params_vec.push(Box::new(update.voice_signature.clone()));
        }

        if update_parts.is_empty() {
            return self
                .get_speaker(id)?
                .ok_or_else(|| anyhow::anyhow!("Speaker not found"));
        }

        update_parts.push("updated_at = ?");
        params_vec.push(Box::new(now.to_rfc3339()));
        params_vec.push(Box::new(id.to_string()));

        let sql = format!(
            "UPDATE speakers SET {} WHERE id = ?",
            update_parts.join(", ")
        );

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();
        let updated = self.conn.execute(&sql, params_refs.as_slice())?;

        if updated == 0 {
            anyhow::bail!("Speaker not found: {}", id);
        }

        self.get_speaker(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated speaker"))
    }

    /// Delete a speaker
    pub fn delete_speaker(&self, id: &str) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM speakers WHERE id = ?1", params![id])?;

        if deleted == 0 {
            anyhow::bail!("Speaker not found: {}", id);
        }

        Ok(())
    }

    /// Count total speakers
    pub fn count_speakers(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM speakers", [], |row| row.get(0))?;
        Ok(count)
    }

    // ===== SPEAKER SEGMENT CRUD OPERATIONS =====

    /// Insert a new speaker segment
    pub fn insert_speaker_segment(&self, segment: &InsertSpeakerSegment) -> Result<SpeakerSegment> {
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO speaker_segments (id, note_id, speaker_id, start_time, end_time, confidence, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &segment.id,
                &segment.note_id,
                &segment.speaker_id,
                segment.start_time,
                segment.end_time,
                segment.confidence,
                now.to_rfc3339(),
            ],
        )?;

        self.get_speaker_segment(&segment.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted speaker segment"))
    }

    /// Get a speaker segment by ID
    pub fn get_speaker_segment(&self, id: &str) -> Result<Option<SpeakerSegment>> {
        let segment = self
            .conn
            .query_row(
                "SELECT id, note_id, speaker_id, start_time, end_time, confidence, created_at
                 FROM speaker_segments WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, f64>(4)?,
                        row.get::<_, f64>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                },
            )
            .optional()?;

        match segment {
            Some((id, note_id, speaker_id, start_time, end_time, confidence, created_at)) => {
                Ok(Some(SpeakerSegment {
                    id,
                    note_id,
                    speaker_id,
                    start_time,
                    end_time,
                    confidence,
                    created_at: parse_datetime(created_at)?,
                }))
            }
            None => Ok(None),
        }
    }

    /// List all speaker segments for a note
    pub fn list_speaker_segments_by_note(&self, note_id: &str) -> Result<Vec<SpeakerSegment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, note_id, speaker_id, start_time, end_time, confidence, created_at
             FROM speaker_segments
             WHERE note_id = ?1
             ORDER BY start_time ASC",
        )?;

        let rows = stmt.query_map(params![note_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, f64>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;

        let mut segments = Vec::new();
        for row in rows {
            let (id, note_id, speaker_id, start_time, end_time, confidence, created_at) = row?;
            segments.push(SpeakerSegment {
                id,
                note_id,
                speaker_id,
                start_time,
                end_time,
                confidence,
                created_at: parse_datetime(created_at)?,
            });
        }

        Ok(segments)
    }

    /// List all speaker segments for a specific speaker
    pub fn list_speaker_segments_by_speaker(
        &self,
        speaker_id: &str,
    ) -> Result<Vec<SpeakerSegment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, note_id, speaker_id, start_time, end_time, confidence, created_at
             FROM speaker_segments
             WHERE speaker_id = ?1
             ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map(params![speaker_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, f64>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;

        let mut segments = Vec::new();
        for row in rows {
            let (id, note_id, speaker_id, start_time, end_time, confidence, created_at) = row?;
            segments.push(SpeakerSegment {
                id,
                note_id,
                speaker_id,
                start_time,
                end_time,
                confidence,
                created_at: parse_datetime(created_at)?,
            });
        }

        Ok(segments)
    }

    /// Delete a speaker segment
    pub fn delete_speaker_segment(&self, id: &str) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM speaker_segments WHERE id = ?1", params![id])?;

        if deleted == 0 {
            anyhow::bail!("Speaker segment not found: {}", id);
        }

        Ok(())
    }

    /// Delete all speaker segments for a note
    pub fn delete_speaker_segments_by_note(&self, note_id: &str) -> Result<usize> {
        let deleted = self.conn.execute(
            "DELETE FROM speaker_segments WHERE note_id = ?1",
            params![note_id],
        )?;

        Ok(deleted)
    }

    /// Count total speaker segments
    pub fn count_speaker_segments(&self) -> Result<i64> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM speaker_segments", [], |row| {
                    row.get(0)
                })?;
        Ok(count)
    }

    /// Count speaker segments for a specific note
    pub fn count_speaker_segments_by_note(&self, note_id: &str) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM speaker_segments WHERE note_id = ?1",
            params![note_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Get speaker statistics for a note
    pub fn get_speaker_statistics(&self, note_id: &str) -> Result<Vec<(String, f64, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT speaker_id, SUM(end_time - start_time) as talk_time, COUNT(*) as turn_count
             FROM speaker_segments
             WHERE note_id = ?1
             GROUP BY speaker_id
             ORDER BY talk_time DESC",
        )?;

        let rows = stmt.query_map(params![note_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(row?);
        }

        Ok(stats)
    }

    /// Update speaker assignment for a segment
    pub fn assign_speaker_to_segment(
        &self,
        segment_id: &str,
        speaker_id: &str,
    ) -> Result<SpeakerSegment> {
        // Verify speaker exists
        self.get_speaker(speaker_id)?
            .ok_or_else(|| anyhow::anyhow!("Speaker not found: {}", speaker_id))?;

        let updated = self.conn.execute(
            "UPDATE speaker_segments SET speaker_id = ?1 WHERE id = ?2",
            params![speaker_id, segment_id],
        )?;

        if updated == 0 {
            anyhow::bail!("Speaker segment not found: {}", segment_id);
        }

        self.get_speaker_segment(segment_id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated speaker segment"))
    }

    /// Compare two voice signatures and return similarity score (0.0 to 1.0)
    /// Returns None if either signature is None
    /// Currently uses simple string comparison - in production this would use
    /// acoustic feature comparison (MFCCs, spectrograms, embeddings, etc.)
    pub fn compare_voice_signatures(
        &self,
        signature1: Option<&str>,
        signature2: Option<&str>,
    ) -> Option<f64> {
        match (signature1, signature2) {
            (Some(sig1), Some(sig2)) => {
                // Simple placeholder implementation - string similarity
                // In production, this would parse acoustic features and compute similarity
                if sig1 == sig2 {
                    Some(1.0)
                } else {
                    // Basic edit distance-based similarity
                    let distance = Self::levenshtein_distance(sig1, sig2);
                    let max_len = sig1.len().max(sig2.len()) as f64;
                    if max_len == 0.0 {
                        Some(1.0)
                    } else {
                        Some(1.0 - (distance as f64 / max_len))
                    }
                }
            }
            _ => None,
        }
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }

    /// Find the best matching speaker for a given voice signature
    /// Returns (speaker_id, similarity_score) if a match above threshold is found
    pub fn find_matching_speaker(
        &self,
        voice_signature: &str,
        threshold: f64,
    ) -> Result<Option<(String, f64)>> {
        let speakers = self.list_speakers()?;

        let mut best_match: Option<(String, f64)> = None;

        for speaker in speakers {
            if let Some(similarity) = self
                .compare_voice_signatures(Some(voice_signature), speaker.voice_signature.as_deref())
            {
                if similarity >= threshold {
                    match best_match {
                        Some((_, best_score)) if similarity > best_score => {
                            best_match = Some((speaker.id.clone(), similarity));
                        }
                        None => {
                            best_match = Some((speaker.id.clone(), similarity));
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(best_match)
    }

    /// Auto-assign speakers to segments based on voice signature matching
    /// Returns the number of segments that were assigned
    pub fn auto_assign_speakers_to_segments(
        &self,
        note_id: &str,
        signature_map: &[(String, String)], // (segment_id, voice_signature)
        threshold: f64,
    ) -> Result<usize> {
        let mut assigned_count = 0;

        for (segment_id, voice_signature) in signature_map {
            if let Some((speaker_id, _similarity)) =
                self.find_matching_speaker(voice_signature, threshold)?
            {
                if self
                    .assign_speaker_to_segment(segment_id, &speaker_id)
                    .is_ok()
                {
                    assigned_count += 1;
                }
            }
        }

        Ok(assigned_count)
    }

    /// Alias method for backward compatibility: list_speaker_segments_for_note
    pub fn list_speaker_segments_for_note(&self, note_id: &str) -> Result<Vec<SpeakerSegment>> {
        self.list_speaker_segments_by_note(note_id)
    }

    /// Update the speaker assignment for all segments of a note from one speaker to another
    /// Used when merging speakers
    pub fn update_segment_speaker(
        &self,
        note_id: &str,
        old_speaker_id: &str,
        new_speaker_id: &str,
    ) -> Result<usize> {
        let updated = self.conn.execute(
            "UPDATE speaker_segments SET speaker_id = ?1 WHERE note_id = ?2 AND speaker_id = ?3",
            params![new_speaker_id, note_id, old_speaker_id],
        )?;

        Ok(updated)
    }

    /// Count total segments for a specific speaker across all notes
    pub fn count_segments_for_speaker(&self, speaker_id: &str) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM speaker_segments WHERE speaker_id = ?1",
            params![speaker_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Update the end time of a speaker segment
    pub fn update_segment_end_time(
        &self,
        segment_id: &str,
        end_time: f64,
    ) -> Result<SpeakerSegment> {
        let updated = self.conn.execute(
            "UPDATE speaker_segments SET end_time = ?1 WHERE id = ?2",
            params![end_time, segment_id],
        )?;

        if updated == 0 {
            anyhow::bail!("Speaker segment not found: {}", segment_id);
        }

        self.get_speaker_segment(segment_id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated speaker segment"))
    }

    /// Alias for delete_speaker_segments_by_note for backward compatibility
    pub fn delete_segments_for_note(&self, note_id: &str) -> Result<usize> {
        self.delete_speaker_segments_by_note(note_id)
    }

    // ===== USER SETTINGS CRUD OPERATIONS =====

    /// Set a user setting (key-value pair)
    pub fn set_user_setting(&self, key: &str, value: &str) -> Result<()> {
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO user_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3",
            params![key, value, now.to_rfc3339()],
        )?;

        Ok(())
    }

    /// Get a user setting by key
    pub fn get_user_setting(&self, key: &str) -> Result<Option<String>> {
        let value = self
            .conn
            .query_row(
                "SELECT value FROM user_settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?;

        Ok(value)
    }

    /// Delete a user setting
    pub fn delete_user_setting(&self, key: &str) -> Result<bool> {
        let deleted = self
            .conn
            .execute("DELETE FROM user_settings WHERE key = ?1", params![key])?;

        Ok(deleted > 0)
    }

    /// List all user settings
    pub fn list_user_settings(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM user_settings ORDER BY key ASC")?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut settings = Vec::new();
        for row in rows {
            settings.push(row?);
        }

        Ok(settings)
    }

    // ===== SUBSCRIPTION CRUD OPERATIONS =====

    /// Insert a new subscription
    pub fn insert_subscription(&self, sub: &InsertSubscription) -> Result<DbSubscription> {
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO subscriptions (product_id, status, expires_at, purchased_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &sub.product_id,
                sub.status.as_str(),
                sub.expires_at.as_ref().map(|dt| dt.to_rfc3339()),
                sub.purchased_at.as_ref().map(|dt| dt.to_rfc3339()),
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        let id = self.conn.last_insert_rowid();
        self.get_subscription(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted subscription"))
    }

    /// Get a subscription by ID
    pub fn get_subscription(&self, id: i64) -> Result<Option<DbSubscription>> {
        let sub = self
            .conn
            .query_row(
                "SELECT id, product_id, status, expires_at, purchased_at, canceled_at, created_at, updated_at
                 FROM subscriptions WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                    ))
                },
            )
            .optional()?;

        match sub {
            Some((
                id,
                product_id,
                status,
                expires_at,
                purchased_at,
                canceled_at,
                created_at,
                updated_at,
            )) => Ok(Some(DbSubscription {
                id,
                product_id,
                status: SubscriptionStatus::from_str(&status)
                    .map_err(|e| anyhow::anyhow!("Invalid status: {}", e))?,
                expires_at: parse_datetime_opt(expires_at)?,
                purchased_at: parse_datetime_opt(purchased_at)?,
                canceled_at: parse_datetime_opt(canceled_at)?,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
            })),
            None => Ok(None),
        }
    }

    /// Get the current (most recent) subscription
    pub fn get_current_subscription(&self) -> Result<Option<DbSubscription>> {
        let sub = self
            .conn
            .query_row(
                "SELECT id, product_id, status, expires_at, purchased_at, canceled_at, created_at, updated_at
                 FROM subscriptions ORDER BY created_at DESC LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                    ))
                },
            )
            .optional()?;

        match sub {
            Some((
                id,
                product_id,
                status,
                expires_at,
                purchased_at,
                canceled_at,
                created_at,
                updated_at,
            )) => Ok(Some(DbSubscription {
                id,
                product_id,
                status: SubscriptionStatus::from_str(&status)
                    .map_err(|e| anyhow::anyhow!("Invalid status: {}", e))?,
                expires_at: parse_datetime_opt(expires_at)?,
                purchased_at: parse_datetime_opt(purchased_at)?,
                canceled_at: parse_datetime_opt(canceled_at)?,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
            })),
            None => Ok(None),
        }
    }

    /// Update a subscription
    pub fn update_subscription(
        &self,
        id: i64,
        update: &UpdateSubscription,
    ) -> Result<DbSubscription> {
        let now = Utc::now();
        let mut update_parts = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref status) = update.status {
            update_parts.push("status = ?");
            params_vec.push(Box::new(status.as_str().to_string()));
        }
        if update.expires_at.is_some() {
            update_parts.push("expires_at = ?");
            params_vec.push(Box::new(update.expires_at.map(|dt| dt.to_rfc3339())));
        }
        if update.canceled_at.is_some() {
            update_parts.push("canceled_at = ?");
            params_vec.push(Box::new(update.canceled_at.map(|dt| dt.to_rfc3339())));
        }

        // Always update updated_at
        update_parts.push("updated_at = ?");
        params_vec.push(Box::new(now.to_rfc3339()));

        // Add id as last param
        params_vec.push(Box::new(id));

        let sql = format!(
            "UPDATE subscriptions SET {} WHERE id = ?",
            update_parts.join(", ")
        );

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();
        let updated = self.conn.execute(&sql, params_refs.as_slice())?;

        if updated == 0 {
            anyhow::bail!("Subscription not found: {}", id);
        }

        self.get_subscription(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated subscription"))
    }

    /// List all subscriptions
    pub fn list_subscriptions(&self, limit: i64) -> Result<Vec<DbSubscription>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, product_id, status, expires_at, purchased_at, canceled_at, created_at, updated_at
             FROM subscriptions ORDER BY created_at DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })?;

        let mut subscriptions = Vec::new();
        for row in rows {
            let (
                id,
                product_id,
                status,
                expires_at,
                purchased_at,
                canceled_at,
                created_at,
                updated_at,
            ) = row?;
            subscriptions.push(DbSubscription {
                id,
                product_id,
                status: SubscriptionStatus::from_str(&status)
                    .map_err(|e| anyhow::anyhow!("Invalid status: {}", e))?,
                expires_at: parse_datetime_opt(expires_at)?,
                purchased_at: parse_datetime_opt(purchased_at)?,
                canceled_at: parse_datetime_opt(canceled_at)?,
                created_at: parse_datetime(created_at)?,
                updated_at: parse_datetime(updated_at)?,
            });
        }

        Ok(subscriptions)
    }

    // ===== SUBSCRIPTION EVENTS CRUD OPERATIONS =====

    /// Insert a new subscription event
    pub fn insert_subscription_event(&self, event: &InsertSubscriptionEvent) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO subscription_events (subscription_id, event_type, product_id, expires_at, occurred_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                event.subscription_id,
                event.event_type.as_str(),
                &event.product_id,
                event.expires_at.as_ref().map(|dt| dt.to_rfc3339()),
                event.occurred_at.to_rfc3339(),
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// List subscription events
    pub fn list_subscription_events(&self, limit: i64) -> Result<Vec<SubscriptionEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, subscription_id, event_type, product_id, expires_at, occurred_at
             FROM subscription_events ORDER BY occurred_at DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;

        let mut events = Vec::new();
        for row in rows {
            let (id, subscription_id, event_type, product_id, expires_at, occurred_at) = row?;
            events.push(SubscriptionEvent {
                id,
                subscription_id,
                event_type: SubscriptionEventType::from_str(&event_type)
                    .map_err(|e| anyhow::anyhow!("Invalid event type: {}", e))?,
                product_id,
                expires_at: parse_datetime_opt(expires_at)?,
                occurred_at: parse_datetime(occurred_at)?,
            });
        }

        Ok(events)
    }

    // ===== REFERRAL CODE CRUD OPERATIONS =====

    /// Generate a new referral code for a user
    pub fn generate_referral_code(&self, user_id: &str) -> Result<crate::referral::ReferralCode> {
        self.generate_referral_code_with_expiry(user_id, None)
    }

    /// Generate a new referral code with optional expiry date
    pub fn generate_referral_code_with_expiry(
        &self,
        user_id: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<crate::referral::ReferralCode> {
        use uuid::Uuid;

        let id = Uuid::new_v4().to_string();
        let code = crate::referral::generator::generate_code();
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO referral_codes (id, user_id, code, created_at, expires_at, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)",
            params![
                &id,
                user_id,
                &code,
                now.to_rfc3339(),
                expires_at.as_ref().map(|dt| dt.to_rfc3339()),
            ],
        )?;

        Ok(crate::referral::ReferralCode {
            id,
            user_id: user_id.to_string(),
            code,
            created_at: now,
            expires_at,
        })
    }

    /// Validate if a referral code exists and is active
    pub fn validate_referral_code(&self, code: &str) -> Result<bool> {
        let now = Utc::now();

        let result: Option<i32> = self
            .conn
            .query_row(
                "SELECT 1 FROM referral_codes
                 WHERE code = ?1
                   AND is_active = 1
                   AND (expires_at IS NULL OR expires_at > ?2)",
                params![code, now.to_rfc3339()],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result.is_some())
    }

    /// Get referral code details by code string
    fn get_referral_code_by_code(
        &self,
        code: &str,
    ) -> Result<Option<crate::referral::ReferralCode>> {
        let result = self
            .conn
            .query_row(
                "SELECT id, user_id, code, created_at, expires_at
                 FROM referral_codes
                 WHERE code = ?1 AND is_active = 1",
                params![code],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .optional()?;

        match result {
            Some((id, user_id, code, created_at, expires_at)) => {
                Ok(Some(crate::referral::ReferralCode {
                    id,
                    user_id,
                    code,
                    created_at: parse_datetime(created_at)?,
                    expires_at: parse_datetime_opt(expires_at)?,
                }))
            }
            None => Ok(None),
        }
    }

    /// Apply a referral code for a new user
    pub fn apply_referral_code(
        &self,
        referred_user_id: &str,
        code: &str,
        reward_config: &crate::referral::RewardConfig,
    ) -> Result<crate::referral::ReferralUsage> {
        let now = Utc::now();

        // Validate the code
        if !self.validate_referral_code(code)? {
            anyhow::bail!("Invalid or expired referral code");
        }

        // Get the referral code details
        let referral_code = self
            .get_referral_code_by_code(code)?
            .ok_or_else(|| anyhow::anyhow!("Referral code not found"))?;

        // Check for self-referral
        if referral_code.user_id == referred_user_id {
            anyhow::bail!("User cannot refer yourself");
        }

        // Check if user has already used a referral code
        let already_used: Option<i32> = self
            .conn
            .query_row(
                "SELECT 1 FROM referral_usage WHERE referred_user_id = ?1",
                params![referred_user_id],
                |row| row.get(0),
            )
            .optional()?;

        if already_used.is_some() {
            anyhow::bail!("User has already used a referral code");
        }

        // Create the referral usage record
        self.conn.execute(
            "INSERT INTO referral_usage (code_id, referred_user_id, referrer_user_id, applied_at, reward_points, reward_credits, reward_subscription_days)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &referral_code.id,
                referred_user_id,
                &referral_code.user_id,
                now.to_rfc3339(),
                reward_config.points,
                reward_config.credits,
                reward_config.subscription_days,
            ],
        )?;

        let id = self.conn.last_insert_rowid();

        Ok(crate::referral::ReferralUsage {
            id,
            code_id: referral_code.id,
            referred_user_id: referred_user_id.to_string(),
            referrer_user_id: referral_code.user_id,
            applied_at: now,
            reward_points: reward_config.points,
            reward_credits: reward_config.credits,
            reward_subscription_days: reward_config.subscription_days,
        })
    }

    /// Get referral statistics for a user
    pub fn get_referral_stats(&self, user_id: &str) -> Result<crate::referral::ReferralStats> {
        // Count total referrals
        let total_referrals: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM referral_usage WHERE referrer_user_id = ?1",
            params![user_id],
            |row| row.get(0),
        )?;

        // Get aggregated rewards
        let (total_points, total_credits, total_days): (Option<i64>, Option<i64>, Option<i64>) =
            self.conn.query_row(
                "SELECT
                    COALESCE(SUM(reward_points), 0),
                    COALESCE(SUM(reward_credits), 0),
                    COALESCE(SUM(reward_subscription_days), 0)
                 FROM referral_usage
                 WHERE referrer_user_id = ?1",
                params![user_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;

        // Get referral chain (list of referred users)
        let mut stmt = self.conn.prepare(
            "SELECT referred_user_id FROM referral_usage WHERE referrer_user_id = ?1 ORDER BY applied_at DESC",
        )?;

        let rows = stmt.query_map(params![user_id], |row| row.get::<_, String>(0))?;

        let mut referral_chain = Vec::new();
        for row in rows {
            referral_chain.push(row?);
        }

        Ok(crate::referral::ReferralStats {
            total_referrals,
            active_referrals: total_referrals, // All referrals are active (not tracking deactivations yet)
            total_reward_points: total_points.unwrap_or(0) as i32,
            total_reward_credits: total_credits.unwrap_or(0) as i32,
            total_subscription_days: total_days.unwrap_or(0) as i32,
            referral_chain,
        })
    }

    /// List all referral codes for a user
    pub fn list_user_referral_codes(
        &self,
        user_id: &str,
    ) -> Result<Vec<crate::referral::ReferralCode>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, code, created_at, expires_at
             FROM referral_codes
             WHERE user_id = ?1 AND is_active = 1
             ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map(params![user_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })?;

        let mut codes = Vec::new();
        for row in rows {
            let (id, user_id, code, created_at, expires_at) = row?;
            codes.push(crate::referral::ReferralCode {
                id,
                user_id,
                code,
                created_at: parse_datetime(created_at)?,
                expires_at: parse_datetime_opt(expires_at)?,
            });
        }

        Ok(codes)
    }

    /// Delete a referral code (deactivate)
    pub fn delete_referral_code(&self, code_id: &str) -> Result<()> {
        let updated = self.conn.execute(
            "UPDATE referral_codes SET is_active = 0 WHERE id = ?1",
            params![code_id],
        )?;

        if updated == 0 {
            anyhow::bail!("Referral code not found: {}", code_id);
        }

        Ok(())
    }
}

/// Row data for pending operations
#[derive(Debug)]
pub struct PendingOperationRow {
    pub id: i64,
    pub operation_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub payload: String,
    pub created_at: String,
    pub retry_count: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Database {
        Database::new_in_memory().expect("Failed to create in-memory database")
    }

    #[test]
    fn test_insert_and_get_folder() {
        let db = setup();

        let folder = db.insert_folder("test-id-1", "My Folder").unwrap();

        assert_eq!(folder.id, "test-id-1");
        assert_eq!(folder.name, "My Folder");
        assert!(folder.synced_at.is_none());

        let retrieved = db.get_folder("test-id-1").unwrap().unwrap();
        assert_eq!(retrieved, folder);
    }

    #[test]
    fn test_get_nonexistent_folder() {
        let db = setup();

        let result = db.get_folder("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_folders_empty() {
        let db = setup();

        let folders = db.list_folders(10, 0).unwrap();
        assert_eq!(folders.len(), 0);
    }

    #[test]
    fn test_list_folders_with_pagination() {
        let db = setup();

        // Insert 5 folders
        for i in 1..=5 {
            db.insert_folder(&format!("folder-{}", i), &format!("Folder {}", i))
                .unwrap();
        }

        // Get first page
        let page1 = db.list_folders(2, 0).unwrap();
        assert_eq!(page1.len(), 2);

        // Get second page
        let page2 = db.list_folders(2, 2).unwrap();
        assert_eq!(page2.len(), 2);

        // Get third page
        let page3 = db.list_folders(2, 4).unwrap();
        assert_eq!(page3.len(), 1);

        // Verify total count
        let count = db.count_folders().unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_update_folder() {
        let db = setup();

        let folder = db.insert_folder("test-id-1", "Original Name").unwrap();
        let original_updated_at = folder.updated_at;

        // Small delay to ensure updated_at changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        let updated = db.update_folder("test-id-1", "New Name").unwrap();

        assert_eq!(updated.id, "test-id-1");
        assert_eq!(updated.name, "New Name");
        assert!(updated.updated_at > original_updated_at);
        assert_eq!(updated.created_at, folder.created_at);
    }

    #[test]
    fn test_update_nonexistent_folder() {
        let db = setup();

        let result = db.update_folder("nonexistent", "New Name");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Folder not found"));
    }

    #[test]
    fn test_delete_folder() {
        let db = setup();

        db.insert_folder("test-id-1", "My Folder").unwrap();

        db.delete_folder("test-id-1").unwrap();

        let result = db.get_folder("test-id-1").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_nonexistent_folder() {
        let db = setup();

        let result = db.delete_folder("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Folder not found"));
    }

    #[test]
    fn test_mark_folder_synced() {
        let db = setup();

        let folder = db.insert_folder("test-id-1", "My Folder").unwrap();
        assert!(folder.synced_at.is_none());

        db.mark_folder_synced("test-id-1").unwrap();

        let synced_folder = db.get_folder("test-id-1").unwrap().unwrap();
        assert!(synced_folder.synced_at.is_some());
    }

    #[test]
    fn test_mark_nonexistent_folder_synced() {
        let db = setup();

        let result = db.mark_folder_synced("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Folder not found"));
    }

    #[test]
    fn test_folders_ordered_by_created_at_desc() {
        let db = setup();

        // Insert folders with delays to ensure different timestamps
        db.insert_folder("folder-1", "First").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        db.insert_folder("folder-2", "Second").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        db.insert_folder("folder-3", "Third").unwrap();

        let folders = db.list_folders(10, 0).unwrap();

        // Most recent first
        assert_eq!(folders[0].id, "folder-3");
        assert_eq!(folders[1].id, "folder-2");
        assert_eq!(folders[2].id, "folder-1");
    }

    // ===== NOTES TESTS =====

    #[test]
    fn test_insert_and_get_note() {
        let db = setup();

        let insert_note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: Some("Test content".to_string()),
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: Some(5),
        };

        let note = db.insert_note(&insert_note).unwrap();
        assert_eq!(note.id, "note-1");
        assert_eq!(note.title, "Test Note");
        assert_eq!(note.content, Some("Test content".to_string()));
        assert_eq!(note.rating, Some(5));
        assert!(note.synced_at.is_none());

        let retrieved = db.get_note("note-1").unwrap().unwrap();
        assert_eq!(retrieved, note);
    }

    #[test]
    fn test_get_nonexistent_note() {
        let db = setup();

        let result = db.get_note("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_notes_with_pagination() {
        let db = setup();

        // Insert 5 notes
        for i in 1..=5 {
            let insert_note = InsertNote {
                id: format!("note-{}", i),
                title: format!("Note {}", i),
                content: None,
                folder_id: None,
                audio_url: None,
                duration: None,
                rating: None,
            };
            db.insert_note(&insert_note).unwrap();
        }

        // Get first page
        let page1 = db.list_notes(2, 0).unwrap();
        assert_eq!(page1.len(), 2);

        // Get second page
        let page2 = db.list_notes(2, 2).unwrap();
        assert_eq!(page2.len(), 2);

        // Get third page
        let page3 = db.list_notes(2, 4).unwrap();
        assert_eq!(page3.len(), 1);

        // Verify total count
        let count = db.count_notes().unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_update_note() {
        let db = setup();

        let insert_note = InsertNote {
            id: "note-1".to_string(),
            title: "Original Title".to_string(),
            content: Some("Original content".to_string()),
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: Some(3),
        };

        db.insert_note(&insert_note).unwrap();

        let update = UpdateNote {
            title: Some("Updated Title".to_string()),
            content: Some("Updated content".to_string()),
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: Some(5),
        };

        let updated = db.update_note("note-1", &update).unwrap();
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.content, Some("Updated content".to_string()));
        assert_eq!(updated.rating, Some(5));
    }

    #[test]
    fn test_partial_update_note() {
        let db = setup();

        let insert_note = InsertNote {
            id: "note-1".to_string(),
            title: "Original".to_string(),
            content: Some("Content".to_string()),
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: Some(3),
        };

        db.insert_note(&insert_note).unwrap();

        // Update only title
        let update = UpdateNote {
            title: Some("New Title".to_string()),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };

        let updated = db.update_note("note-1", &update).unwrap();
        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.content, Some("Content".to_string())); // unchanged
        assert_eq!(updated.rating, Some(3)); // unchanged
    }

    #[test]
    fn test_update_nonexistent_note() {
        let db = setup();

        let update = UpdateNote {
            title: Some("New Title".to_string()),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };

        let result = db.update_note("nonexistent", &update);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Note not found"));
    }

    #[test]
    fn test_delete_note() {
        let db = setup();

        let insert_note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };

        db.insert_note(&insert_note).unwrap();
        assert!(db.get_note("note-1").unwrap().is_some());

        db.delete_note("note-1").unwrap();
        assert!(db.get_note("note-1").unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent_note() {
        let db = setup();

        let result = db.delete_note("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Note not found"));
    }

    #[test]
    fn test_notes_with_folders() {
        let db = setup();

        // Create folder
        let folder = db.insert_folder("folder-1", "Work").unwrap();
        assert_eq!(folder.name, "Work");

        // Create notes in folder
        for i in 1..=3 {
            let insert_note = InsertNote {
                id: format!("note-{}", i),
                title: format!("Note {}", i),
                content: None,
                folder_id: Some("folder-1".to_string()),
                audio_url: None,
                duration: None,
                rating: None,
            };
            db.insert_note(&insert_note).unwrap();
        }

        // List notes by folder
        let notes = db.list_notes_by_folder("folder-1", 10, 0).unwrap();
        assert_eq!(notes.len(), 3);
        assert!(notes
            .iter()
            .all(|n| n.folder_id == Some("folder-1".to_string())));
    }

    #[test]
    fn test_mark_note_synced() {
        let db = setup();

        let insert_note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };

        let note = db.insert_note(&insert_note).unwrap();
        assert!(note.synced_at.is_none());

        db.mark_note_synced("note-1").unwrap();

        let synced_note = db.get_note("note-1").unwrap().unwrap();
        assert!(synced_note.synced_at.is_some());
    }

    #[test]
    fn test_mark_nonexistent_note_synced() {
        let db = setup();

        let result = db.mark_note_synced("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Note not found"));
    }

    #[test]
    fn test_notes_ordered_by_created_at_desc() {
        let db = setup();

        // Insert notes with delays to ensure different timestamps
        db.insert_note(&InsertNote {
            id: "note-1".to_string(),
            title: "First".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        })
        .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        db.insert_note(&InsertNote {
            id: "note-2".to_string(),
            title: "Second".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        })
        .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        db.insert_note(&InsertNote {
            id: "note-3".to_string(),
            title: "Third".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        })
        .unwrap();

        let notes = db.list_notes(10, 0).unwrap();

        // Most recent first
        assert_eq!(notes[0].id, "note-3");
        assert_eq!(notes[1].id, "note-2");
        assert_eq!(notes[2].id, "note-1");
    }

    // ===== TODO TESTS =====

    fn create_test_todo(id: &str, description: &str) -> InsertTodo {
        InsertTodo {
            id: id.to_string(),
            description: description.to_string(),
            due_date: None,
            state: TodoState::Open,
            smart_label: None,
        }
    }

    #[test]
    fn test_insert_and_get_todo() {
        let db = setup();
        let todo_data = create_test_todo("todo-1", "Test todo");

        let inserted = db.insert_todo(todo_data).expect("Failed to insert todo");

        assert_eq!(inserted.id, "todo-1");
        assert_eq!(inserted.description, "Test todo");
        assert_eq!(inserted.state, TodoState::Open);
        assert!(inserted.due_date.is_none());
        assert!(inserted.smart_label.is_none());

        let retrieved = db.get_todo("todo-1").expect("Failed to get todo");
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, inserted.id);
        assert_eq!(retrieved.description, inserted.description);
    }

    #[test]
    fn test_get_nonexistent_todo() {
        let db = setup();
        let result = db.get_todo("nonexistent").expect("Query failed");
        assert!(result.is_none());
    }

    #[test]
    fn test_insert_todo_with_all_fields() {
        let db = setup();
        let due_date = Utc::now();

        let todo_data = InsertTodo {
            id: "todo-2".to_string(),
            description: "Todo with all fields".to_string(),
            due_date: Some(due_date),
            state: TodoState::Open,
            smart_label: Some("work".to_string()),
        };

        let inserted = db.insert_todo(todo_data).expect("Failed to insert todo");

        assert_eq!(inserted.id, "todo-2");
        assert_eq!(inserted.description, "Todo with all fields");
        assert!(inserted.due_date.is_some());
        assert_eq!(inserted.smart_label, Some("work".to_string()));
    }

    #[test]
    fn test_list_todos_empty() {
        let db = setup();
        let todos = db.list_todos(None).expect("Failed to list todos");
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_list_todos() {
        let db = setup();

        db.insert_todo(create_test_todo("todo-1", "First todo"))
            .unwrap();
        db.insert_todo(create_test_todo("todo-2", "Second todo"))
            .unwrap();
        db.insert_todo(create_test_todo("todo-3", "Third todo"))
            .unwrap();

        let todos = db.list_todos(None).expect("Failed to list todos");
        assert_eq!(todos.len(), 3);

        // Should be ordered by created_at DESC, so most recent first
        assert_eq!(todos[0].id, "todo-3");
        assert_eq!(todos[1].id, "todo-2");
        assert_eq!(todos[2].id, "todo-1");
    }

    #[test]
    fn test_list_todos_with_pagination() {
        let db = setup();

        for i in 1..=5 {
            db.insert_todo(create_test_todo(
                &format!("todo-{}", i),
                &format!("Todo {}", i),
            ))
            .unwrap();
        }

        // Get first page (2 items)
        let page1 = db.list_todos(Some(PaginationParams::new(2, 0))).unwrap();
        assert_eq!(page1.len(), 2);
        assert_eq!(page1[0].id, "todo-5");
        assert_eq!(page1[1].id, "todo-4");

        // Get second page (2 items)
        let page2 = db.list_todos(Some(PaginationParams::new(2, 2))).unwrap();
        assert_eq!(page2.len(), 2);
        assert_eq!(page2[0].id, "todo-3");
        assert_eq!(page2[1].id, "todo-2");

        // Get third page (1 item remaining)
        let page3 = db.list_todos(Some(PaginationParams::new(2, 4))).unwrap();
        assert_eq!(page3.len(), 1);
        assert_eq!(page3[0].id, "todo-1");
    }

    #[test]
    fn test_list_todos_by_state() {
        let db = setup();

        let mut todo1 = create_test_todo("todo-1", "Open todo 1");
        todo1.state = TodoState::Open;
        db.insert_todo(todo1).unwrap();

        let mut todo2 = create_test_todo("todo-2", "Closed todo");
        todo2.state = TodoState::Closed;
        db.insert_todo(todo2).unwrap();

        let mut todo3 = create_test_todo("todo-3", "Open todo 2");
        todo3.state = TodoState::Open;
        db.insert_todo(todo3).unwrap();

        let open_todos = db.list_todos_by_state(TodoState::Open, None).unwrap();
        assert_eq!(open_todos.len(), 2);
        assert!(open_todos.iter().all(|t| t.state == TodoState::Open));

        let closed_todos = db.list_todos_by_state(TodoState::Closed, None).unwrap();
        assert_eq!(closed_todos.len(), 1);
        assert_eq!(closed_todos[0].id, "todo-2");
    }

    #[test]
    fn test_update_todo_description() {
        let db = setup();
        db.insert_todo(create_test_todo("todo-1", "Original description"))
            .unwrap();

        let updates = UpdateTodo {
            description: Some("Updated description".to_string()),
            due_date: None,
            state: None,
            smart_label: None,
        };

        let updated = db
            .update_todo("todo-1", updates)
            .expect("Failed to update todo");
        assert_eq!(updated.description, "Updated description");
        assert_eq!(updated.state, TodoState::Open);
    }

    #[test]
    fn test_update_todo_state() {
        let db = setup();
        db.insert_todo(create_test_todo("todo-1", "Test todo"))
            .unwrap();

        let updates = UpdateTodo {
            description: None,
            due_date: None,
            state: Some(TodoState::Closed),
            smart_label: None,
        };

        let updated = db
            .update_todo("todo-1", updates)
            .expect("Failed to update todo");
        assert_eq!(updated.state, TodoState::Closed);
        assert_eq!(updated.description, "Test todo");
    }

    #[test]
    fn test_update_todo_multiple_fields() {
        let db = setup();
        db.insert_todo(create_test_todo("todo-1", "Test todo"))
            .unwrap();

        let due_date = Utc::now();
        let updates = UpdateTodo {
            description: Some("Updated todo".to_string()),
            due_date: Some(due_date),
            state: Some(TodoState::Closed),
            smart_label: Some("urgent".to_string()),
        };

        let updated = db
            .update_todo("todo-1", updates)
            .expect("Failed to update todo");
        assert_eq!(updated.description, "Updated todo");
        assert_eq!(updated.state, TodoState::Closed);
        assert!(updated.due_date.is_some());
        assert_eq!(updated.smart_label, Some("urgent".to_string()));
    }

    #[test]
    fn test_update_nonexistent_todo() {
        let db = setup();

        let updates = UpdateTodo {
            description: Some("Updated".to_string()),
            due_date: None,
            state: None,
            smart_label: None,
        };

        let result = db.update_todo("nonexistent", updates);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Todo not found"));
    }

    #[test]
    fn test_delete_todo() {
        let db = setup();
        db.insert_todo(create_test_todo("todo-1", "Test todo"))
            .unwrap();

        let deleted = db.delete_todo("todo-1").expect("Failed to delete todo");
        assert!(deleted);

        let retrieved = db.get_todo("todo-1").expect("Query failed");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_delete_nonexistent_todo() {
        let db = setup();
        let deleted = db
            .delete_todo("nonexistent")
            .expect("Failed to delete todo");
        assert!(!deleted);
    }

    #[test]
    fn test_count_todos() {
        let db = setup();
        assert_eq!(db.count_todos().unwrap(), 0);

        db.insert_todo(create_test_todo("todo-1", "First")).unwrap();
        assert_eq!(db.count_todos().unwrap(), 1);

        db.insert_todo(create_test_todo("todo-2", "Second"))
            .unwrap();
        assert_eq!(db.count_todos().unwrap(), 2);

        db.delete_todo("todo-1").unwrap();
        assert_eq!(db.count_todos().unwrap(), 1);
    }

    #[test]
    fn test_count_todos_by_state() {
        let db = setup();

        let mut todo1 = create_test_todo("todo-1", "Open 1");
        todo1.state = TodoState::Open;
        db.insert_todo(todo1).unwrap();

        let mut todo2 = create_test_todo("todo-2", "Open 2");
        todo2.state = TodoState::Open;
        db.insert_todo(todo2).unwrap();

        let mut todo3 = create_test_todo("todo-3", "Closed");
        todo3.state = TodoState::Closed;
        db.insert_todo(todo3).unwrap();

        assert_eq!(db.count_todos_by_state(TodoState::Open).unwrap(), 2);
        assert_eq!(db.count_todos_by_state(TodoState::Closed).unwrap(), 1);
    }

    #[test]
    fn test_mark_todo_synced() {
        let db = setup();

        let todo = db
            .insert_todo(create_test_todo("todo-1", "Test todo"))
            .unwrap();
        assert!(todo.synced_at.is_none());

        db.mark_todo_synced("todo-1").unwrap();

        let synced_todo = db.get_todo("todo-1").unwrap().unwrap();
        assert!(synced_todo.synced_at.is_some());
    }

    #[test]
    fn test_mark_nonexistent_todo_synced() {
        let db = setup();

        let result = db.mark_todo_synced("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Todo not found"));
    }

    // ===== CALENDAR EVENTS TESTS =====

    fn create_test_event(id: &str, title: &str, hours_from_now: i64) -> CalendarEvent {
        let now = Utc::now();
        CalendarEvent {
            id: id.to_string(),
            title: title.to_string(),
            start_time: now + chrono::Duration::hours(hours_from_now),
            end_time: now + chrono::Duration::hours(hours_from_now + 1),
            source: EventSource::GoogleCalendar,
            meeting_url: Some("https://meet.google.com/test".to_string()),
            created_at: now,
            synced_at: None,
        }
    }

    #[test]
    fn test_insert_and_get_calendar_event() {
        let db = setup();

        let event = create_test_event("event-1", "Team Meeting", 2);
        let inserted = db.insert_calendar_event(&event).unwrap();

        assert_eq!(inserted.id, "event-1");
        assert_eq!(inserted.title, "Team Meeting");
        assert_eq!(inserted.source, EventSource::GoogleCalendar);
        assert!(inserted.meeting_url.is_some());
        assert!(inserted.synced_at.is_none());

        let retrieved = db.get_calendar_event("event-1").unwrap().unwrap();
        assert_eq!(retrieved.id, inserted.id);
        assert_eq!(retrieved.title, inserted.title);
    }

    #[test]
    fn test_get_nonexistent_calendar_event() {
        let db = setup();

        let result = db.get_calendar_event("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_calendar_events_with_pagination() {
        let db = setup();

        // Insert 5 events
        for i in 1..=5 {
            let event = create_test_event(&format!("event-{}", i), &format!("Event {}", i), i);
            db.insert_calendar_event(&event).unwrap();
        }

        // Get first page
        let page1 = db.list_calendar_events(2, 0).unwrap();
        assert_eq!(page1.len(), 2);

        // Get second page
        let page2 = db.list_calendar_events(2, 2).unwrap();
        assert_eq!(page2.len(), 2);

        // Get third page
        let page3 = db.list_calendar_events(2, 4).unwrap();
        assert_eq!(page3.len(), 1);

        // Verify total count
        let count = db.count_calendar_events().unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_list_calendar_events_in_range() {
        let db = setup();

        let now = Utc::now();

        // Insert events at different times
        let event1 = create_test_event("event-1", "Today", 1);
        let event2 = create_test_event("event-2", "Tomorrow", 25);
        let event3 = create_test_event("event-3", "Later Today", 5);

        db.insert_calendar_event(&event1).unwrap();
        db.insert_calendar_event(&event2).unwrap();
        db.insert_calendar_event(&event3).unwrap();

        // Query events for the next 24 hours
        let start = now;
        let end = now + chrono::Duration::hours(24);

        let events = db.list_calendar_events_in_range(start, end).unwrap();

        // Should only get events within the range
        assert_eq!(events.len(), 2);
        assert!(events.iter().any(|e| e.id == "event-1"));
        assert!(events.iter().any(|e| e.id == "event-3"));

        // Events should be ordered by start_time
        assert!(events[0].start_time <= events[1].start_time);
    }

    #[test]
    fn test_update_calendar_event() {
        let db = setup();

        let event = create_test_event("event-1", "Original Title", 2);
        db.insert_calendar_event(&event).unwrap();

        let mut updated_event = event.clone();
        updated_event.title = "Updated Title".to_string();
        updated_event.meeting_url = Some("https://zoom.us/new".to_string());

        let result = db.update_calendar_event(&updated_event).unwrap();

        assert_eq!(result.title, "Updated Title");
        assert_eq!(result.meeting_url, Some("https://zoom.us/new".to_string()));
    }

    #[test]
    fn test_update_nonexistent_calendar_event() {
        let db = setup();

        let event = create_test_event("nonexistent", "Test", 2);
        let result = db.update_calendar_event(&event);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Calendar event not found"));
    }

    #[test]
    fn test_delete_calendar_event() {
        let db = setup();

        let event = create_test_event("event-1", "Test Event", 2);
        db.insert_calendar_event(&event).unwrap();
        assert!(db.get_calendar_event("event-1").unwrap().is_some());

        db.delete_calendar_event("event-1").unwrap();
        assert!(db.get_calendar_event("event-1").unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent_calendar_event() {
        let db = setup();

        let result = db.delete_calendar_event("nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Calendar event not found"));
    }

    #[test]
    fn test_mark_calendar_event_synced() {
        let db = setup();

        let event = create_test_event("event-1", "Test Event", 2);
        let inserted = db.insert_calendar_event(&event).unwrap();
        assert!(inserted.synced_at.is_none());

        db.mark_calendar_event_synced("event-1").unwrap();

        let synced_event = db.get_calendar_event("event-1").unwrap().unwrap();
        assert!(synced_event.synced_at.is_some());
    }

    #[test]
    fn test_mark_nonexistent_calendar_event_synced() {
        let db = setup();

        let result = db.mark_calendar_event_synced("nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Calendar event not found"));
    }

    #[test]
    fn test_calendar_events_ordered_by_start_time() {
        let db = setup();

        // Insert events in non-chronological order
        let event3 = create_test_event("event-3", "Third", 6);
        let event1 = create_test_event("event-1", "First", 2);
        let event2 = create_test_event("event-2", "Second", 4);

        db.insert_calendar_event(&event3).unwrap();
        db.insert_calendar_event(&event1).unwrap();
        db.insert_calendar_event(&event2).unwrap();

        let events = db.list_calendar_events(10, 0).unwrap();

        // Should be ordered by start_time ASC
        assert_eq!(events[0].id, "event-1");
        assert_eq!(events[1].id, "event-2");
        assert_eq!(events[2].id, "event-3");
    }

    #[test]
    fn test_calendar_event_sources() {
        let db = setup();

        let mut google_event = create_test_event("event-1", "Google Event", 2);
        google_event.source = EventSource::GoogleCalendar;

        let mut hinotes_event = create_test_event("event-2", "HiNotes Event", 3);
        hinotes_event.source = EventSource::Hinotes;

        db.insert_calendar_event(&google_event).unwrap();
        db.insert_calendar_event(&hinotes_event).unwrap();

        let retrieved_google = db.get_calendar_event("event-1").unwrap().unwrap();
        let retrieved_hinotes = db.get_calendar_event("event-2").unwrap().unwrap();

        assert_eq!(retrieved_google.source, EventSource::GoogleCalendar);
        assert_eq!(retrieved_hinotes.source, EventSource::Hinotes);
    }

    // ===== TEMPLATE TESTS =====

    #[test]
    fn test_insert_and_get_template() {
        let db = setup();

        let template = db
            .insert_template(
                "template-1",
                "Meeting Notes",
                "## Meeting Notes\n\nDate: \nAttendees: ",
                false,
                false,
            )
            .unwrap();

        assert_eq!(template.id, "template-1");
        assert_eq!(template.title, "Meeting Notes");
        assert_eq!(template.content, "## Meeting Notes\n\nDate: \nAttendees: ");
        assert!(!template.is_favorite);
        assert!(!template.is_default);
        assert!(template.synced_at.is_none());

        let retrieved = db.get_template("template-1").unwrap().unwrap();
        assert_eq!(retrieved.id, template.id);
        assert_eq!(retrieved.title, template.title);
    }

    #[test]
    fn test_get_nonexistent_template() {
        let db = setup();

        let result = db.get_template("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_templates() {
        let db = setup();

        // Insert multiple templates
        db.insert_template("template-1", "Template 1", "Content 1", false, false)
            .unwrap();
        db.insert_template("template-2", "Template 2", "Content 2", true, false)
            .unwrap();
        db.insert_template("template-3", "Template 3", "Content 3", true, false)
            .unwrap();

        // List all templates
        let all_templates = db.list_templates(false, 10, 0).unwrap();
        assert_eq!(all_templates.len(), 3);

        // List only favorites
        let favorites = db.list_templates(true, 10, 0).unwrap();
        assert_eq!(favorites.len(), 2);
        assert!(favorites.iter().all(|t| t.is_favorite));
    }

    #[test]
    fn test_list_templates_pagination() {
        let db = setup();

        // Insert 5 templates
        for i in 1..=5 {
            db.insert_template(
                &format!("template-{}", i),
                &format!("Template {}", i),
                &format!("Content {}", i),
                false,
                false,
            )
            .unwrap();
        }

        // Get first page
        let page1 = db.list_templates(false, 2, 0).unwrap();
        assert_eq!(page1.len(), 2);

        // Get second page
        let page2 = db.list_templates(false, 2, 2).unwrap();
        assert_eq!(page2.len(), 2);

        // Get third page
        let page3 = db.list_templates(false, 2, 4).unwrap();
        assert_eq!(page3.len(), 1);
    }

    #[test]
    fn test_update_template() {
        let db = setup();

        db.insert_template(
            "template-1",
            "Original Title",
            "Original Content",
            false,
            false,
        )
        .unwrap();

        let updated = db
            .update_template(
                "template-1",
                Some("Updated Title"),
                Some("Updated Content"),
                Some(true),
                None,
            )
            .unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.content, "Updated Content");
        assert!(updated.is_favorite);
        assert!(!updated.is_default);
    }

    #[test]
    fn test_update_template_partial() {
        let db = setup();

        db.insert_template(
            "template-1",
            "Original Title",
            "Original Content",
            false,
            false,
        )
        .unwrap();

        // Update only title
        let updated = db
            .update_template("template-1", Some("New Title"), None, None, None)
            .unwrap();

        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.content, "Original Content"); // Content unchanged
    }

    #[test]
    fn test_update_nonexistent_template() {
        let db = setup();

        let result = db.update_template("nonexistent", Some("Title"), None, None, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Template not found"));
    }

    #[test]
    fn test_toggle_template_favorite() {
        let db = setup();

        let template = db
            .insert_template("template-1", "Test Template", "Content", false, false)
            .unwrap();
        assert!(!template.is_favorite);

        // Toggle to favorite
        let toggled = db.toggle_template_favorite("template-1").unwrap();
        assert!(toggled.is_favorite);

        // Toggle back to non-favorite
        let toggled_back = db.toggle_template_favorite("template-1").unwrap();
        assert!(!toggled_back.is_favorite);
    }

    #[test]
    fn test_set_template_default() {
        let db = setup();

        // Insert multiple templates
        db.insert_template("template-1", "Template 1", "Content 1", false, false)
            .unwrap();
        db.insert_template("template-2", "Template 2", "Content 2", false, false)
            .unwrap();

        // Set first template as default
        db.set_template_default("template-1").unwrap();

        let template1 = db.get_template("template-1").unwrap().unwrap();
        assert!(template1.is_default);

        // Set second template as default (should unset first)
        db.set_template_default("template-2").unwrap();

        let template1_updated = db.get_template("template-1").unwrap().unwrap();
        let template2 = db.get_template("template-2").unwrap().unwrap();

        assert!(!template1_updated.is_default);
        assert!(template2.is_default);
    }

    #[test]
    fn test_get_default_template() {
        let db = setup();

        // No default initially
        let default = db.get_default_template().unwrap();
        assert!(default.is_none());

        // Set a template as default
        db.insert_template("template-1", "Default Template", "Content", false, true)
            .unwrap();

        let default = db.get_default_template().unwrap().unwrap();
        assert_eq!(default.id, "template-1");
        assert!(default.is_default);
    }

    #[test]
    fn test_insert_template_as_default_unsets_others() {
        let db = setup();

        // Insert first template as default
        db.insert_template("template-1", "Template 1", "Content 1", false, true)
            .unwrap();

        // Insert second template as default
        db.insert_template("template-2", "Template 2", "Content 2", false, true)
            .unwrap();

        let template1 = db.get_template("template-1").unwrap().unwrap();
        let template2 = db.get_template("template-2").unwrap().unwrap();

        assert!(!template1.is_default);
        assert!(template2.is_default);
    }

    #[test]
    fn test_delete_template() {
        let db = setup();

        db.insert_template("template-1", "Test Template", "Content", false, false)
            .unwrap();
        assert!(db.get_template("template-1").unwrap().is_some());

        db.delete_template("template-1").unwrap();
        assert!(db.get_template("template-1").unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent_template() {
        let db = setup();

        let result = db.delete_template("nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Template not found"));
    }

    #[test]
    fn test_count_templates() {
        let db = setup();

        assert_eq!(db.count_templates().unwrap(), 0);

        db.insert_template("template-1", "Template 1", "Content 1", false, false)
            .unwrap();
        db.insert_template("template-2", "Template 2", "Content 2", false, false)
            .unwrap();

        assert_eq!(db.count_templates().unwrap(), 2);
    }

    #[test]
    fn test_mark_template_synced() {
        let db = setup();

        let template = db
            .insert_template("template-1", "Test Template", "Content", false, false)
            .unwrap();
        assert!(template.synced_at.is_none());

        db.mark_template_synced("template-1").unwrap();

        let synced_template = db.get_template("template-1").unwrap().unwrap();
        assert!(synced_template.synced_at.is_some());
    }

    #[test]
    fn test_mark_nonexistent_template_synced() {
        let db = setup();

        let result = db.mark_template_synced("nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Template not found"));
    }

    // ===== SMART LABELS TESTS =====

    #[test]
    fn test_insert_and_get_smart_label() {
        let db = setup();

        let label = InsertSmartLabel {
            id: "label-1".to_string(),
            name: "Work".to_string(),
            color: Some("#FF0000".to_string()),
        };

        let inserted = db.insert_smart_label(&label).unwrap();
        assert_eq!(inserted.id, "label-1");
        assert_eq!(inserted.name, "Work");
        assert_eq!(inserted.color, Some("#FF0000".to_string()));

        let retrieved = db.get_smart_label("label-1").unwrap().unwrap();
        assert_eq!(retrieved.id, inserted.id);
        assert_eq!(retrieved.name, inserted.name);
    }

    #[test]
    fn test_get_nonexistent_smart_label() {
        let db = setup();

        let result = db.get_smart_label("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_smart_labels() {
        let db = setup();

        // Insert multiple labels
        db.insert_smart_label(&InsertSmartLabel {
            id: "label-1".to_string(),
            name: "Work".to_string(),
            color: Some("#FF0000".to_string()),
        })
        .unwrap();

        db.insert_smart_label(&InsertSmartLabel {
            id: "label-2".to_string(),
            name: "Personal".to_string(),
            color: Some("#00FF00".to_string()),
        })
        .unwrap();

        let labels = db.list_smart_labels().unwrap();
        assert_eq!(labels.len(), 2);
    }

    #[test]
    fn test_update_smart_label() {
        let db = setup();

        db.insert_smart_label(&InsertSmartLabel {
            id: "label-1".to_string(),
            name: "Work".to_string(),
            color: Some("#FF0000".to_string()),
        })
        .unwrap();

        let update = UpdateSmartLabel {
            name: Some("Office".to_string()),
            color: Some("#0000FF".to_string()),
        };

        let updated = db.update_smart_label("label-1", &update).unwrap();
        assert_eq!(updated.name, "Office");
        assert_eq!(updated.color, Some("#0000FF".to_string()));
    }

    #[test]
    fn test_update_nonexistent_smart_label() {
        let db = setup();

        let update = UpdateSmartLabel {
            name: Some("Test".to_string()),
            color: None,
        };

        let result = db.update_smart_label("nonexistent", &update);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Smart label not found"));
    }

    #[test]
    fn test_delete_smart_label() {
        let db = setup();

        db.insert_smart_label(&InsertSmartLabel {
            id: "label-1".to_string(),
            name: "Work".to_string(),
            color: None,
        })
        .unwrap();

        assert!(db.get_smart_label("label-1").unwrap().is_some());

        db.delete_smart_label("label-1").unwrap();
        assert!(db.get_smart_label("label-1").unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent_smart_label() {
        let db = setup();

        let result = db.delete_smart_label("nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Smart label not found"));
    }

    #[test]
    fn test_count_smart_labels() {
        let db = setup();

        assert_eq!(db.count_smart_labels().unwrap(), 0);

        db.insert_smart_label(&InsertSmartLabel {
            id: "label-1".to_string(),
            name: "Work".to_string(),
            color: None,
        })
        .unwrap();

        assert_eq!(db.count_smart_labels().unwrap(), 1);
    }

    #[test]
    fn test_smart_label_unique_name() {
        let db = setup();

        db.insert_smart_label(&InsertSmartLabel {
            id: "label-1".to_string(),
            name: "Work".to_string(),
            color: None,
        })
        .unwrap();

        // Try to insert another label with the same name (should fail due to UNIQUE constraint)
        let result = db.insert_smart_label(&InsertSmartLabel {
            id: "label-2".to_string(),
            name: "Work".to_string(),
            color: Some("#FF0000".to_string()),
        });

        assert!(result.is_err());
    }

    // ===== VOCABULARY TESTS =====

    #[test]
    fn test_insert_and_get_vocabulary() {
        let db = setup();

        let vocab = InsertVocabulary {
            id: "vocab-1".to_string(),
            word: "Kubernetes".to_string(),
            pronunciation: Some("koo-ber-net-eez".to_string()),
        };

        let inserted = db.insert_vocabulary(&vocab).unwrap();
        assert_eq!(inserted.id, "vocab-1");
        assert_eq!(inserted.word, "Kubernetes");
        assert_eq!(inserted.pronunciation, Some("koo-ber-net-eez".to_string()));

        let retrieved = db.get_vocabulary("vocab-1").unwrap().unwrap();
        assert_eq!(retrieved.id, inserted.id);
        assert_eq!(retrieved.word, inserted.word);
    }

    #[test]
    fn test_get_nonexistent_vocabulary() {
        let db = setup();

        let result = db.get_vocabulary("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_vocabulary() {
        let db = setup();

        // Insert multiple words
        db.insert_vocabulary(&InsertVocabulary {
            id: "vocab-1".to_string(),
            word: "Kubernetes".to_string(),
            pronunciation: None,
        })
        .unwrap();

        db.insert_vocabulary(&InsertVocabulary {
            id: "vocab-2".to_string(),
            word: "Algorithm".to_string(),
            pronunciation: Some("al-go-rith-um".to_string()),
        })
        .unwrap();

        let words = db.list_vocabulary().unwrap();
        assert_eq!(words.len(), 2);

        // Should be ordered alphabetically
        assert_eq!(words[0].word, "Algorithm");
        assert_eq!(words[1].word, "Kubernetes");
    }

    #[test]
    fn test_delete_vocabulary() {
        let db = setup();

        db.insert_vocabulary(&InsertVocabulary {
            id: "vocab-1".to_string(),
            word: "Test".to_string(),
            pronunciation: None,
        })
        .unwrap();

        assert!(db.get_vocabulary("vocab-1").unwrap().is_some());

        db.delete_vocabulary("vocab-1").unwrap();
        assert!(db.get_vocabulary("vocab-1").unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent_vocabulary() {
        let db = setup();

        let result = db.delete_vocabulary("nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Vocabulary not found"));
    }

    #[test]
    fn test_count_vocabulary() {
        let db = setup();

        assert_eq!(db.count_vocabulary().unwrap(), 0);

        db.insert_vocabulary(&InsertVocabulary {
            id: "vocab-1".to_string(),
            word: "Test".to_string(),
            pronunciation: None,
        })
        .unwrap();

        assert_eq!(db.count_vocabulary().unwrap(), 1);
    }

    #[test]
    fn test_vocabulary_unique_word() {
        let db = setup();

        db.insert_vocabulary(&InsertVocabulary {
            id: "vocab-1".to_string(),
            word: "Kubernetes".to_string(),
            pronunciation: None,
        })
        .unwrap();

        // Try to insert another word with the same name (should fail due to UNIQUE constraint)
        let result = db.insert_vocabulary(&InsertVocabulary {
            id: "vocab-2".to_string(),
            word: "Kubernetes".to_string(),
            pronunciation: Some("different".to_string()),
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_export_vocabulary_json() {
        let db = setup();

        db.insert_vocabulary(&InsertVocabulary {
            id: "vocab-1".to_string(),
            word: "Kubernetes".to_string(),
            pronunciation: Some("koo-ber-net-eez".to_string()),
        })
        .unwrap();

        db.insert_vocabulary(&InsertVocabulary {
            id: "vocab-2".to_string(),
            word: "Algorithm".to_string(),
            pronunciation: None,
        })
        .unwrap();

        let json = db.export_vocabulary_json().unwrap();
        assert!(json.contains("Kubernetes"));
        assert!(json.contains("Algorithm"));
        assert!(json.contains("koo-ber-net-eez"));
    }

    #[test]
    fn test_import_vocabulary_json() {
        let db = setup();

        let json = r#"[
            {
                "id": "vocab-1",
                "word": "Kubernetes",
                "pronunciation": "koo-ber-net-eez"
            },
            {
                "id": "vocab-2",
                "word": "Algorithm",
                "pronunciation": null
            }
        ]"#;

        let imported_count = db.import_vocabulary_json(json).unwrap();
        assert_eq!(imported_count, 2);

        let words = db.list_vocabulary().unwrap();
        assert_eq!(words.len(), 2);
    }

    #[test]
    fn test_import_vocabulary_json_skip_duplicates() {
        let db = setup();

        // Insert one word first
        db.insert_vocabulary(&InsertVocabulary {
            id: "vocab-1".to_string(),
            word: "Kubernetes".to_string(),
            pronunciation: None,
        })
        .unwrap();

        // Try to import a list that includes a duplicate
        let json = r#"[
            {
                "id": "vocab-1-dup",
                "word": "Kubernetes",
                "pronunciation": "koo-ber-net-eez"
            },
            {
                "id": "vocab-2",
                "word": "Algorithm",
                "pronunciation": null
            }
        ]"#;

        let imported_count = db.import_vocabulary_json(json).unwrap();
        // Should only import 1 (Algorithm), skipping the duplicate Kubernetes
        assert_eq!(imported_count, 1);

        let words = db.list_vocabulary().unwrap();
        assert_eq!(words.len(), 2);
    }

    // ===== DEVICE TESTS =====

    fn create_test_device(id: &str, name: &str, status: DeviceStatus) -> Device {
        let now = Utc::now();
        Device {
            id: id.to_string(),
            name: name.to_string(),
            status,
            last_sync: None,
            created_at: now,
        }
    }

    #[test]
    fn test_insert_and_get_device() {
        let db = setup();

        let device = create_test_device("device-1", "HiDoc P1", DeviceStatus::Connected);
        let inserted = db.insert_device(&device).unwrap();

        assert_eq!(inserted.id, "device-1");
        assert_eq!(inserted.name, "HiDoc P1");
        assert_eq!(inserted.status, DeviceStatus::Connected);
        assert!(inserted.last_sync.is_none());

        let retrieved = db.get_device("device-1").unwrap().unwrap();
        assert_eq!(retrieved.id, inserted.id);
        assert_eq!(retrieved.name, inserted.name);
        assert_eq!(retrieved.status, inserted.status);
    }

    #[test]
    fn test_get_nonexistent_device() {
        let db = setup();

        let result = db.get_device("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_devices() {
        let db = setup();

        // Insert multiple devices with delays to ensure different timestamps
        let device1 = create_test_device("device-1", "HiDoc P1", DeviceStatus::Connected);
        db.insert_device(&device1).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));
        let device2 = create_test_device("device-2", "HiDoc P2", DeviceStatus::Disconnected);
        db.insert_device(&device2).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));
        let device3 = create_test_device("device-3", "HiDoc P3", DeviceStatus::Connected);
        db.insert_device(&device3).unwrap();

        let devices = db.list_devices().unwrap();
        assert_eq!(devices.len(), 3);

        // Should be ordered by created_at DESC
        assert_eq!(devices[0].id, "device-3");
        assert_eq!(devices[1].id, "device-2");
        assert_eq!(devices[2].id, "device-1");
    }

    #[test]
    fn test_list_devices_empty() {
        let db = setup();

        let devices = db.list_devices().unwrap();
        assert_eq!(devices.len(), 0);
    }

    #[test]
    fn test_update_device_status() {
        let db = setup();

        let device = create_test_device("device-1", "HiDoc P1", DeviceStatus::Connected);
        db.insert_device(&device).unwrap();

        let updated = db
            .update_device_status("device-1", DeviceStatus::Disconnected)
            .unwrap();

        assert_eq!(updated.status, DeviceStatus::Disconnected);
        assert_eq!(updated.name, "HiDoc P1");
        assert_eq!(updated.id, "device-1");
    }

    #[test]
    fn test_update_device_status_nonexistent() {
        let db = setup();

        let result = db.update_device_status("nonexistent", DeviceStatus::Connected);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Device not found"));
    }

    #[test]
    fn test_delete_device() {
        let db = setup();

        let device = create_test_device("device-1", "HiDoc P1", DeviceStatus::Connected);
        db.insert_device(&device).unwrap();
        assert!(db.get_device("device-1").unwrap().is_some());

        db.delete_device("device-1").unwrap();
        assert!(db.get_device("device-1").unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent_device() {
        let db = setup();

        let result = db.delete_device("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Device not found"));
    }

    #[test]
    fn test_update_device_last_sync() {
        let db = setup();

        let device = create_test_device("device-1", "HiDoc P1", DeviceStatus::Connected);
        let inserted = db.insert_device(&device).unwrap();
        assert!(inserted.last_sync.is_none());

        std::thread::sleep(std::time::Duration::from_millis(10));

        let updated = db.update_device_last_sync("device-1").unwrap();
        assert!(updated.last_sync.is_some());
        assert!(updated.last_sync.unwrap() > inserted.created_at);
    }

    #[test]
    fn test_update_device_last_sync_nonexistent() {
        let db = setup();

        let result = db.update_device_last_sync("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Device not found"));
    }

    #[test]
    fn test_count_devices() {
        let db = setup();
        assert_eq!(db.count_devices().unwrap(), 0);

        let device1 = create_test_device("device-1", "HiDoc P1", DeviceStatus::Connected);
        db.insert_device(&device1).unwrap();
        assert_eq!(db.count_devices().unwrap(), 1);

        let device2 = create_test_device("device-2", "HiDoc P2", DeviceStatus::Disconnected);
        db.insert_device(&device2).unwrap();
        assert_eq!(db.count_devices().unwrap(), 2);

        db.delete_device("device-1").unwrap();
        assert_eq!(db.count_devices().unwrap(), 1);
    }

    #[test]
    fn test_device_status_persistence() {
        let db = setup();

        // Test connected status
        let device1 = create_test_device("device-1", "HiDoc P1", DeviceStatus::Connected);
        db.insert_device(&device1).unwrap();
        let retrieved1 = db.get_device("device-1").unwrap().unwrap();
        assert_eq!(retrieved1.status, DeviceStatus::Connected);

        // Test disconnected status
        let device2 = create_test_device("device-2", "HiDoc P2", DeviceStatus::Disconnected);
        db.insert_device(&device2).unwrap();
        let retrieved2 = db.get_device("device-2").unwrap().unwrap();
        assert_eq!(retrieved2.status, DeviceStatus::Disconnected);
    }

    #[test]
    fn test_device_with_last_sync() {
        let db = setup();

        let now = Utc::now();
        let mut device = create_test_device("device-1", "HiDoc P1", DeviceStatus::Connected);
        device.last_sync = Some(now);

        let inserted = db.insert_device(&device).unwrap();
        assert!(inserted.last_sync.is_some());

        let retrieved = db.get_device("device-1").unwrap().unwrap();
        assert!(retrieved.last_sync.is_some());
    }

    // ===== SPEAKER TESTS =====

    fn create_test_speaker(id: &str, name: Option<&str>) -> InsertSpeaker {
        InsertSpeaker {
            id: id.to_string(),
            name: name.map(|s| s.to_string()),
            voice_signature: None,
        }
    }

    #[test]
    fn test_insert_and_get_speaker() {
        let db = setup();

        let speaker_data = create_test_speaker("speaker-1", Some("John Doe"));
        let inserted = db.insert_speaker(&speaker_data).unwrap();

        assert_eq!(inserted.id, "speaker-1");
        assert_eq!(inserted.name, Some("John Doe".to_string()));
        assert!(inserted.voice_signature.is_none());

        let retrieved = db.get_speaker("speaker-1").unwrap().unwrap();
        assert_eq!(retrieved.id, inserted.id);
        assert_eq!(retrieved.name, inserted.name);
    }

    #[test]
    fn test_get_nonexistent_speaker() {
        let db = setup();

        let result = db.get_speaker("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_speakers() {
        let db = setup();

        db.insert_speaker(&create_test_speaker("speaker-1", Some("Alice")))
            .unwrap();
        db.insert_speaker(&create_test_speaker("speaker-2", Some("Bob")))
            .unwrap();
        db.insert_speaker(&create_test_speaker("speaker-3", None))
            .unwrap();

        let speakers = db.list_speakers().unwrap();
        assert_eq!(speakers.len(), 3);
    }

    #[test]
    fn test_update_speaker_name() {
        let db = setup();

        db.insert_speaker(&create_test_speaker("speaker-1", Some("Original Name")))
            .unwrap();

        let update = UpdateSpeaker {
            name: Some("Updated Name".to_string()),
            voice_signature: None,
        };

        let updated = db.update_speaker("speaker-1", &update).unwrap();
        assert_eq!(updated.name, Some("Updated Name".to_string()));
    }

    #[test]
    fn test_update_speaker_voice_signature() {
        let db = setup();

        db.insert_speaker(&create_test_speaker("speaker-1", Some("John")))
            .unwrap();

        let update = UpdateSpeaker {
            name: None,
            voice_signature: Some("voice_data_123".to_string()),
        };

        let updated = db.update_speaker("speaker-1", &update).unwrap();
        assert_eq!(updated.voice_signature, Some("voice_data_123".to_string()));
        assert_eq!(updated.name, Some("John".to_string()));
    }

    #[test]
    fn test_delete_speaker() {
        let db = setup();

        db.insert_speaker(&create_test_speaker("speaker-1", Some("John")))
            .unwrap();
        assert!(db.get_speaker("speaker-1").unwrap().is_some());

        db.delete_speaker("speaker-1").unwrap();
        assert!(db.get_speaker("speaker-1").unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent_speaker() {
        let db = setup();

        let result = db.delete_speaker("nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Speaker not found"));
    }

    #[test]
    fn test_count_speakers() {
        let db = setup();

        assert_eq!(db.count_speakers().unwrap(), 0);

        db.insert_speaker(&create_test_speaker("speaker-1", Some("Alice")))
            .unwrap();
        assert_eq!(db.count_speakers().unwrap(), 1);

        db.insert_speaker(&create_test_speaker("speaker-2", Some("Bob")))
            .unwrap();
        assert_eq!(db.count_speakers().unwrap(), 2);
    }

    // ===== SPEAKER SEGMENT TESTS =====

    fn create_test_segment(
        note_id: &str,
        speaker_id: &str,
        start: f64,
        end: f64,
    ) -> InsertSpeakerSegment {
        InsertSpeakerSegment {
            id: uuid::Uuid::new_v4().to_string(),
            note_id: note_id.to_string(),
            speaker_id: speaker_id.to_string(),
            start_time: start,
            end_time: end,
            confidence: 0.95,
        }
    }

    #[test]
    fn test_insert_and_get_speaker_segment() {
        let db = setup();

        // Create prerequisites
        let note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();
        db.insert_speaker(&create_test_speaker("speaker-1", Some("John")))
            .unwrap();

        let segment_data = create_test_segment("note-1", "speaker-1", 0.0, 10.5);
        let segment_id = segment_data.id.clone();
        let inserted = db.insert_speaker_segment(&segment_data).unwrap();

        assert_eq!(inserted.id, segment_id);
        assert_eq!(inserted.note_id, "note-1");
        assert_eq!(inserted.speaker_id, "speaker-1");
        assert_eq!(inserted.start_time, 0.0);
        assert_eq!(inserted.end_time, 10.5);
        assert_eq!(inserted.confidence, 0.95);

        let retrieved = db.get_speaker_segment(&segment_id).unwrap().unwrap();
        assert_eq!(retrieved.id, inserted.id);
    }

    #[test]
    fn test_list_speaker_segments_by_note() {
        let db = setup();

        // Create prerequisites
        let note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();
        db.insert_speaker(&create_test_speaker("speaker-1", Some("Alice")))
            .unwrap();
        db.insert_speaker(&create_test_speaker("speaker-2", Some("Bob")))
            .unwrap();

        db.insert_speaker_segment(&create_test_segment("note-1", "speaker-1", 0.0, 10.0))
            .unwrap();
        db.insert_speaker_segment(&create_test_segment("note-1", "speaker-2", 10.0, 20.0))
            .unwrap();
        db.insert_speaker_segment(&create_test_segment("note-1", "speaker-1", 20.0, 30.0))
            .unwrap();

        let segments = db.list_speaker_segments_by_note("note-1").unwrap();
        assert_eq!(segments.len(), 3);

        // Verify ordering by start_time
        assert_eq!(segments[0].start_time, 0.0);
        assert_eq!(segments[1].start_time, 10.0);
        assert_eq!(segments[2].start_time, 20.0);
    }

    #[test]
    fn test_list_speaker_segments_by_speaker() {
        let db = setup();

        // Create prerequisites
        let note1 = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note 1".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        let note2 = InsertNote {
            id: "note-2".to_string(),
            title: "Test Note 2".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note1).unwrap();
        db.insert_note(&note2).unwrap();
        db.insert_speaker(&create_test_speaker("speaker-1", Some("Alice")))
            .unwrap();

        db.insert_speaker_segment(&create_test_segment("note-1", "speaker-1", 0.0, 10.0))
            .unwrap();
        db.insert_speaker_segment(&create_test_segment("note-2", "speaker-1", 0.0, 5.0))
            .unwrap();

        let segments = db.list_speaker_segments_by_speaker("speaker-1").unwrap();
        assert_eq!(segments.len(), 2);
    }

    #[test]
    fn test_delete_speaker_segment() {
        let db = setup();

        // Create prerequisites
        let note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();
        db.insert_speaker(&create_test_speaker("speaker-1", Some("John")))
            .unwrap();

        let segment_data = create_test_segment("note-1", "speaker-1", 0.0, 10.0);
        let segment_id = segment_data.id.clone();
        db.insert_speaker_segment(&segment_data).unwrap();

        assert!(db.get_speaker_segment(&segment_id).unwrap().is_some());

        db.delete_speaker_segment(&segment_id).unwrap();
        assert!(db.get_speaker_segment(&segment_id).unwrap().is_none());
    }

    #[test]
    fn test_delete_speaker_segments_by_note() {
        let db = setup();

        // Create prerequisites
        let note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();
        db.insert_speaker(&create_test_speaker("speaker-1", Some("Alice")))
            .unwrap();
        db.insert_speaker(&create_test_speaker("speaker-2", Some("Bob")))
            .unwrap();

        db.insert_speaker_segment(&create_test_segment("note-1", "speaker-1", 0.0, 10.0))
            .unwrap();
        db.insert_speaker_segment(&create_test_segment("note-1", "speaker-2", 10.0, 20.0))
            .unwrap();

        assert_eq!(db.count_speaker_segments_by_note("note-1").unwrap(), 2);

        let deleted = db.delete_speaker_segments_by_note("note-1").unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(db.count_speaker_segments_by_note("note-1").unwrap(), 0);
    }

    #[test]
    fn test_count_speaker_segments() {
        let db = setup();

        // Create prerequisites
        let note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();
        db.insert_speaker(&create_test_speaker("speaker-1", Some("Alice")))
            .unwrap();

        assert_eq!(db.count_speaker_segments().unwrap(), 0);

        db.insert_speaker_segment(&create_test_segment("note-1", "speaker-1", 0.0, 10.0))
            .unwrap();
        assert_eq!(db.count_speaker_segments().unwrap(), 1);

        db.insert_speaker_segment(&create_test_segment("note-1", "speaker-1", 10.0, 20.0))
            .unwrap();
        assert_eq!(db.count_speaker_segments().unwrap(), 2);
    }

    #[test]
    fn test_get_speaker_statistics() {
        let db = setup();

        // Create prerequisites
        let note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();
        db.insert_speaker(&create_test_speaker("speaker-1", Some("Alice")))
            .unwrap();
        db.insert_speaker(&create_test_speaker("speaker-2", Some("Bob")))
            .unwrap();

        // Speaker 1: 0-10 (10s) + 20-25 (5s) = 15s total, 2 turns
        db.insert_speaker_segment(&create_test_segment("note-1", "speaker-1", 0.0, 10.0))
            .unwrap();
        db.insert_speaker_segment(&create_test_segment("note-1", "speaker-1", 20.0, 25.0))
            .unwrap();

        // Speaker 2: 10-20 (10s) = 10s total, 1 turn
        db.insert_speaker_segment(&create_test_segment("note-1", "speaker-2", 10.0, 20.0))
            .unwrap();

        let stats = db.get_speaker_statistics("note-1").unwrap();
        assert_eq!(stats.len(), 2);

        // First entry should be speaker-1 (most talk time)
        assert_eq!(stats[0].0, "speaker-1");
        assert_eq!(stats[0].1, 15.0); // 10 + 5
        assert_eq!(stats[0].2, 2); // 2 turns

        // Second entry should be speaker-2
        assert_eq!(stats[1].0, "speaker-2");
        assert_eq!(stats[1].1, 10.0);
        assert_eq!(stats[1].2, 1);
    }

    #[test]
    fn test_speaker_cascade_delete() {
        let db = setup();

        // Create prerequisites
        let note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();
        db.insert_speaker(&create_test_speaker("speaker-1", Some("Alice")))
            .unwrap();

        db.insert_speaker_segment(&create_test_segment("note-1", "speaker-1", 0.0, 10.0))
            .unwrap();
        assert_eq!(db.count_speaker_segments_by_note("note-1").unwrap(), 1);

        // Delete note should cascade delete segments
        db.delete_note("note-1").unwrap();
        assert_eq!(db.count_speaker_segments().unwrap(), 0);
    }

    // ===== SPEAKER PROFILE MANAGEMENT TESTS =====

    #[test]
    fn test_assign_speaker_to_segment() {
        let db = setup();

        // Create prerequisites
        let note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();
        db.insert_speaker(&create_test_speaker("speaker-1", Some("Alice")))
            .unwrap();
        db.insert_speaker(&create_test_speaker("speaker-2", Some("Bob")))
            .unwrap();

        let segment_data = create_test_segment("note-1", "speaker-1", 0.0, 10.0);
        let segment_id = segment_data.id.clone();
        db.insert_speaker_segment(&segment_data).unwrap();

        // Verify initial assignment
        let segment = db.get_speaker_segment(&segment_id).unwrap().unwrap();
        assert_eq!(segment.speaker_id, "speaker-1");

        // Reassign to speaker-2
        let updated = db
            .assign_speaker_to_segment(&segment_id, "speaker-2")
            .unwrap();
        assert_eq!(updated.speaker_id, "speaker-2");
        assert_eq!(updated.id, segment_id);
    }

    #[test]
    fn test_assign_nonexistent_speaker_to_segment() {
        let db = setup();

        // Create prerequisites
        let note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();
        db.insert_speaker(&create_test_speaker("speaker-1", Some("Alice")))
            .unwrap();

        let segment_data = create_test_segment("note-1", "speaker-1", 0.0, 10.0);
        let segment_id = segment_data.id.clone();
        db.insert_speaker_segment(&segment_data).unwrap();

        // Try to assign to non-existent speaker
        let result = db.assign_speaker_to_segment(&segment_id, "nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Speaker not found"));
    }

    #[test]
    fn test_assign_speaker_to_nonexistent_segment() {
        let db = setup();

        db.insert_speaker(&create_test_speaker("speaker-1", Some("Alice")))
            .unwrap();

        let result = db.assign_speaker_to_segment("nonexistent-segment", "speaker-1");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Speaker segment not found"));
    }

    #[test]
    fn test_compare_voice_signatures_identical() {
        let db = setup();

        let signature = "voice_data_abc123";
        let similarity = db.compare_voice_signatures(Some(signature), Some(signature));

        assert!(similarity.is_some());
        assert_eq!(similarity.unwrap(), 1.0);
    }

    #[test]
    fn test_compare_voice_signatures_different() {
        let db = setup();

        let sig1 = "voice_data_abc123";
        let sig2 = "voice_data_xyz789";
        let similarity = db.compare_voice_signatures(Some(sig1), Some(sig2));

        assert!(similarity.is_some());
        let score = similarity.unwrap();
        assert!(score >= 0.0 && score < 1.0);
    }

    #[test]
    fn test_compare_voice_signatures_similar() {
        let db = setup();

        let sig1 = "voice_data_abc123";
        let sig2 = "voice_data_abc124"; // Very similar
        let similarity = db.compare_voice_signatures(Some(sig1), Some(sig2));

        assert!(similarity.is_some());
        let score = similarity.unwrap();
        assert!(score > 0.8); // Should be high similarity
    }

    #[test]
    fn test_compare_voice_signatures_none() {
        let db = setup();

        let sig1 = "voice_data_abc123";

        // One signature is None
        let similarity1 = db.compare_voice_signatures(Some(sig1), None);
        assert!(similarity1.is_none());

        // Other signature is None
        let similarity2 = db.compare_voice_signatures(None, Some(sig1));
        assert!(similarity2.is_none());

        // Both are None
        let similarity3 = db.compare_voice_signatures(None, None);
        assert!(similarity3.is_none());
    }

    #[test]
    fn test_find_matching_speaker_exact_match() {
        let db = setup();

        let voice_sig = "voice_data_alice_123";
        let mut speaker = create_test_speaker("speaker-1", Some("Alice"));
        speaker.voice_signature = Some(voice_sig.to_string());
        db.insert_speaker(&speaker).unwrap();

        let result = db.find_matching_speaker(voice_sig, 0.9).unwrap();
        assert!(result.is_some());

        let (speaker_id, score) = result.unwrap();
        assert_eq!(speaker_id, "speaker-1");
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_find_matching_speaker_above_threshold() {
        let db = setup();

        let mut speaker1 = create_test_speaker("speaker-1", Some("Alice"));
        speaker1.voice_signature = Some("voice_data_abc123".to_string());
        db.insert_speaker(&speaker1).unwrap();

        let mut speaker2 = create_test_speaker("speaker-2", Some("Bob"));
        speaker2.voice_signature = Some("voice_data_xyz789".to_string());
        db.insert_speaker(&speaker2).unwrap();

        // Very similar to speaker1's signature
        let test_signature = "voice_data_abc124";
        let result = db.find_matching_speaker(test_signature, 0.8).unwrap();

        assert!(result.is_some());
        let (speaker_id, score) = result.unwrap();
        assert_eq!(speaker_id, "speaker-1");
        assert!(score > 0.8);
    }

    #[test]
    fn test_find_matching_speaker_below_threshold() {
        let db = setup();

        let mut speaker = create_test_speaker("speaker-1", Some("Alice"));
        speaker.voice_signature = Some("voice_data_abc123".to_string());
        db.insert_speaker(&speaker).unwrap();

        // Very different signature
        let test_signature = "completely_different_voice_xyz789";
        let result = db.find_matching_speaker(test_signature, 0.9).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_find_matching_speaker_no_speakers() {
        let db = setup();

        let test_signature = "voice_data_test";
        let result = db.find_matching_speaker(test_signature, 0.8).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_find_matching_speaker_best_match() {
        let db = setup();

        // Create multiple speakers with different signatures
        let mut speaker1 = create_test_speaker("speaker-1", Some("Alice"));
        speaker1.voice_signature = Some("voice_data_abc".to_string());
        db.insert_speaker(&speaker1).unwrap();

        let mut speaker2 = create_test_speaker("speaker-2", Some("Bob"));
        speaker2.voice_signature = Some("voice_data_abcd".to_string());
        db.insert_speaker(&speaker2).unwrap();

        let mut speaker3 = create_test_speaker("speaker-3", Some("Charlie"));
        speaker3.voice_signature = Some("voice_data_xyz".to_string());
        db.insert_speaker(&speaker3).unwrap();

        // Test signature most similar to speaker2
        let test_signature = "voice_data_abcde";
        let result = db.find_matching_speaker(test_signature, 0.5).unwrap();

        assert!(result.is_some());
        let (speaker_id, _score) = result.unwrap();
        assert_eq!(speaker_id, "speaker-2"); // Best match
    }

    #[test]
    fn test_auto_assign_speakers_to_segments_success() {
        let db = setup();

        // Create note and speakers
        let note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();

        let mut speaker1 = create_test_speaker("speaker-1", Some("Alice"));
        speaker1.voice_signature = Some("voice_alice_123".to_string());
        db.insert_speaker(&speaker1).unwrap();

        let mut speaker2 = create_test_speaker("speaker-2", Some("Bob"));
        speaker2.voice_signature = Some("voice_bob_456".to_string());
        db.insert_speaker(&speaker2).unwrap();

        // Create dummy speaker for initial assignment
        db.insert_speaker(&create_test_speaker("speaker-unknown", Some("Unknown")))
            .unwrap();

        // Create segments with unknown speaker
        let segment1 = create_test_segment("note-1", "speaker-unknown", 0.0, 10.0);
        let segment2 = create_test_segment("note-1", "speaker-unknown", 10.0, 20.0);
        let segment1_id = segment1.id.clone();
        let segment2_id = segment2.id.clone();
        db.insert_speaker_segment(&segment1).unwrap();
        db.insert_speaker_segment(&segment2).unwrap();

        // Auto-assign based on voice signatures
        let signature_map = vec![
            (segment1_id.clone(), "voice_alice_123".to_string()),
            (segment2_id.clone(), "voice_bob_456".to_string()),
        ];

        let assigned_count = db
            .auto_assign_speakers_to_segments("note-1", &signature_map, 0.9)
            .unwrap();
        assert_eq!(assigned_count, 2);

        // Verify assignments
        let seg1 = db.get_speaker_segment(&segment1_id).unwrap().unwrap();
        assert_eq!(seg1.speaker_id, "speaker-1");

        let seg2 = db.get_speaker_segment(&segment2_id).unwrap().unwrap();
        assert_eq!(seg2.speaker_id, "speaker-2");
    }

    #[test]
    fn test_auto_assign_speakers_partial_match() {
        let db = setup();

        // Create note and speaker
        let note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();

        let mut speaker = create_test_speaker("speaker-1", Some("Alice"));
        speaker.voice_signature = Some("voice_alice_123".to_string());
        db.insert_speaker(&speaker).unwrap();

        db.insert_speaker(&create_test_speaker("speaker-unknown", Some("Unknown")))
            .unwrap();

        // Create two segments
        let segment1 = create_test_segment("note-1", "speaker-unknown", 0.0, 10.0);
        let segment2 = create_test_segment("note-1", "speaker-unknown", 10.0, 20.0);
        let segment1_id = segment1.id.clone();
        let segment2_id = segment2.id.clone();
        db.insert_speaker_segment(&segment1).unwrap();
        db.insert_speaker_segment(&segment2).unwrap();

        // One matching, one non-matching
        let signature_map = vec![
            (segment1_id.clone(), "voice_alice_123".to_string()),
            (
                segment2_id.clone(),
                "completely_different_voice".to_string(),
            ),
        ];

        let assigned_count = db
            .auto_assign_speakers_to_segments("note-1", &signature_map, 0.9)
            .unwrap();
        assert_eq!(assigned_count, 1); // Only one should match

        // Verify only segment1 was reassigned
        let seg1 = db.get_speaker_segment(&segment1_id).unwrap().unwrap();
        assert_eq!(seg1.speaker_id, "speaker-1");

        let seg2 = db.get_speaker_segment(&segment2_id).unwrap().unwrap();
        assert_eq!(seg2.speaker_id, "speaker-unknown"); // Unchanged
    }

    #[test]
    fn test_auto_assign_speakers_no_matches() {
        let db = setup();

        let note = InsertNote {
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            content: None,
            folder_id: None,
            audio_url: None,
            duration: None,
            rating: None,
        };
        db.insert_note(&note).unwrap();

        let mut speaker = create_test_speaker("speaker-1", Some("Alice"));
        speaker.voice_signature = Some("voice_alice_123".to_string());
        db.insert_speaker(&speaker).unwrap();

        db.insert_speaker(&create_test_speaker("speaker-unknown", Some("Unknown")))
            .unwrap();

        let segment = create_test_segment("note-1", "speaker-unknown", 0.0, 10.0);
        let segment_id = segment.id.clone();
        db.insert_speaker_segment(&segment).unwrap();

        // No matching signatures
        let signature_map = vec![(segment_id.clone(), "completely_different_voice".to_string())];

        let assigned_count = db
            .auto_assign_speakers_to_segments("note-1", &signature_map, 0.9)
            .unwrap();
        assert_eq!(assigned_count, 0);

        // Verify segment unchanged
        let seg = db.get_speaker_segment(&segment_id).unwrap().unwrap();
        assert_eq!(seg.speaker_id, "speaker-unknown");
    }

    // ===== WHISPER NOTES TESTS =====

    fn create_test_whisper_note(
        id: &str,
        content: &str,
        audio_url: Option<String>,
    ) -> InsertWhisperNote {
        InsertWhisperNote {
            id: id.to_string(),
            content: content.to_string(),
            audio_url,
        }
    }

    #[test]
    fn test_create_whisper_note_basic() {
        let db = setup();

        let whisper = create_test_whisper_note("whisper-1", "Test whisper content", None);
        let result = db.create_whisper_note(&whisper).unwrap();

        assert_eq!(result.id, "whisper-1");
        assert_eq!(result.content, "Test whisper content");
        assert_eq!(result.audio_url, None);
        assert!(result.synced_at.is_none());
    }

    #[test]
    fn test_create_whisper_note_with_audio() {
        let db = setup();

        let whisper = create_test_whisper_note(
            "whisper-1",
            "Test whisper with audio",
            Some("audio/whisper1.m4a".to_string()),
        );
        let result = db.create_whisper_note(&whisper).unwrap();

        assert_eq!(result.id, "whisper-1");
        assert_eq!(result.content, "Test whisper with audio");
        assert_eq!(result.audio_url, Some("audio/whisper1.m4a".to_string()));
    }

    #[test]
    fn test_get_whisper_note_exists() {
        let db = setup();

        let whisper = create_test_whisper_note("whisper-1", "Test content", None);
        db.create_whisper_note(&whisper).unwrap();

        let result = db.get_whisper_note("whisper-1").unwrap();
        assert!(result.is_some());

        let whisper = result.unwrap();
        assert_eq!(whisper.id, "whisper-1");
        assert_eq!(whisper.content, "Test content");
    }

    #[test]
    fn test_get_whisper_note_not_exists() {
        let db = setup();

        let result = db.get_whisper_note("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_whisper_notes_empty() {
        let db = setup();

        let result = db.list_whisper_notes(0, 10).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_list_whisper_notes_pagination() {
        let db = setup();

        // Create 5 whisper notes
        for i in 0..5 {
            let whisper = create_test_whisper_note(
                &format!("whisper-{}", i),
                &format!("Content {}", i),
                None,
            );
            db.create_whisper_note(&whisper).unwrap();
        }

        // Test first page
        let page1 = db.list_whisper_notes(0, 2).unwrap();
        assert_eq!(page1.len(), 2);

        // Test second page
        let page2 = db.list_whisper_notes(1, 2).unwrap();
        assert_eq!(page2.len(), 2);

        // Test third page
        let page3 = db.list_whisper_notes(2, 2).unwrap();
        assert_eq!(page3.len(), 1);

        // Test beyond available data
        let page4 = db.list_whisper_notes(3, 2).unwrap();
        assert_eq!(page4.len(), 0);
    }

    #[test]
    fn test_list_whisper_notes_order() {
        let db = setup();

        // Create whisper notes (newest should come first)
        let whisper1 = create_test_whisper_note("whisper-1", "First", None);
        db.create_whisper_note(&whisper1).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let whisper2 = create_test_whisper_note("whisper-2", "Second", None);
        db.create_whisper_note(&whisper2).unwrap();

        let result = db.list_whisper_notes(0, 10).unwrap();
        assert_eq!(result.len(), 2);
        // Most recent first
        assert_eq!(result[0].id, "whisper-2");
        assert_eq!(result[1].id, "whisper-1");
    }

    #[test]
    fn test_delete_whisper_note_success() {
        let db = setup();

        let whisper = create_test_whisper_note("whisper-1", "Test", None);
        db.create_whisper_note(&whisper).unwrap();

        let result = db.delete_whisper_note("whisper-1");
        assert!(result.is_ok());

        // Verify deletion
        let get_result = db.get_whisper_note("whisper-1").unwrap();
        assert!(get_result.is_none());
    }

    #[test]
    fn test_delete_whisper_note_not_found() {
        let db = setup();

        let result = db.delete_whisper_note("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_convert_whisper_to_note_basic() {
        let db = setup();

        let whisper = create_test_whisper_note(
            "whisper-1",
            "Convert me to a note",
            Some("audio/whisper1.m4a".to_string()),
        );
        db.create_whisper_note(&whisper).unwrap();

        let result = db.convert_whisper_to_note("whisper-1", None).unwrap();

        // Verify note was created
        assert_eq!(result.id, "whisper-1");
        assert_eq!(result.title, "Convert me to a note");
        assert_eq!(result.content, Some("Convert me to a note".to_string()));
        assert_eq!(result.audio_url, Some("audio/whisper1.m4a".to_string()));
        assert_eq!(result.folder_id, None);

        // Verify whisper was deleted
        let whisper_check = db.get_whisper_note("whisper-1").unwrap();
        assert!(whisper_check.is_none());
    }

    #[test]
    fn test_convert_whisper_to_note_with_folder() {
        let db = setup();

        // Create folder
        db.insert_folder("folder-1", "Test Folder").unwrap();

        let whisper = create_test_whisper_note("whisper-1", "Content", None);
        db.create_whisper_note(&whisper).unwrap();

        let result = db
            .convert_whisper_to_note("whisper-1", Some("folder-1".to_string()))
            .unwrap();

        assert_eq!(result.folder_id, Some("folder-1".to_string()));
    }

    #[test]
    fn test_convert_whisper_to_note_not_found() {
        let db = setup();

        let result = db.convert_whisper_to_note("nonexistent", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_convert_whisper_to_todo_basic() {
        let db = setup();

        let whisper = create_test_whisper_note("whisper-1", "Buy groceries", None);
        db.create_whisper_note(&whisper).unwrap();

        let result = db.convert_whisper_to_todo("whisper-1").unwrap();

        // Verify todo was created
        assert_eq!(result.id, "whisper-1");
        assert_eq!(result.description, "Buy groceries");
        assert_eq!(result.state, TodoState::Open);
        assert_eq!(result.due_date, None);
        assert_eq!(result.smart_label, None);

        // Verify whisper was deleted
        let whisper_check = db.get_whisper_note("whisper-1").unwrap();
        assert!(whisper_check.is_none());
    }

    #[test]
    fn test_convert_whisper_to_todo_not_found() {
        let db = setup();

        let result = db.convert_whisper_to_todo("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_count_whisper_notes_empty() {
        let db = setup();

        let count = db.count_whisper_notes().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_whisper_notes_multiple() {
        let db = setup();

        for i in 0..5 {
            let whisper = create_test_whisper_note(&format!("whisper-{}", i), "Content", None);
            db.create_whisper_note(&whisper).unwrap();
        }

        let count = db.count_whisper_notes().unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_count_whisper_notes_after_deletion() {
        let db = setup();

        for i in 0..3 {
            let whisper = create_test_whisper_note(&format!("whisper-{}", i), "Content", None);
            db.create_whisper_note(&whisper).unwrap();
        }

        assert_eq!(db.count_whisper_notes().unwrap(), 3);

        db.delete_whisper_note("whisper-1").unwrap();
        assert_eq!(db.count_whisper_notes().unwrap(), 2);
    }

    #[test]
    fn test_count_whisper_notes_after_conversion() {
        let db = setup();

        for i in 0..3 {
            let whisper = create_test_whisper_note(&format!("whisper-{}", i), "Content", None);
            db.create_whisper_note(&whisper).unwrap();
        }

        assert_eq!(db.count_whisper_notes().unwrap(), 3);

        db.convert_whisper_to_note("whisper-1", None).unwrap();
        assert_eq!(db.count_whisper_notes().unwrap(), 2);

        db.convert_whisper_to_todo("whisper-2").unwrap();
        assert_eq!(db.count_whisper_notes().unwrap(), 1);
    }

    #[test]
    fn test_whisper_note_persistence() {
        let db = setup();

        let whisper = create_test_whisper_note(
            "whisper-1",
            "Persistent content",
            Some("audio.m4a".to_string()),
        );
        db.create_whisper_note(&whisper).unwrap();

        // Retrieve and verify all fields
        let retrieved = db.get_whisper_note("whisper-1").unwrap().unwrap();
        assert_eq!(retrieved.id, "whisper-1");
        assert_eq!(retrieved.content, "Persistent content");
        assert_eq!(retrieved.audio_url, Some("audio.m4a".to_string()));
        assert!(retrieved.created_at <= Utc::now());
        assert!(retrieved.synced_at.is_none());
    }
}
