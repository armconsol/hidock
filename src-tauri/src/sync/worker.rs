use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{interval, sleep};

use crate::api::client::HiNotesClient;
use crate::db::Database;

const SYNC_INTERVAL_SECS: u64 = 30;
const MAX_RETRIES: i32 = 3;
const CONNECTIVITY_CHECK_URL: &str = "https://www.google.com";

/// Background sync worker that periodically syncs pending operations with the server
pub struct SyncWorker {
    db: Arc<Mutex<Database>>,
    client: Arc<HiNotesClient>,
    is_running: Arc<Mutex<bool>>,
}

impl SyncWorker {
    /// Create a new sync worker
    pub fn new(db: Arc<Mutex<Database>>, client: Arc<HiNotesClient>) -> Self {
        Self {
            db,
            client,
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the background sync loop
    ///
    /// This spawns a tokio task that runs indefinitely, syncing every 30 seconds
    /// when online connectivity is available.
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            anyhow::bail!("Sync worker is already running");
        }
        *is_running = true;
        drop(is_running);

        let db = self.db.clone();
        let client = self.client.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            log::info!("Sync worker started");
            let mut tick = interval(Duration::from_secs(SYNC_INTERVAL_SECS));

            loop {
                tick.tick().await;

                // Check if worker should still be running
                {
                    let running = is_running.lock().await;
                    if !*running {
                        log::info!("Sync worker stopping");
                        break;
                    }
                }

                // Check connectivity
                if !Self::check_connectivity().await {
                    log::debug!("No internet connectivity, skipping sync");
                    continue;
                }

                // Process pending operations
                match Self::process_pending_operations(&db, &client).await {
                    Ok(count) => {
                        if count > 0 {
                            log::info!("Processed {} pending operation(s)", count);
                        }
                    }
                    Err(e) => {
                        log::error!("Error processing pending operations: {}", e);
                    }
                }
            }

            log::info!("Sync worker stopped");
        });

        Ok(())
    }

    /// Stop the background sync loop
    pub async fn stop(&self) {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        log::info!("Sync worker stop requested");
    }

    /// Check if the worker is currently running
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }

    /// Check internet connectivity with a simple HTTP HEAD request
    async fn check_connectivity() -> bool {
        match reqwest::Client::new()
            .head(CONNECTIVITY_CHECK_URL)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(_) => true,
            Err(e) => {
                log::debug!("Connectivity check failed: {}", e);
                false
            }
        }
    }

    /// Process pending operations from the database
    ///
    /// Returns the number of operations successfully processed
    async fn process_pending_operations(
        db: &Arc<Mutex<Database>>,
        client: &Arc<HiNotesClient>,
    ) -> Result<usize> {
        let operations = {
            let db_lock = db.lock().await;
            db_lock.get_pending_operations()?
        };

        let mut processed_count = 0;

        for op in operations {
            // Skip operations that have exceeded max retries
            if op.retry_count >= MAX_RETRIES {
                log::warn!(
                    "Operation {} has exceeded max retries ({}), marking as permanently failed",
                    op.id,
                    MAX_RETRIES
                );
                // Remove the permanently failed operation
                let db_lock = db.lock().await;
                if let Err(e) = db_lock.remove_pending_operation(op.id) {
                    log::error!("Failed to remove permanently failed operation {}: {}", op.id, e);
                }
                continue;
            }

            // Process the operation
            match Self::sync_operation(client, &op).await {
                Ok(_) => {
                    log::info!(
                        "Successfully synced {} operation for {} {}",
                        op.operation_type,
                        op.entity_type,
                        op.entity_id
                    );

                    // Remove from pending operations
                    let db_lock = db.lock().await;
                    if let Err(e) = db_lock.remove_pending_operation(op.id) {
                        log::error!("Failed to remove completed operation {}: {}", op.id, e);
                    }

                    processed_count += 1;
                }
                Err(e) => {
                    log::warn!(
                        "Failed to sync operation {} (attempt {}/{}): {}",
                        op.id,
                        op.retry_count + 1,
                        MAX_RETRIES,
                        e
                    );

                    // Increment retry count with exponential backoff
                    let db_lock = db.lock().await;
                    if let Err(e) = db_lock.increment_pending_operation_retry(op.id) {
                        log::error!("Failed to increment retry count for operation {}: {}", op.id, e);
                    }

                    // Calculate exponential backoff delay
                    let backoff_secs = 2u64.pow(op.retry_count as u32);
                    log::debug!(
                        "Backing off for {} seconds before next retry",
                        backoff_secs
                    );
                    sleep(Duration::from_secs(backoff_secs)).await;
                }
            }
        }

        Ok(processed_count)
    }

    /// Sync a single operation to the server
    ///
    /// This is a placeholder implementation. Each operation type would need
    /// specific API calls to the HiNotes API (e.g., create note, update folder, etc.)
    async fn sync_operation(
        client: &Arc<HiNotesClient>,
        op: &crate::db::PendingOperationRow,
    ) -> Result<()> {
        // Parse the payload
        let _payload: serde_json::Value = serde_json::from_str(&op.payload)?;

        // Check if we have an auth token
        if client.get_token().await.is_none() {
            anyhow::bail!("Not authenticated - cannot sync operation");
        }

        // Route to appropriate API call based on operation type and entity type
        match (op.operation_type.as_str(), op.entity_type.as_str()) {
            ("create", "note") => {
                // TODO: Call client.create_note() when implemented
                log::warn!("Note creation sync not yet implemented");
                anyhow::bail!("Note creation sync not yet implemented");
            }
            ("update", "note") => {
                // TODO: Call client.update_note() when implemented
                log::warn!("Note update sync not yet implemented");
                anyhow::bail!("Note update sync not yet implemented");
            }
            ("delete", "note") => {
                // TODO: Call client.delete_note() when implemented
                log::warn!("Note deletion sync not yet implemented");
                anyhow::bail!("Note deletion sync not yet implemented");
            }
            ("create", "folder") => {
                // TODO: Call client.create_folder() when implemented
                log::warn!("Folder creation sync not yet implemented");
                anyhow::bail!("Folder creation sync not yet implemented");
            }
            ("update", "folder") => {
                // TODO: Call client.update_folder() when implemented
                log::warn!("Folder update sync not yet implemented");
                anyhow::bail!("Folder update sync not yet implemented");
            }
            ("delete", "folder") => {
                // TODO: Call client.delete_folder() when implemented
                log::warn!("Folder deletion sync not yet implemented");
                anyhow::bail!("Folder deletion sync not yet implemented");
            }
            ("create", "todo") => {
                // TODO: Call client.create_todo() when implemented
                log::warn!("Todo creation sync not yet implemented");
                anyhow::bail!("Todo creation sync not yet implemented");
            }
            ("update", "todo") => {
                // TODO: Call client.update_todo() when implemented
                log::warn!("Todo update sync not yet implemented");
                anyhow::bail!("Todo update sync not yet implemented");
            }
            ("delete", "todo") => {
                // TODO: Call client.delete_todo() when implemented
                log::warn!("Todo deletion sync not yet implemented");
                anyhow::bail!("Todo deletion sync not yet implemented");
            }
            ("create", "template") => {
                // TODO: Call client.create_template() when implemented
                log::warn!("Template creation sync not yet implemented");
                anyhow::bail!("Template creation sync not yet implemented");
            }
            ("update", "template") => {
                // TODO: Call client.update_template() when implemented
                log::warn!("Template update sync not yet implemented");
                anyhow::bail!("Template update sync not yet implemented");
            }
            ("delete", "template") => {
                // TODO: Call client.delete_template() when implemented
                log::warn!("Template deletion sync not yet implemented");
                anyhow::bail!("Template deletion sync not yet implemented");
            }
            (op_type, entity_type) => {
                log::error!(
                    "Unknown operation type '{}' for entity type '{}'",
                    op_type,
                    entity_type
                );
                anyhow::bail!(
                    "Unknown operation type '{}' for entity type '{}'",
                    op_type,
                    entity_type
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::HiNotesClient;
    use crate::db::Database;

    #[tokio::test]
    async fn test_sync_worker_lifecycle() {
        let db = Arc::new(Mutex::new(Database::new_in_memory().unwrap()));
        let client = Arc::new(HiNotesClient::with_base_url("https://hinotes.hidock.com/v1".to_string()));
        let worker = SyncWorker::new(db, client);

        // Initially not running
        assert!(!worker.is_running().await);

        // Start the worker
        worker.start().await.unwrap();
        assert!(worker.is_running().await);

        // Stop the worker
        worker.stop().await;
        // Note: The worker may take a moment to stop, so we don't assert immediately
    }

    #[tokio::test]
    async fn test_connectivity_check() {
        // This test requires internet connectivity
        let result = SyncWorker::check_connectivity().await;
        // We don't assert true/false because it depends on network availability
        // Just ensure it doesn't panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_process_pending_operations_empty() {
        let db = Arc::new(Mutex::new(Database::new_in_memory().unwrap()));
        let client = Arc::new(HiNotesClient::with_base_url("https://hinotes.hidock.com/v1".to_string()));

        let result = SyncWorker::process_pending_operations(&db, &client).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let db = Arc::new(Mutex::new(Database::new_in_memory().unwrap()));
        let client = Arc::new(HiNotesClient::with_base_url("https://hinotes.hidock.com/v1".to_string()));

        // Add a pending operation
        let operation_id = {
            let db_lock = db.lock().await;
            db_lock
                .add_pending_operation(
                    "create",
                    "note",
                    "test-note-id",
                    r#"{"title":"Test Note"}"#,
                )
                .unwrap()
        };

        // Manually set retry count to MAX_RETRIES
        {
            let db_lock = db.lock().await;
            for _ in 0..MAX_RETRIES {
                db_lock
                    .increment_pending_operation_retry(operation_id)
                    .unwrap();
            }
        }

        // Process operations - should remove the operation
        let result = SyncWorker::process_pending_operations(&db, &client).await;
        assert!(result.is_ok());

        // Verify operation was removed
        let remaining_ops = {
            let db_lock = db.lock().await;
            db_lock.get_pending_operations().unwrap()
        };
        assert_eq!(remaining_ops.len(), 0);
    }
}
