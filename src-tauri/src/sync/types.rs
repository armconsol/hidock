use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a change from the server's /v1/changes endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Change {
    pub id: String,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub operation: Operation,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Note,
    Folder,
    Todo,
    CalendarEvent,
    Template,
    Settings,
}

impl EntityType {
    pub fn as_str(&self) -> &str {
        match self {
            EntityType::Note => "note",
            EntityType::Folder => "folder",
            EntityType::Todo => "todo",
            EntityType::CalendarEvent => "calendar_event",
            EntityType::Template => "template",
            EntityType::Settings => "settings",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Create,
    Update,
    Delete,
}

impl Operation {
    pub fn as_str(&self) -> &str {
        match self {
            Operation::Create => "create",
            Operation::Update => "update",
            Operation::Delete => "delete",
        }
    }
}

/// Response from /v1/changes endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesResponse {
    pub changes: Vec<Change>,
    pub has_more: bool,
    pub latest_timestamp: Option<DateTime<Utc>>,
}

/// Represents a pending operation to be synced to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOperation {
    pub id: i64,
    pub operation_type: Operation,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub retry_count: i32,
}

/// Sync state stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub last_sync_timestamp: Option<DateTime<Utc>>,
    pub is_syncing: bool,
    pub last_error: Option<String>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            last_sync_timestamp: None,
            is_syncing: false,
            last_error: None,
        }
    }
}
