// Database layer - SQLite

pub mod types;

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use types::{CalendarEvent, EventSource, Folder, InsertNote, InsertTodo, Note, PaginationParams, Todo, TodoState, UpdateNote, UpdateTodo};

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

/// Helper function to parse datetime from SQLite TEXT field (for use inside query_map with rusqlite::Error)
fn parse_datetime_sql(s: String) -> Result<DateTime<Utc>, rusqlite::Error> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| rusqlite::Error::InvalidQuery)
}

/// Helper function to parse optional datetime from SQLite TEXT field (for use inside query_map with rusqlite::Error)
fn parse_datetime_opt_sql(s: Option<String>) -> Result<Option<DateTime<Utc>>, rusqlite::Error> {
    match s {
        Some(s) => Ok(Some(parse_datetime_sql(s)?)),
        None => Ok(None),
    }
}

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create a new SQLite database
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Create an in-memory database for testing
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Database { conn };
        db.initialize()?;
        Ok(db)
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
            Some((id, title, content, folder_id, audio_url, duration, rating, created_at, updated_at, synced_at)) => {
                Ok(Some(Note {
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
                }))
            }
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
            let (id, title, content, folder_id, audio_url, duration, rating, created_at, updated_at, synced_at) = row?;
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
    pub fn list_notes_by_folder(&self, folder_id: &str, limit: i64, offset: i64) -> Result<Vec<Note>> {
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
            let (id, title, content, folder_id, audio_url, duration, rating, created_at, updated_at, synced_at) = row?;
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
            return self.get_note(id)?.ok_or_else(|| anyhow::anyhow!("Note not found"));
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
        let deleted = self.conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;

        if deleted == 0 {
            anyhow::bail!("Note not found: {}", id);
        }

        Ok(())
    }

    /// Count total notes
    pub fn count_notes(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;
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
            Some((id, description, due_date, state, smart_label, created_at, updated_at, synced_at)) => {
                Ok(Some(Todo {
                    id,
                    description,
                    due_date: parse_datetime_opt(due_date)?,
                    state: TodoState::from_str(&state)
                        .map_err(|e| anyhow::anyhow!("Invalid todo state: {}", e))?,
                    smart_label,
                    created_at: parse_datetime(created_at)?,
                    updated_at: parse_datetime(updated_at)?,
                    synced_at: parse_datetime_opt(synced_at)?,
                }))
            }
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
            let (id, description, due_date, state, smart_label, created_at, updated_at, synced_at) = row?;
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
            let (id, description, due_date, state, smart_label, created_at, updated_at, synced_at) = row?;
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

        let sql = format!(
            "UPDATE todos SET {} WHERE id = ?",
            update_parts.join(", ")
        );

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
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
        assert!(notes.iter().all(|n| n.folder_id == Some("folder-1".to_string())));
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
}
