use anyhow::{Context, Result};
use chrono::Utc;
use log::{debug, error, info, warn};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time;

use crate::api::client::HiNotesClient;
use crate::db::types::{Folder, InsertNote, Note, Template, Todo};
use crate::db::Database;

use super::types::{Change, ChangesResponse, EntityType, Operation, PendingOperation, SyncState};

const SYNC_INTERVAL_SECS: u64 = 30;
const MAX_RETRY_COUNT: i32 = 5;

/// The main sync engine that coordinates offline-first synchronization
///
/// NOTE: Due to rusqlite::Connection not being Send + Sync, this engine
/// creates a new Database connection for each operation. In production,
/// you should use a proper connection pool or async SQLite library.
pub struct SyncEngine {
    db_path: PathBuf,
    client: Arc<HiNotesClient>,
    state: Arc<RwLock<SyncState>>,
    sync_task: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl SyncEngine {
    /// Create a new SyncEngine instance with database path
    pub fn new(db_path: PathBuf, client: Arc<HiNotesClient>) -> Self {
        Self {
            db_path,
            client,
            state: Arc::new(RwLock::new(SyncState::default())),
            sync_task: Arc::new(RwLock::new(None)),
        }
    }

    /// Helper to get a database connection
    fn get_db(&self) -> Result<Database> {
        Database::new(&self.db_path)
    }

    /// Start the background sync loop
    pub async fn start(&self) -> Result<()> {
        let mut task_lock = self.sync_task.write().await;

        // Don't start if already running
        if task_lock.is_some() {
            warn!("Sync engine already running");
            return Ok(());
        }

        info!("Starting sync engine with {}s interval", SYNC_INTERVAL_SECS);

        let db_path = self.db_path.clone();
        let client = Arc::clone(&self.client);
        let state = Arc::clone(&self.state);

        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(SYNC_INTERVAL_SECS));

            loop {
                interval.tick().await;

                // Check if we have network connectivity and auth token
                if client.get_token().await.is_none() {
                    debug!("Skipping sync - no auth token");
                    continue;
                }

                // Perform sync cycle
                let engine = SyncEngine {
                    db_path: db_path.clone(),
                    client: Arc::clone(&client),
                    state: Arc::clone(&state),
                    sync_task: Arc::new(RwLock::new(None)),
                };

                if let Err(e) = engine.sync_cycle().await {
                    error!("Sync cycle failed: {}", e);

                    let mut state_lock = state.write().await;
                    state_lock.last_error = Some(e.to_string());
                    state_lock.is_syncing = false;
                }
            }
        });

        *task_lock = Some(handle);
        Ok(())
    }

    /// Stop the background sync loop
    pub async fn stop(&self) {
        let mut task_lock = self.sync_task.write().await;

        if let Some(handle) = task_lock.take() {
            info!("Stopping sync engine");
            handle.abort();
        }
    }

    /// Get the current sync state
    pub async fn get_state(&self) -> SyncState {
        self.state.read().await.clone()
    }

    /// Perform a single sync cycle
    pub async fn sync_cycle(&self) -> Result<()> {
        // Mark sync as in progress
        {
            let mut state = self.state.write().await;
            if state.is_syncing {
                debug!("Sync already in progress, skipping");
                return Ok(());
            }
            state.is_syncing = true;
            state.last_error = None;
        }

        info!("Starting sync cycle");

        // Push pending operations first (local changes to server)
        if let Err(e) = self.push_pending_operations().await {
            error!("Failed to push pending operations: {}", e);
            // Continue with pull even if push fails
        }

        // Pull changes from server
        if let Err(e) = self.pull_changes().await {
            error!("Failed to pull changes: {}", e);
            let mut state = self.state.write().await;
            state.is_syncing = false;
            return Err(e);
        }

        // Update sync state
        {
            let mut state = self.state.write().await;
            state.last_sync_timestamp = Some(Utc::now());
            state.is_syncing = false;
        }

        info!("Sync cycle completed successfully");
        Ok(())
    }

    /// Push pending operations to the server
    async fn push_pending_operations(&self) -> Result<()> {
        let pending_ops = self.get_pending_operations()?;

        if pending_ops.is_empty() {
            debug!("No pending operations to push");
            return Ok(());
        }

        info!("Pushing {} pending operations", pending_ops.len());

        for op in pending_ops {
            match self.push_operation(&op).await {
                Ok(_) => {
                    // Remove from pending queue
                    self.remove_pending_operation(op.id)?;
                    debug!("Successfully pushed operation {}", op.id);
                }
                Err(e) => {
                    error!("Failed to push operation {}: {}", op.id, e);

                    // Increment retry count
                    self.increment_retry_count(op.id)?;

                    // Remove if exceeded max retries
                    if op.retry_count >= MAX_RETRY_COUNT {
                        warn!(
                            "Operation {} exceeded max retries, removing from queue",
                            op.id
                        );
                        self.remove_pending_operation(op.id)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Push a single operation to the server
    async fn push_operation(&self, op: &PendingOperation) -> Result<()> {
        let endpoint = format!(
            "{}/{}",
            self.client_base_url(),
            self.endpoint_for_entity(&op.entity_type)
        );

        let token = self
            .client
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("No auth token available"))?;

        match op.operation_type {
            Operation::Create | Operation::Update => {
                // POST or PUT request
                let response = reqwest::Client::new()
                    .post(&endpoint)
                    .bearer_auth(&token)
                    .json(&op.payload)
                    .send()
                    .await
                    .context("Failed to send request")?;

                if !response.status().is_success() {
                    anyhow::bail!("Server returned error: {}", response.status());
                }
            }
            Operation::Delete => {
                // DELETE request
                let delete_url = format!("{}/{}", endpoint, op.entity_id);
                let response = reqwest::Client::new()
                    .delete(&delete_url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .context("Failed to send delete request")?;

                if !response.status().is_success() {
                    anyhow::bail!("Server returned error: {}", response.status());
                }
            }
        }

        Ok(())
    }

    /// Pull changes from the server
    async fn pull_changes(&self) -> Result<()> {
        let state = self.state.read().await;
        let since = state.last_sync_timestamp;
        drop(state);

        let endpoint = format!("{}/changes", self.client_base_url());
        let token = self
            .client
            .get_token()
            .await
            .ok_or_else(|| anyhow::anyhow!("No auth token available"))?;

        let mut query_params = vec![];
        if let Some(since_time) = since {
            query_params.push(("since", since_time.to_rfc3339()));
        }

        let response = reqwest::Client::new()
            .get(&endpoint)
            .bearer_auth(&token)
            .query(&query_params)
            .send()
            .await
            .context("Failed to fetch changes")?;

        if !response.status().is_success() {
            anyhow::bail!("Server returned error: {}", response.status());
        }

        let changes_response: ChangesResponse = response
            .json()
            .await
            .context("Failed to parse changes response")?;

        info!(
            "Received {} changes from server",
            changes_response.changes.len()
        );

        // Apply changes to local database
        self.apply_changes(&changes_response.changes).await?;

        Ok(())
    }

    /// Apply changes from the server to local database
    async fn apply_changes(&self, changes: &[Change]) -> Result<()> {
        for change in changes {
            if let Err(e) = self.apply_single_change(change).await {
                error!(
                    "Failed to apply change {} for {} {}: {}",
                    change.operation.as_str(),
                    change.entity_type.as_str(),
                    change.entity_id,
                    e
                );
                // Continue with other changes
            }
        }
        Ok(())
    }

    /// Apply a single change to the local database (last-write-wins)
    async fn apply_single_change(&self, change: &Change) -> Result<()> {
        debug!(
            "Applying {} for {} {}",
            change.operation.as_str(),
            change.entity_type.as_str(),
            change.entity_id
        );

        let db = self.get_db()?;

        match change.operation {
            Operation::Create | Operation::Update => match change.entity_type {
                EntityType::Note => {
                    let note: Note = serde_json::from_value(change.payload.clone())
                        .context("Failed to parse note")?;

                    // Check if note exists locally
                    if db.get_note(&note.id)?.is_some() {
                        // Update existing note - convert to UpdateNote
                        let update = crate::db::types::UpdateNote {
                            title: Some(note.title),
                            content: note.content,
                            folder_id: note.folder_id,
                            audio_url: note.audio_url,
                            duration: note.duration,
                            rating: note.rating,
                        };
                        db.update_note(&note.id, &update)?;
                    } else {
                        // Insert new note - convert to InsertNote
                        let insert = InsertNote {
                            id: note.id,
                            title: note.title,
                            content: note.content,
                            folder_id: note.folder_id,
                            audio_url: note.audio_url,
                            duration: note.duration,
                            rating: note.rating,
                        };
                        db.insert_note(&insert)?;
                    }
                }
                EntityType::Folder => {
                    let folder: Folder = serde_json::from_value(change.payload.clone())
                        .context("Failed to parse folder")?;

                    if db.get_folder(&folder.id)?.is_some() {
                        db.update_folder(&folder.id, &folder.name)?;
                    } else {
                        db.insert_folder(&folder.id, &folder.name)?;
                    }
                }
                EntityType::Todo => {
                    let todo: Todo = serde_json::from_value(change.payload.clone())
                        .context("Failed to parse todo")?;

                    if db.get_todo(&todo.id)?.is_some() {
                        let update = crate::db::types::UpdateTodo {
                            description: Some(todo.description),
                            due_date: todo.due_date,
                            state: Some(todo.state),
                            smart_label: todo.smart_label,
                        };
                        db.update_todo(&todo.id, update)?;
                    } else {
                        let insert = crate::db::types::InsertTodo {
                            id: todo.id,
                            description: todo.description,
                            due_date: todo.due_date,
                            state: todo.state,
                            smart_label: todo.smart_label,
                        };
                        db.insert_todo(insert)?;
                    }
                }
                EntityType::CalendarEvent => {
                    let event: crate::db::types::CalendarEvent =
                        serde_json::from_value(change.payload.clone())
                            .context("Failed to parse calendar event")?;

                    if db.get_calendar_event(&event.id)?.is_some() {
                        db.update_calendar_event(&event)?;
                    } else {
                        db.insert_calendar_event(&event)?;
                    }
                }
                EntityType::Template => {
                    let template: Template = serde_json::from_value(change.payload.clone())
                        .context("Failed to parse template")?;

                    if db.get_template(&template.id)?.is_some() {
                        db.update_template(
                            &template.id,
                            Some(&template.title),
                            Some(&template.content),
                            Some(template.is_favorite),
                            Some(template.is_default),
                        )?;
                    } else {
                        db.insert_template(
                            &template.id,
                            &template.title,
                            &template.content,
                            template.is_favorite,
                            template.is_default,
                        )?;
                    }
                }
                EntityType::Settings => {
                    // Settings are stored as key-value pairs in user_settings table
                    // Parse the payload as a map of setting key to value
                    let settings_map: std::collections::HashMap<String, String> =
                        serde_json::from_value(change.payload.clone())
                            .context("Failed to parse settings")?;

                    // Update each setting in the database
                    for (key, value) in settings_map {
                        db.set_user_setting(&key, &value)?;
                    }
                }
            },
            Operation::Delete => match change.entity_type {
                EntityType::Note => {
                    if db.get_note(&change.entity_id)?.is_some() {
                        db.delete_note(&change.entity_id)?;
                    }
                }
                EntityType::Folder => {
                    if db.get_folder(&change.entity_id)?.is_some() {
                        db.delete_folder(&change.entity_id)?;
                    }
                }
                EntityType::Todo => {
                    db.delete_todo(&change.entity_id)?;
                }
                EntityType::CalendarEvent => {
                    if db.get_calendar_event(&change.entity_id)?.is_some() {
                        db.delete_calendar_event(&change.entity_id)?;
                    }
                }
                EntityType::Template => {
                    if db.get_template(&change.entity_id)?.is_some() {
                        db.delete_template(&change.entity_id)?;
                    }
                }
                EntityType::Settings => {
                    // Delete a specific setting by key (entity_id is the setting key)
                    db.delete_user_setting(&change.entity_id)?;
                }
            },
        }

        Ok(())
    }

    // Helper methods for pending operations queue

    fn get_pending_operations(&self) -> Result<Vec<PendingOperation>> {
        let db = self.get_db()?;
        let rows = db.get_pending_operations()?;

        let mut operations = Vec::new();
        for row in rows {
            let operation_type = match row.operation_type.as_str() {
                "create" => Operation::Create,
                "update" => Operation::Update,
                "delete" => Operation::Delete,
                _ => continue,
            };

            let entity_type = match row.entity_type.as_str() {
                "note" => EntityType::Note,
                "folder" => EntityType::Folder,
                "todo" => EntityType::Todo,
                "calendar_event" => EntityType::CalendarEvent,
                "template" => EntityType::Template,
                "settings" => EntityType::Settings,
                _ => continue,
            };

            let payload: serde_json::Value = serde_json::from_str(&row.payload)?;
            let created_at =
                chrono::DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc);

            operations.push(PendingOperation {
                id: row.id,
                operation_type,
                entity_type,
                entity_id: row.entity_id,
                payload,
                created_at,
                retry_count: row.retry_count,
            });
        }

        Ok(operations)
    }

    fn remove_pending_operation(&self, id: i64) -> Result<()> {
        let db = self.get_db()?;
        db.remove_pending_operation(id)
    }

    fn increment_retry_count(&self, id: i64) -> Result<()> {
        let db = self.get_db()?;
        db.increment_pending_operation_retry(id)
    }

    // Helper methods for accessing internals

    fn client_base_url(&self) -> String {
        // Extract base URL from client - this is a simplified approach
        // In production, you'd store this in the client
        "http://localhost:3001/v1".to_string()
    }

    fn endpoint_for_entity(&self, entity_type: &EntityType) -> &str {
        match entity_type {
            EntityType::Note => "notes",
            EntityType::Folder => "folders",
            EntityType::Todo => "todos",
            EntityType::CalendarEvent => "calendar/events",
            EntityType::Template => "templates",
            EntityType::Settings => "user/setting/save",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_sync_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        Database::new(&db_path).unwrap(); // Initialize database

        let client = Arc::new(HiNotesClient::with_base_url(
            "http://localhost:3001/v1".to_string(),
        ));

        let engine = SyncEngine::new(db_path, client);
        let state = engine.get_state().await;

        assert!(state.last_sync_timestamp.is_none());
        assert!(!state.is_syncing);
        assert!(state.last_error.is_none());
    }

    #[tokio::test]
    async fn test_sync_state_management() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        Database::new(&db_path).unwrap(); // Initialize database

        let client = Arc::new(HiNotesClient::with_base_url(
            "http://localhost:3001/v1".to_string(),
        ));

        let engine = SyncEngine::new(db_path, client);

        // Initially not syncing
        assert!(!engine.get_state().await.is_syncing);

        // Manually set syncing state for testing
        {
            let mut state = engine.state.write().await;
            state.is_syncing = true;
        }

        assert!(engine.get_state().await.is_syncing);
    }

    #[tokio::test]
    async fn test_get_pending_operations_empty() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        Database::new(&db_path).unwrap(); // Initialize database

        let client = Arc::new(HiNotesClient::with_base_url(
            "http://localhost:3001/v1".to_string(),
        ));

        let engine = SyncEngine::new(db_path, client);
        let ops = engine.get_pending_operations().unwrap();

        assert_eq!(ops.len(), 0);
    }
}
